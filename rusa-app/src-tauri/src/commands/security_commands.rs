use crate::auth::{require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct IncidentReportRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    location: Option<String>,
    incident_date: Option<chrono::DateTime<chrono::Utc>>,
    severity: Option<String>,
    status: Option<String>,
    reported_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct LostFoundRow {
    id: Uuid,
    item_name: String,
    description: Option<String>,
    found_location: Option<String>,
    found_date: Option<chrono::NaiveDate>,
    status: Option<String>,
    logged_by: Option<Uuid>,
    claimed_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct BroadcastRequestRow {
    id: Uuid,
    title: String,
    content: Option<String>,
    target_audience: Option<String>,
    status: Option<String>,
    requested_by: Option<Uuid>,
    reviewed_by: Option<Uuid>,
    review_notes: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct SecurityFindingRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    findings_date: Option<chrono::NaiveDate>,
    reported_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

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

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO incident_reports (title, description, location, incident_date, severity, status, reported_by) VALUES ($1,$2,$3,$4,$5,'open',$6) RETURNING id"
    )
    .bind(&title).bind(&description).bind(&location).bind(idate).bind(&severity).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_INCIDENT_REPORT", Some("incident_reports"), Some(row.0), None, Some(serde_json::json!({ "title": title, "severity": severity }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_incident_reports(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, IncidentReportRow>(
        "SELECT id, title, description, location, incident_date, severity, status, reported_by, created_at FROM incident_reports WHERE deleted_at IS NULL ORDER BY created_at DESC"
    )
    .fetch_all(db::get_db())
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

    sqlx::query("UPDATE incident_reports SET status = $1 WHERE id = $2 AND deleted_at IS NULL")
        .bind(&status).bind(rid)
        .execute(db::get_db())
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

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO lost_and_found (item_name, description, found_location, found_date, status, logged_by) VALUES ($1,$2,$3,$4,'unclaimed',$5) RETURNING id"
    )
    .bind(&item_name).bind(&description).bind(&found_location).bind(fdate).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ADD_LOST_FOUND", Some("lost_and_found"), Some(row.0), None, Some(serde_json::json!({ "item_name": item_name }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_lost_found(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, LostFoundRow>(
        "SELECT id, item_name, description, found_location, found_date, status, logged_by, claimed_by, created_at FROM lost_and_found WHERE deleted_at IS NULL ORDER BY created_at DESC"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn claim_lost_found(token: String, item_id: String) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    let iid = Uuid::parse_str(&item_id).map_err(|_| "Invalid item ID".to_string())?;

    sqlx::query("UPDATE lost_and_found SET status = 'claimed', claimed_by = $1 WHERE id = $2 AND deleted_at IS NULL AND status = 'unclaimed'")
        .bind(session.user_id).bind(iid)
        .execute(db::get_db())
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

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO broadcast_requests (title, content, target_audience, status, requested_by) VALUES ($1,$2,$3,'pending',$4) RETURNING id"
    )
    .bind(&title).bind(&content).bind(&target_audience).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_BROADCAST_REQUEST", Some("broadcast_requests"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_broadcast_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, BroadcastRequestRow>(
        "SELECT id, title, content, target_audience, status, requested_by, reviewed_by, review_notes, created_at FROM broadcast_requests WHERE deleted_at IS NULL ORDER BY created_at DESC"
    )
    .fetch_all(db::get_db())
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

    sqlx::query("UPDATE broadcast_requests SET status = $1, reviewed_by = $2, review_notes = $3 WHERE id = $4 AND deleted_at IS NULL")
        .bind(&status).bind(session.user_id).bind(&notes).bind(rid)
        .execute(db::get_db())
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

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO security_findings (title, description, findings_date, reported_by) VALUES ($1,$2,$3,$4) RETURNING id"
    )
    .bind(&title).bind(&description).bind(fdate).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_SECURITY_FINDINGS", Some("security_findings"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_security_findings(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, SecurityFindingRow>(
        "SELECT id, title, description, findings_date, reported_by, created_at FROM security_findings WHERE deleted_at IS NULL ORDER BY created_at DESC"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}
