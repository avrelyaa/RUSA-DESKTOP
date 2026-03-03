use crate::auth::{require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct WorkOrderRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    priority: Option<String>,
    system_affected: Option<String>,
    status: Option<String>,
    assigned_to: Option<Uuid>,
    notes: Option<String>,
    created_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct TechnicalReportRow {
    id: Uuid,
    title: String,
    content: Option<String>,
    findings: Option<String>,
    recommendations: Option<String>,
    submitted_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn get_work_orders(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;
    let rows = sqlx::query_as::<_, WorkOrderRow>(
        "SELECT id, title, description, priority, system_affected, status, assigned_to, notes, created_by, created_at FROM aerospace_work_orders WHERE deleted_at IS NULL ORDER BY created_at DESC"
    ).fetch_all(db::get_db()).await.map_err(|e| format!("DB error: {}", e))?;
    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn create_work_order(
    token: String,
    title: String,
    description: Option<String>,
    priority: Option<String>,
    system_affected: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "aerospace_engineer")?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO aerospace_work_orders (title, description, priority, system_affected, status, created_by) VALUES ($1,$2,$3,$4,'open',$5) RETURNING id"
    )
    .bind(&title).bind(&description).bind(&priority).bind(&system_affected).bind(session.user_id)
    .fetch_one(db::get_db()).await.map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_WORK_ORDER", Some("aerospace_work_orders"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn update_work_order_status(
    token: String,
    work_order_id: String,
    status: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "aerospace_engineer")?;

    let wid = Uuid::parse_str(&work_order_id).map_err(|_| "Invalid ID".to_string())?;
    sqlx::query("UPDATE aerospace_work_orders SET status = $1, notes = $2 WHERE id = $3 AND deleted_at IS NULL")
        .bind(&status).bind(&notes).bind(wid)
        .execute(db::get_db()).await.map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_WORK_ORDER_STATUS", Some("aerospace_work_orders"), Some(wid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

#[tauri::command]
pub async fn submit_technical_report(
    token: String,
    title: String,
    content: String,
    findings: Option<String>,
    recommendations: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "aerospace_engineer")?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO aerospace_technical_reports (title, content, findings, recommendations, submitted_by) VALUES ($1,$2,$3,$4,$5) RETURNING id"
    )
    .bind(&title).bind(&content).bind(&findings).bind(&recommendations).bind(session.user_id)
    .fetch_one(db::get_db()).await.map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_TECHNICAL_REPORT", Some("aerospace_technical_reports"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_technical_reports(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;
    let rows = sqlx::query_as::<_, TechnicalReportRow>(
        "SELECT id, title, content, findings, recommendations, submitted_by, created_at FROM aerospace_technical_reports WHERE deleted_at IS NULL ORDER BY created_at DESC"
    ).fetch_all(db::get_db()).await.map_err(|e| format!("DB error: {}", e))?;
    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}
