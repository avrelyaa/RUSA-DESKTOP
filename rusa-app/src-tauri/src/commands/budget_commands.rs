use crate::auth::{require_role_name, validate_session_command};
use crate::queries::auth::write_audit_log;
use crate::queries::budget as budget_queries;
use uuid::Uuid;

#[tauri::command]
pub async fn get_budget_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if session.role_name == "the_accountant" {
        budget_queries::get_all_budget_requests().await
    } else {
        budget_queries::get_user_budget_requests(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn review_budget_request(
    token: String,
    request_id: String,
    status: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_accountant")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    budget_queries::update_budget_request_review(rid, &status, notes.as_deref())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REVIEW_BUDGET_REQUEST", Some("budget_requests"), Some(rid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

#[tauri::command]
pub async fn flag_budget_request(
    token: String,
    request_id: String,
    reason: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_accountant")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    budget_queries::flag_budget_request(rid, &reason)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "FLAG_BUDGET_REQUEST", Some("budget_requests"), Some(rid), None, Some(serde_json::json!({ "reason": reason }))).await;
    Ok(())
}

#[tauri::command]
pub async fn get_expenditure_reports(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if session.role_name == "the_accountant" {
        budget_queries::get_all_expenditure_reports().await
    } else {
        budget_queries::get_user_expenditure_reports(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn flag_expenditure_report(
    token: String,
    report_id: String,
    reason: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_accountant")?;

    let rid = Uuid::parse_str(&report_id).map_err(|_| "Invalid report ID".to_string())?;

    budget_queries::flag_expenditure_report(rid, &reason)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "FLAG_EXPENDITURE_REPORT", Some("expenditure_reports"), Some(rid), None, Some(serde_json::json!({ "reason": reason }))).await;
    Ok(())
}

#[tauri::command]
pub async fn submit_investigation(
    token: String,
    title: String,
    description: String,
    related_report_id: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_accountant")?;

    let rid = related_report_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let id = budget_queries::insert_investigation(&title, &description, rid, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_INVESTIGATION", Some("budget_investigations"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}
