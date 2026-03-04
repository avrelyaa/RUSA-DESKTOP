use crate::auth::{require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct DataRequestRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    data_type: Option<String>,
    status: Option<String>,
    requested_by: Option<Uuid>,
    reviewed_by: Option<Uuid>,
    review_notes: Option<String>,
    response_data: Option<String>,
    responded_by: Option<Uuid>,
    analyst_notes: Option<String>,
    assigned_to: Option<Uuid>,
    delivered_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

// Any authenticated user can submit
#[tauri::command]
pub async fn submit_data_request(
    token: String,
    title: String,
    description: String,
    data_type: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO data_requests (title, description, data_type, status, requested_by) VALUES ($1,$2,$3,'pending',$4) RETURNING id"
    )
    .bind(&title).bind(&description).bind(&data_type).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_DATA_REQUEST", Some("data_requests"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

// Role-scoped fetch:
// - the_statistician: sees ALL requests
// - data_analyst: sees only 'approved', 'processing', and 'analyst_submitted' requests
// - others: see only their own requests
#[tauri::command]
pub async fn get_data_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let cols = "id, title, description, data_type, status, requested_by, reviewed_by, review_notes, response_data, responded_by, analyst_notes, assigned_to, delivered_at, created_at";
    let rows = if session.role_name == "the_statistician" {
        sqlx::query_as::<_, DataRequestRow>(&format!(
            "SELECT {cols} FROM data_requests WHERE deleted_at IS NULL ORDER BY created_at DESC"
        )).fetch_all(db::get_db()).await
    } else if session.role_name == "data_analyst" {
        sqlx::query_as::<_, DataRequestRow>(&format!(
            "SELECT {cols} FROM data_requests WHERE status IN ('approved', 'processing', 'analyst_submitted') AND deleted_at IS NULL ORDER BY created_at DESC"
        )).fetch_all(db::get_db()).await
    } else {
        sqlx::query_as::<_, DataRequestRow>(&format!(
            "SELECT {cols} FROM data_requests WHERE requested_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        )).bind(session.user_id).fetch_all(db::get_db()).await
    }.map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

// Step 2: The Statistician approves or rejects, and optionally assigns to a data analyst
#[tauri::command]
pub async fn review_data_request(
    token: String,
    request_id: String,
    status: String,
    notes: Option<String>,
    assigned_to: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_statistician")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;
    let aid = assigned_to.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    sqlx::query("UPDATE data_requests SET status = $1, reviewed_by = $2, review_notes = $3, assigned_to = COALESCE($4, assigned_to) WHERE id = $5 AND deleted_at IS NULL")
        .bind(&status).bind(session.user_id).bind(&notes).bind(aid).bind(rid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REVIEW_DATA_REQUEST", Some("data_requests"), Some(rid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

// Step 2b: Data Analyst marks a request as processing (starts working on it)
#[tauri::command]
pub async fn start_processing(token: String, request_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "data_analyst")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    sqlx::query("UPDATE data_requests SET status = 'processing', assigned_to = COALESCE(assigned_to, $1) WHERE id = $2 AND status = 'approved' AND deleted_at IS NULL")
        .bind(session.user_id).bind(rid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "START_PROCESSING", Some("data_requests"), Some(rid), None, None).await;
    Ok(())
}

// Step 3: Data Analyst processes and submits results back to The Statistician
#[tauri::command]
pub async fn analyst_submit_response(
    token: String,
    request_id: String,
    response_data: String,
    analyst_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "data_analyst")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    sqlx::query("UPDATE data_requests SET response_data = $1, analyst_notes = $2, responded_by = $3, status = 'analyst_submitted' WHERE id = $4 AND status IN ('approved', 'processing') AND deleted_at IS NULL")
        .bind(&response_data).bind(&analyst_notes).bind(session.user_id).bind(rid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ANALYST_SUBMIT_RESPONSE", Some("data_requests"), Some(rid), None, None).await;
    Ok(())
}

// Step 4: The Statistician delivers to requester after final review
#[tauri::command]
pub async fn deliver_data_response(
    token: String,
    request_id: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_statistician")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    sqlx::query("UPDATE data_requests SET status = 'delivered', delivered_at = NOW() WHERE id = $1 AND status = 'analyst_submitted' AND deleted_at IS NULL")
        .bind(rid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "DELIVER_DATA_RESPONSE", Some("data_requests"), Some(rid), None, None).await;
    Ok(())
}
