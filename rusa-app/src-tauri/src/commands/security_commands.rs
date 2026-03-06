use crate::auth::{require_role_name, validate_session_command};
use crate::queries::auth::write_audit_log;
use crate::queries::security as security_queries;
use uuid::Uuid;

#[tauri::command]
pub async fn create_incident_report(
    token: String,
    title: String,
    description: String,
    location: Option<String>,
    incident_date: Option<String>,
    severity: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let idate = incident_date.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let id = security_queries::insert_incident_report(
        &title,
        &description,
        location.as_deref(),
        idate,
        &severity,
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_INCIDENT_REPORT", Some("incident_reports"), Some(id), None, Some(serde_json::json!({ "title": title, "severity": severity }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_incident_reports(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = security_queries::get_all_incident_reports()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn update_incident_status(
    token: String,
    report_id: String,
    status: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    let rid = Uuid::parse_str(&report_id).map_err(|_| "Invalid report ID".to_string())?;

    security_queries::update_incident_status(rid, &status)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_INCIDENT_STATUS", Some("incident_reports"), Some(rid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

#[tauri::command]
pub async fn add_lost_found_item(
    token: String,
    item_name: String,
    description: Option<String>,
    found_location: Option<String>,
    found_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "earth_security_head")?;

    let fdate = found_date.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let id = security_queries::insert_lost_found_item(
        &item_name,
        description.as_deref(),
        found_location.as_deref(),
        fdate,
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ADD_LOST_FOUND", Some("lost_and_found"), Some(id), None, Some(serde_json::json!({ "item_name": item_name }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_lost_found(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = security_queries::get_all_lost_found()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn claim_lost_found(token: String, item_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    let iid = Uuid::parse_str(&item_id).map_err(|_| "Invalid item ID".to_string())?;

    security_queries::claim_lost_found_item(iid, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CLAIM_LOST_FOUND", Some("lost_and_found"), Some(iid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn submit_broadcast_request(
    token: String,
    title: String,
    content: String,
    target_audience: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    let id = security_queries::insert_broadcast_request(
        &title,
        &content,
        target_audience.as_deref(),
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_BROADCAST_REQUEST", Some("broadcast_requests"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_broadcast_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = security_queries::get_all_broadcast_requests()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn review_broadcast_request(
    token: String,
    request_id: String,
    status: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "the_guardian" && session.role_name != "the_anchorman" {
        return Err("Only guardian or anchorman can review broadcast requests".to_string());
    }

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    security_queries::update_broadcast_request_review(rid, &status, session.user_id, notes.as_deref())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REVIEW_BROADCAST_REQUEST", Some("broadcast_requests"), Some(rid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

#[tauri::command]
pub async fn submit_security_findings(
    token: String,
    title: String,
    description: Option<String>,
    findings_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let fdate = findings_date.as_deref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let id = security_queries::insert_security_finding(
        &title,
        description.as_deref(),
        fdate,
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_SECURITY_FINDINGS", Some("security_findings"), Some(id), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_security_findings(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = security_queries::get_all_security_findings()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}
