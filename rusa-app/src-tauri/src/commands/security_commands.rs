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

// ──────────────── Security Tasks (Guardian) ──────────────────────

use crate::db;

async fn ensure_security_tasks_table() -> Result<(), String> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS security_tasks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title TEXT NOT NULL,
            description TEXT,
            assigned_to UUID REFERENCES users(id),
            assigned_by UUID REFERENCES users(id),
            status TEXT NOT NULL DEFAULT 'pending'
                CHECK (status IN ('pending','in_progress','result_submitted','completed','cancelled')),
            result_notes TEXT,
            priority VARCHAR(50) DEFAULT 'medium' CHECK (priority IN ('low','medium','high','critical')),
            due_date TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"#,
    )
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error creating table: {}", e))?;
    Ok(())
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct SecurityTaskRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    assigned_to: Option<Uuid>,
    assigned_by: Option<Uuid>,
    status: String,
    result_notes: Option<String>,
    priority: Option<String>,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn assign_security_task(
    token: String,
    assigned_to: String,
    title: String,
    description: Option<String>,
    priority: Option<String>,
    due_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "the_guardian" && session.role_name != "the_administrator" {
        return Err("Only the_guardian or the_administrator can assign security tasks".to_string());
    }

    ensure_security_tasks_table().await?;

    let atid = Uuid::parse_str(&assigned_to).map_err(|_| "Invalid user ID".to_string())?;
    let prio = priority.as_deref().unwrap_or("medium");
    let ddate = due_date.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO security_tasks (title, description, assigned_to, assigned_by, status, priority, due_date) VALUES ($1,$2,$3,$4,'pending',$5,$6) RETURNING id"
    )
    .bind(&title).bind(&description).bind(atid).bind(session.user_id).bind(prio).bind(ddate)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ASSIGN_SECURITY_TASK", Some("security_tasks"), Some(row.0), None, Some(serde_json::json!({ "title": title, "assigned_to": assigned_to }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_security_tasks(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    ensure_security_tasks_table().await?;

    let rows = if session.role_name == "the_guardian" || session.role_name == "the_administrator" {
        sqlx::query_as::<_, SecurityTaskRow>(
            "SELECT id, title, description, assigned_to, assigned_by, status, result_notes, priority, due_date, created_at FROM security_tasks WHERE deleted_at IS NULL ORDER BY created_at DESC"
        ).fetch_all(db::get_db()).await
    } else {
        sqlx::query_as::<_, SecurityTaskRow>(
            "SELECT id, title, description, assigned_to, assigned_by, status, result_notes, priority, due_date, created_at FROM security_tasks WHERE assigned_to = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ).bind(session.user_id).fetch_all(db::get_db()).await
    }.map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn update_security_task(
    token: String,
    task_id: String,
    status: String,
    result_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;

    ensure_security_tasks_table().await?;

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    if session.role_name == "the_guardian" || session.role_name == "the_administrator" {
        // Guardian can complete tasks (stage 2 approval)
        sqlx::query("UPDATE security_tasks SET status = $1, result_notes = COALESCE($2, result_notes) WHERE id = $3 AND deleted_at IS NULL")
            .bind(&status).bind(&result_notes).bind(tid)
            .execute(db::get_db())
            .await
            .map_err(|e| format!("DB error: {}", e))?;
    } else {
        // Security team members can submit results for their own assigned tasks
        sqlx::query("UPDATE security_tasks SET status = $1, result_notes = COALESCE($2, result_notes) WHERE id = $3 AND assigned_to = $4 AND deleted_at IS NULL")
            .bind(&status).bind(&result_notes).bind(tid).bind(session.user_id)
            .execute(db::get_db())
            .await
            .map_err(|e| format!("DB error: {}", e))?;
    }

    let _ = write_audit_log(Some(session.user_id), "UPDATE_SECURITY_TASK", Some("security_tasks"), Some(tid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}
