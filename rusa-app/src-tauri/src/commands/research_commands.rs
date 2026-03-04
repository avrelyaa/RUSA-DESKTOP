use crate::auth::{require_role, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct ExperimentRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    experiment_type: Option<String>,
    status: Option<String>,
    proposed_by: Option<Uuid>,
    reviewed_by: Option<Uuid>,
    review_notes: Option<String>,
    start_date: Option<chrono::DateTime<chrono::Utc>>,
    end_date: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct SpeciesRow {
    id: Uuid,
    name: String,
    classification: Option<String>,
    description: Option<String>,
    habitat: Option<String>,
    discovered_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct TestProposalRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    methodology: Option<String>,
    status: Option<String>,
    proposed_by: Option<Uuid>,
    reviewed_by: Option<Uuid>,
    review_notes: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct ExperimentLogRow {
    id: Uuid,
    experiment_id: Uuid,
    log_date: Option<chrono::DateTime<chrono::Utc>>,
    personnel_present: Option<String>,
    species_matter_tested: Option<String>,
    tests_performed: Option<String>,
    notes: Option<String>,
    logged_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn get_experiments(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, ExperimentRow>(
        "SELECT id, title, description, experiment_type, status, proposed_by, reviewed_by, review_notes, start_date, end_date, created_at FROM experiments WHERE deleted_at IS NULL ORDER BY created_at DESC"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn propose_experiment(
    token: String,
    title: String,
    description: String,
    experiment_type: String,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    let sdate = start_date.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));
    let edate = end_date.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO experiments (title, description, experiment_type, status, proposed_by, start_date, end_date) VALUES ($1, $2, $3, 'pending', $4, $5, $6) RETURNING id"
    )
    .bind(&title).bind(&description).bind(&experiment_type)
    .bind(session.user_id).bind(sdate).bind(edate)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "PROPOSE_EXPERIMENT", Some("experiments"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn review_experiment(
    token: String,
    experiment_id: String,
    status: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    // Requires director or higher (the_observer role or tier >= 3)
    if session.role_name != "the_observer" {
        require_role(&session, 3)?;
    }

    let eid = Uuid::parse_str(&experiment_id).map_err(|_| "Invalid experiment ID".to_string())?;

    sqlx::query("UPDATE experiments SET status = $1, reviewed_by = $2, review_notes = $3 WHERE id = $4 AND deleted_at IS NULL")
        .bind(&status).bind(session.user_id).bind(&notes).bind(eid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REVIEW_EXPERIMENT", Some("experiments"), Some(eid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

#[tauri::command]
pub async fn add_experiment_log(
    token: String,
    experiment_id: String,
    log_date: String,
    personnel_present: Option<String>,
    species_matter_tested: Option<String>,
    tests_performed: Option<String>,
    notes: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let eid = Uuid::parse_str(&experiment_id).map_err(|_| "Invalid experiment ID".to_string())?;
    let ldate = chrono::DateTime::parse_from_rfc3339(&log_date)
        .map(|d| d.with_timezone(&chrono::Utc))
        .map_err(|_| "Invalid log_date format".to_string())?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO experiment_logs (experiment_id, log_date, personnel_present, species_matter_tested, tests_performed, notes, logged_by) VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id"
    )
    .bind(eid).bind(ldate).bind(&personnel_present).bind(&species_matter_tested)
    .bind(&tests_performed).bind(&notes).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ADD_EXPERIMENT_LOG", Some("experiment_logs"), Some(row.0), None, None).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_species_archive(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, SpeciesRow>(
        "SELECT id, name, classification, description, habitat, discovered_by, created_at FROM species_archive WHERE deleted_at IS NULL ORDER BY name"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn add_species(
    token: String,
    name: String,
    classification: Option<String>,
    description: Option<String>,
    habitat: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO species_archive (name, classification, description, habitat, discovered_by) VALUES ($1,$2,$3,$4,$5) RETURNING id"
    )
    .bind(&name).bind(&classification).bind(&description).bind(&habitat).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ADD_SPECIES", Some("species_archive"), Some(row.0), None, Some(serde_json::json!({ "name": name }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_test_archive(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, TestProposalRow>(
        "SELECT id, title, description, methodology, status, proposed_by, reviewed_by, review_notes, created_at FROM test_proposals WHERE deleted_at IS NULL ORDER BY created_at DESC"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn propose_test(
    token: String,
    title: String,
    description: Option<String>,
    methodology: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO test_proposals (title, description, methodology, status, proposed_by) VALUES ($1,$2,$3,'pending',$4) RETURNING id"
    )
    .bind(&title).bind(&description).bind(&methodology).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let proposal_id = row.0;

    // Automatically trigger a Governance vote for this test proposal
    let vote_title = format!("Test Proposal: {}", title);
    let vote_desc = description.as_ref().map(|d| format!("Auto-generated vote for test proposal. {}", d));
    let vote_id = crate::commands::governance_commands::create_governance_vote_for_request(
        session.user_id,
        vote_title,
        vote_desc,
        proposal_id,
        "test_proposals",
    ).await?;

    // Link the vote back to the proposal
    sqlx::query("UPDATE test_proposals SET vote_id = $1 WHERE id = $2")
        .bind(vote_id).bind(proposal_id)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error linking vote: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "PROPOSE_TEST", Some("test_proposals"), Some(proposal_id), None, Some(serde_json::json!({ "title": title, "vote_id": vote_id }))).await;
    Ok(proposal_id.to_string())
}

#[tauri::command]
pub async fn review_test_proposal(
    token: String,
    proposal_id: String,
    status: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    // Requires observer or artificer role
    if session.role_name != "the_observer" && session.role_name != "the_artificer" {
        require_role(&session, 3)?;
    }

    // approved/rejected must only come from Governance votes
    if status == "approved" || status == "rejected" {
        return Err("Test proposals can only be approved or rejected through a Governance vote by the Directors".to_string());
    }

    let pid = Uuid::parse_str(&proposal_id).map_err(|_| "Invalid proposal ID".to_string())?;

    sqlx::query("UPDATE test_proposals SET status = $1, reviewed_by = $2, review_notes = $3 WHERE id = $4 AND deleted_at IS NULL")
        .bind(&status).bind(session.user_id).bind(&notes).bind(pid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REVIEW_TEST_PROPOSAL", Some("test_proposals"), Some(pid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}
