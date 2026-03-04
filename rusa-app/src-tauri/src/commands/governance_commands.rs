use crate::auth::{require_role, require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

/// Minimum number of ballots required for a vote to be resolved.
const VOTE_QUORUM: i64 = 8;
/// Duration of each voting window (and snooze extension) in minutes.
const VOTE_WINDOW_MINUTES: i64 = 15;

#[derive(sqlx::FromRow, serde::Serialize)]
struct VoteRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    status: Option<String>,
    initiated_by: Option<Uuid>,
    interrupted_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    deadline: Option<chrono::DateTime<chrono::Utc>>,
    snooze_count: Option<i32>,
    target_id: Option<Uuid>,
    target_table: Option<String>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct VoteDetailRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    status: Option<String>,
    initiated_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    deadline: Option<chrono::DateTime<chrono::Utc>>,
    yes_count: Option<i64>,
    no_count: Option<i64>,
    abstain_count: Option<i64>,
    snooze_count: Option<i32>,
    target_id: Option<Uuid>,
    target_table: Option<String>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct MeetingRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    location: Option<String>,
    created_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct RelocationRow {
    id: Uuid,
    staff_id: Option<Uuid>,
    to_location: String,
    relocation_type: Option<String>,
    start_date: Option<chrono::DateTime<chrono::Utc>>,
    end_date: Option<chrono::DateTime<chrono::Utc>>,
    reason: Option<String>,
    requested_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct EventDocumentRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    event_date: Option<chrono::DateTime<chrono::Utc>>,
    venue: Option<String>,
    venue_invoice: Option<String>,
    logged_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn initiate_vote(
    token: String,
    title: String,
    description: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 3)?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO votes (title, description, status, initiated_by, deadline) VALUES ($1,$2,'open',$3, NOW() + ($4 * INTERVAL '1 minute')) RETURNING id"
    )
    .bind(&title).bind(&description).bind(session.user_id).bind(VOTE_WINDOW_MINUTES)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "INITIATE_VOTE", Some("votes"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn cast_vote(
    token: String,
    vote_id: String,
    decision: String,
    reasoning: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    // Directors only vote
    if session.tier < 3 {
        return Err("Only directors can cast votes".to_string());
    }

    // Enforce reasoning is non-empty
    if reasoning.trim().is_empty() {
        return Err("A reasoning/justification is required to cast a vote".to_string());
    }

    let vid = Uuid::parse_str(&vote_id).map_err(|_| "Invalid vote ID".to_string())?;

    // Check that the vote is still open and within the 15-minute window
    let vote_check: Option<(String, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        "SELECT status, deadline FROM votes WHERE id = $1 AND deleted_at IS NULL"
    )
    .bind(vid)
    .fetch_optional(db::get_db())
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

    // Upsert: allow switching within the time window, also update reasoning
    sqlx::query(
        r#"INSERT INTO vote_ballots (vote_id, voter_id, decision, reasoning, voted_at)
           VALUES ($1, $2, $3, $4, NOW())
           ON CONFLICT (vote_id, voter_id) DO UPDATE SET decision = $3, reasoning = $4, voted_at = NOW()"#
    )
    .bind(vid).bind(session.user_id).bind(&decision).bind(&reasoning)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CAST_VOTE", Some("vote_ballots"), Some(vid), None, Some(serde_json::json!({ "vote_id": vote_id, "decision": decision, "reasoning": reasoning }))).await;
    Ok(())
}

#[tauri::command]
pub async fn get_votes(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    // Find all votes that have passed their deadline and are still 'open'
    #[derive(sqlx::FromRow)]
    struct ExpiredVote {
        id: Uuid,
        target_id: Option<Uuid>,
        target_table: Option<String>,
        snooze_count: i32,
    }

    let expired: Vec<ExpiredVote> = sqlx::query_as(
        "SELECT id, target_id, target_table, snooze_count FROM votes WHERE status = 'open' AND deadline IS NOT NULL AND deadline < NOW() AND deleted_at IS NULL"
    )
    .fetch_all(db::get_db())
    .await
    .unwrap_or_default();

    for vote in &expired {
        // Count ballots
        let (yes_count, no_count, abstain_count): (i64, i64, i64) = sqlx::query_as(
            r#"SELECT
                COUNT(*) FILTER (WHERE decision = 'yes'),
                COUNT(*) FILTER (WHERE decision = 'no'),
                COUNT(*) FILTER (WHERE decision = 'abstain')
               FROM vote_ballots WHERE vote_id = $1"#
        )
        .bind(vote.id)
        .fetch_one(db::get_db())
        .await
        .unwrap_or((0, 0, 0));

        let total = yes_count + no_count + abstain_count;

        if total < VOTE_QUORUM {
            // Snooze: extend by VOTE_WINDOW_MINUTES, increment snooze_count
            sqlx::query(
                "UPDATE votes SET status = 'open', deadline = NOW() + ($1 * INTERVAL '1 minute'), snooze_count = snooze_count + 1 WHERE id = $2"
            )
            .bind(VOTE_WINDOW_MINUTES)
            .bind(vote.id)
            .execute(db::get_db())
            .await
            .ok();
            // TODO: trigger reminder notification to Directors here
        } else {
            // Quorum met — resolve
            let (new_vote_status, new_request_status) = if yes_count == 0 && no_count == 0 {
                // All abstain → failed
                ("failed", "rejected")
            } else if yes_count > no_count {
                ("passed", "approved")
            } else {
                ("failed", "rejected")
            };

            sqlx::query("UPDATE votes SET status = $1 WHERE id = $2")
                .bind(new_vote_status).bind(vote.id)
                .execute(db::get_db())
                .await
                .ok();

            // Sync linked request status
            if let (Some(tid), Some(ttable)) = (vote.target_id, vote.target_table.as_deref()) {
                match ttable {
                    "budget_requests" => {
                        sqlx::query("UPDATE budget_requests SET status = $1 WHERE id = $2 AND deleted_at IS NULL")
                            .bind(new_request_status).bind(tid)
                            .execute(db::get_db())
                            .await
                            .ok();
                    }
                    "test_proposals" => {
                        sqlx::query("UPDATE test_proposals SET status = $1 WHERE id = $2 AND deleted_at IS NULL")
                            .bind(new_request_status).bind(tid)
                            .execute(db::get_db())
                            .await
                            .ok();
                    }
                    _ => {}
                }
            }
        }
    }

    let rows = sqlx::query_as::<_, VoteRow>(
        "SELECT id, title, description, status, initiated_by, interrupted_by, created_at, deadline, snooze_count, target_id, target_table FROM votes WHERE deleted_at IS NULL ORDER BY created_at DESC"
    )
    .fetch_all(db::get_db())
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

    let row = sqlx::query_as::<_, VoteDetailRow>(
        r#"SELECT v.id, v.title, v.description, v.status, v.initiated_by, v.created_at, v.deadline,
                  (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = v.id AND decision = 'yes') as yes_count,
                  (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = v.id AND decision = 'no') as no_count,
                  (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = v.id AND decision = 'abstain') as abstain_count,
                  v.snooze_count, v.target_id, v.target_table
           FROM votes v WHERE v.id = $1"#
    )
    .bind(vid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?
    .ok_or_else(|| "Vote not found".to_string())?;

    Ok(serde_json::to_value(row).unwrap_or_default())
}

/// Creates a Governance vote linked to a specific request.
/// Called automatically when a pending request is inserted.
pub async fn create_governance_vote_for_request(
    initiated_by: Uuid,
    title: String,
    description: Option<String>,
    target_id: Uuid,
    target_table: &str,
) -> Result<Uuid, String> {
    let row: (Uuid,) = sqlx::query_as(
        r#"INSERT INTO votes (title, description, status, initiated_by, deadline, target_id, target_table)
           VALUES ($1, $2, 'open', $3, NOW() + ($6 * INTERVAL '1 minute'), $4, $5)
           RETURNING id"#
    )
    .bind(&title)
    .bind(&description)
    .bind(initiated_by)
    .bind(target_id)
    .bind(target_table)
    .bind(VOTE_WINDOW_MINUTES)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error creating governance vote: {}", e))?;

    let vote_id = row.0;

    let _ = write_audit_log(
        Some(initiated_by),
        "AUTO_INITIATE_VOTE",
        Some("votes"),
        Some(vote_id),
        None,
        Some(serde_json::json!({ "target_id": target_id, "target_table": target_table, "title": title })),
    ).await;

    Ok(vote_id)
}

#[tauri::command]
pub async fn interrupt_vote(token: String, vote_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_administrator")?;

    let vid = Uuid::parse_str(&vote_id).map_err(|_| "Invalid vote ID".to_string())?;

    sqlx::query("UPDATE votes SET status = 'interrupted', interrupted_by = $1 WHERE id = $2 AND status = 'open'")
        .bind(session.user_id).bind(vid)
        .execute(db::get_db())
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

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO meetings (title, description, scheduled_at, location, created_by) VALUES ($1,$2,$3,$4,$5) RETURNING id"
    )
    .bind(&title).bind(&description).bind(sched).bind(&location).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let meeting_id = row.0;

    for aid_str in &attendee_ids {
        if let Ok(aid) = Uuid::parse_str(aid_str) {
            let _ = sqlx::query("INSERT INTO meeting_attendees (meeting_id, user_id) VALUES ($1,$2) ON CONFLICT DO NOTHING")
                .bind(meeting_id).bind(aid)
                .execute(db::get_db())
                .await;
        }
    }

    let _ = write_audit_log(Some(session.user_id), "CREATE_MEETING", Some("meetings"), Some(meeting_id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(meeting_id.to_string())
}

#[tauri::command]
pub async fn get_meetings(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, MeetingRow>(
        "SELECT id, title, description, scheduled_at, location, created_by, created_at FROM meetings WHERE deleted_at IS NULL ORDER BY scheduled_at DESC"
    )
    .fetch_all(db::get_db())
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

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO staff_relocations (staff_id, to_location, relocation_type, start_date, end_date, reason, requested_by) VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id"
    )
    .bind(sid).bind(&to_location).bind(&relocation_type).bind(sdate).bind(edate).bind(&reason).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "RELOCATE_STAFF", Some("staff_relocations"), Some(row.0), None, Some(serde_json::json!({ "staff_id": staff_id, "to_location": to_location }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_relocations(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, RelocationRow>(
        "SELECT id, staff_id, to_location, relocation_type, start_date, end_date, reason, requested_by, created_at FROM staff_relocations ORDER BY created_at DESC"
    )
    .fetch_all(db::get_db())
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

    sqlx::query("INSERT INTO archive_permissions (table_name, record_id, access_level, set_by) VALUES ($1,$2,$3,$4) ON CONFLICT (table_name, record_id) DO UPDATE SET access_level = $3, set_by = $4")
        .bind(&table_name).bind(rid).bind(&access_level).bind(session.user_id)
        .execute(db::get_db())
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

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO event_documents (title, description, event_date, venue, venue_invoice, logged_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id"
    )
    .bind(&title).bind(&description).bind(edate).bind(&venue).bind(&venue_invoice).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "LOG_EVENT_DOCUMENT", Some("event_documents"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_event_documents(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, EventDocumentRow>(
        "SELECT id, title, description, event_date, venue, venue_invoice, logged_by, created_at FROM event_documents WHERE deleted_at IS NULL ORDER BY event_date DESC"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}
