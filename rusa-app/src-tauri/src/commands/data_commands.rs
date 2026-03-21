use crate::auth::{require_role_name, validate_session_command};
use crate::queries::auth::write_audit_log;
use crate::queries::data as data_queries;
use uuid::Uuid;

// Any authenticated user can submit
#[tauri::command]
pub async fn submit_data_request(
    token: String,
    title: String,
    description: String,
    data_type: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    let id = data_queries::insert_data_request(
        &title,
        &description,
        data_type.as_deref(),
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_DATA_REQUEST", Some("data_requests"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}

// Role-scoped fetch:
// - the_statistician: sees ALL requests
// - data_analyst: sees only 'approved', 'processing', and 'analyst_submitted' requests
// - others: see only their own requests
#[tauri::command]
pub async fn get_data_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if session.role_name == "the_statistician" {
        data_queries::get_all_data_requests().await
    } else if session.role_name == "data_analyst" {
        data_queries::get_analyst_data_requests().await
    } else {
        data_queries::get_user_data_requests(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

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

    data_queries::update_data_request_review(rid, &status, session.user_id, notes.as_deref(), aid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REVIEW_DATA_REQUEST", Some("data_requests"), Some(rid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

// Step 2b: Data Analyst (or Statistician) marks a request as processing (starts working on it)
#[tauri::command]
pub async fn start_processing(token: String, request_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "data_analyst" && session.role_name != "the_statistician" {
        return Err("Only data_analyst or the_statistician can start processing requests".to_string());
    }

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    data_queries::update_data_request_processing(rid, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "START_PROCESSING", Some("data_requests"), Some(rid), None, None).await;
    Ok(())
}

// Step 3: Data Analyst (or Statistician) processes and submits results back to The Statistician
#[tauri::command]
pub async fn analyst_submit_response(
    token: String,
    request_id: String,
    response_data: String,
    analyst_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "data_analyst" && session.role_name != "the_statistician" {
        return Err("Only data_analyst or the_statistician can submit analysis responses".to_string());
    }

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    data_queries::update_data_request_analyst_response(
        rid,
        &response_data,
        analyst_notes.as_deref(),
        session.user_id,
    )
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

    data_queries::update_data_request_delivered(rid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "DELIVER_DATA_RESPONSE", Some("data_requests"), Some(rid), None, None).await;
    Ok(())
}
