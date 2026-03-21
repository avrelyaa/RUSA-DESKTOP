use crate::auth::{require_role, require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct AerospaceTaskRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    assigned_to: Option<Uuid>,
    assigned_by: Option<Uuid>,
    status: Option<String>,
    progress_notes: Option<String>,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn ensure_aerospace_tasks_table() -> Result<(), String> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS aerospace_assigned_tasks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title TEXT NOT NULL,
            description TEXT,
            assigned_to UUID REFERENCES users(id),
            assigned_by UUID REFERENCES users(id),
            status TEXT NOT NULL DEFAULT 'pending',
            progress_notes TEXT,
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

#[tauri::command]
pub async fn assign_aerospace_task(
    token: String,
    assigned_to: String,
    title: String,
    description: Option<String>,
    due_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role(&session, 2)?;

    ensure_aerospace_tasks_table().await?;

    let atid = Uuid::parse_str(&assigned_to).map_err(|_| "Invalid user ID".to_string())?;
    let dd = due_date
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO aerospace_assigned_tasks (title, description, assigned_to, assigned_by, status, due_date) VALUES ($1,$2,$3,$4,'pending',$5) RETURNING id",
    )
    .bind(&title)
    .bind(&description)
    .bind(atid)
    .bind(session.user_id)
    .bind(dd)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "ASSIGN_AEROSPACE_TASK",
        Some("aerospace_assigned_tasks"),
        Some(row.0),
        None,
        Some(serde_json::json!({ "title": title, "assigned_to": assigned_to })),
    )
    .await;

    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_aerospace_assigned_tasks(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    ensure_aerospace_tasks_table().await?;

    let rows = if session.tier >= 2 {
        sqlx::query_as::<_, AerospaceTaskRow>(
            "SELECT id, title, description, assigned_to, assigned_by, status, progress_notes, due_date, created_at FROM aerospace_assigned_tasks WHERE assigned_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(session.user_id)
        .fetch_all(db::get_db())
        .await
    } else {
        sqlx::query_as::<_, AerospaceTaskRow>(
            "SELECT id, title, description, assigned_to, assigned_by, status, progress_notes, due_date, created_at FROM aerospace_assigned_tasks WHERE assigned_to = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(session.user_id)
        .fetch_all(db::get_db())
        .await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or_default())
        .collect())
}

#[tauri::command]
pub async fn update_aerospace_task_status(
    token: String,
    task_id: String,
    status: String,
    progress_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;

    ensure_aerospace_tasks_table().await?;

    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    sqlx::query(
        "UPDATE aerospace_assigned_tasks SET status = $1, progress_notes = COALESCE($2, progress_notes) WHERE id = $3 AND (assigned_to = $4 OR $5 >= 2) AND deleted_at IS NULL",
    )
    .bind(&status)
    .bind(&progress_notes)
    .bind(tid)
    .bind(session.user_id)
    .bind(session.tier)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "UPDATE_AEROSPACE_TASK_STATUS",
        Some("aerospace_assigned_tasks"),
        Some(tid),
        None,
        Some(serde_json::json!({ "status": status })),
    )
    .await;

    Ok(())
}

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
