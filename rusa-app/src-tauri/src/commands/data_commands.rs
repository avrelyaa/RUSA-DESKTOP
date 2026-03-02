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
    delivered_at: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

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

#[tauri::command]
pub async fn get_data_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if session.role_name == "the_statistician" {
        sqlx::query_as::<_, DataRequestRow>(
            "SELECT id, title, description, data_type, status, requested_by, reviewed_by, review_notes, response_data, responded_by, delivered_at, created_at FROM data_requests WHERE deleted_at IS NULL ORDER BY created_at DESC"
        ).fetch_all(db::get_db()).await
    } else {
        sqlx::query_as::<_, DataRequestRow>(
            "SELECT id, title, description, data_type, status, requested_by, reviewed_by, review_notes, response_data, responded_by, delivered_at, created_at FROM data_requests WHERE requested_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ).bind(session.user_id).fetch_all(db::get_db()).await
    }.map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn review_data_request(
    token: String,
    request_id: String,
    status: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_statistician")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    sqlx::query("UPDATE data_requests SET status = $1, reviewed_by = $2, review_notes = $3 WHERE id = $4 AND deleted_at IS NULL")
        .bind(&status).bind(session.user_id).bind(&notes).bind(rid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REVIEW_DATA_REQUEST", Some("data_requests"), Some(rid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

#[tauri::command]
pub async fn respond_to_data_request(
    token: String,
    request_id: String,
    response_data: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "data_analyst")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    sqlx::query("UPDATE data_requests SET response_data = $1, review_notes = $2, responded_by = $3, status = 'responded' WHERE id = $4 AND deleted_at IS NULL")
        .bind(&response_data).bind(&notes).bind(session.user_id).bind(rid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "RESPOND_DATA_REQUEST", Some("data_requests"), Some(rid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn deliver_data_response(
    token: String,
    request_id: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_statistician")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    sqlx::query("UPDATE data_requests SET status = 'delivered', delivered_at = NOW() WHERE id = $1 AND deleted_at IS NULL")
        .bind(rid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "DELIVER_DATA_RESPONSE", Some("data_requests"), Some(rid), None, None).await;
    Ok(())
}
