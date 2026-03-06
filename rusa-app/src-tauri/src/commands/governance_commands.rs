use crate::auth::{require_role, require_role_name, validate_session_command};
use crate::queries::auth::write_audit_log;
use crate::queries::governance as governance_queries;
use uuid::Uuid;

#[tauri::command]
pub async fn initiate_vote(
    token: String,
    title: String,
    description: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    let id = governance_queries::insert_vote(&title, description.as_deref(), session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "INITIATE_VOTE", Some("votes"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn cast_vote(
    token: String,
    vote_id: String,
    decision: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    // Directors only vote
    if session.tier < 3 {
        return Err("Only directors can cast votes".to_string());
    }

    let vid = Uuid::parse_str(&vote_id).map_err(|_| "Invalid vote ID".to_string())?;

    // Check that the vote is still open and within the 15-minute window
    let vote_check = governance_queries::check_vote_status(vid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let (vstatus, vdeadline) = vote_check.ok_or_else(|| "Vote not found".to_string())?;
    if vstatus != "open" {
        return Err(format!("Vote is no longer open (status: {})", vstatus));
    }
    if let Some(dl) = vdeadline {
        if chrono::Utc::now() > dl {
            return Err("Voting window has expired (15-minute limit reached)".to_string());
        }
    }

    // Upsert: allow switching within the time window
    governance_queries::upsert_vote_ballot(vid, session.user_id, &decision)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CAST_VOTE", Some("vote_ballots"), Some(vid), None, Some(serde_json::json!({ "vote_id": vote_id, "decision": decision }))).await;
    Ok(())
}

#[tauri::command]
pub async fn get_votes(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    // Lazily resolve expired votes
    governance_queries::resolve_expired_votes().await.ok(); // best-effort, don't fail on error

    let rows = governance_queries::get_all_votes()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn get_vote_details(
    token: String,
    vote_id: String,
) -> Result<serde_json::Value, String> {
    let _session = validate_session_command(&token).await?;
    let vid = Uuid::parse_str(&vote_id).map_err(|_| "Invalid vote ID".to_string())?;

    let row = governance_queries::get_vote_details(vid)
        .await
        .map_err(|e| format!("DB error: {}", e))?
        .ok_or_else(|| "Vote not found".to_string())?;

    Ok(serde_json::to_value(row).unwrap_or_default())
}

#[tauri::command]
pub async fn interrupt_vote(token: String, vote_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_administrator")?;

    let vid = Uuid::parse_str(&vote_id).map_err(|_| "Invalid vote ID".to_string())?;

    governance_queries::interrupt_vote(vid, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "INTERRUPT_VOTE", Some("votes"), Some(vid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn create_meeting(
    token: String,
    title: String,
    description: Option<String>,
    scheduled_at: String,
    location: Option<String>,
    attendee_ids: Vec<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    let sched = chrono::DateTime::parse_from_rfc3339(&scheduled_at)
        .map(|d| d.with_timezone(&chrono::Utc))
        .map_err(|_| "Invalid scheduled_at format".to_string())?;

    let meeting_id = governance_queries::insert_meeting(
        &title,
        description.as_deref(),
        sched,
        location.as_deref(),
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    for aid_str in &attendee_ids {
        if let Ok(aid) = Uuid::parse_str(aid_str) {
            let _ = governance_queries::insert_meeting_attendee(meeting_id, aid).await;
        }
    }

    let _ = write_audit_log(Some(session.user_id), "CREATE_MEETING", Some("meetings"), Some(meeting_id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(meeting_id.to_string())
}

#[tauri::command]
pub async fn get_meetings(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = governance_queries::get_all_meetings()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn relocate_staff(
    token: String,
    staff_id: String,
    to_location: String,
    relocation_type: String,
    start_date: Option<String>,
    end_date: Option<String>,
    reason: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "the_nomad" && session.role_name != "the_administrator" {
        return Err("Only nomad or administrator can relocate staff".to_string());
    }

    let sid = Uuid::parse_str(&staff_id).map_err(|_| "Invalid staff ID".to_string())?;
    let sdate = start_date.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));
    let edate = end_date.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let id = governance_queries::insert_staff_relocation(
        sid,
        &to_location,
        &relocation_type,
        sdate,
        edate,
        reason.as_deref(),
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "RELOCATE_STAFF", Some("staff_relocations"), Some(id), None, Some(serde_json::json!({ "staff_id": staff_id, "to_location": to_location }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_relocations(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = governance_queries::get_all_relocations()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn set_archive_permission(
    token: String,
    table_name: String,
    record_id: String,
    access_level: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_librarian")?;

    let rid = Uuid::parse_str(&record_id).map_err(|_| "Invalid record ID".to_string())?;

    governance_queries::upsert_archive_permission(&table_name, rid, &access_level, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SET_ARCHIVE_PERMISSION", Some("archive_permissions"), Some(rid), None, Some(serde_json::json!({ "table_name": table_name, "access_level": access_level }))).await;
    Ok(())
}

#[tauri::command]
pub async fn log_event_document(
    token: String,
    title: String,
    description: Option<String>,
    event_date: Option<String>,
    venue: Option<String>,
    venue_invoice: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    // coordinator role (excluded but still seeded)
    require_role_name(&session, "the_coordinator")?;

    let edate = event_date.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let id = governance_queries::insert_event_document(
        &title,
        description.as_deref(),
        edate,
        venue.as_deref(),
        venue_invoice.as_deref(),
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "LOG_EVENT_DOCUMENT", Some("event_documents"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_event_documents(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = governance_queries::get_all_event_documents()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}
