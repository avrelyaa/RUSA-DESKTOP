use crate::auth::{require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct BudgetRequestRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    amount: Option<f64>,
    items: Option<String>,
    status: Option<String>,
    requested_by: Option<Uuid>,
    accountant_notes: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct ExpenditureReportRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    total_amount: Option<f64>,
    items: Option<String>,
    invoice_data: Option<String>,
    status: Option<String>,
    reported_by: Option<Uuid>,
    is_flagged: Option<bool>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct InvestigationRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    related_report_id: Option<Uuid>,
    status: Option<String>,
    submitted_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn get_budget_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if session.role_name == "the_accountant" {
        sqlx::query_as::<_, BudgetRequestRow>(
            "SELECT id, title, description, amount, items, status, requested_by, accountant_notes, created_at FROM budget_requests WHERE deleted_at IS NULL ORDER BY created_at DESC"
        ).fetch_all(db::get_db()).await
    } else {
        sqlx::query_as::<_, BudgetRequestRow>(
            "SELECT id, title, description, amount, items, status, requested_by, accountant_notes, created_at FROM budget_requests WHERE requested_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ).bind(session.user_id).fetch_all(db::get_db()).await
    }.map_err(|e| format!("DB error: {}", e))?;

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

    // Accountant may only move to under_review; approved/rejected come from Governance votes
    if status == "approved" || status == "rejected" {
        return Err("Budget requests can only be approved or rejected through a Governance vote by the Directors".to_string());
    }

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    sqlx::query("UPDATE budget_requests SET status = $1, accountant_notes = $2 WHERE id = $3 AND deleted_at IS NULL")
        .bind(&status).bind(&notes).bind(rid)
        .execute(db::get_db())
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

    sqlx::query("UPDATE budget_requests SET status = 'flagged', accountant_notes = $1 WHERE id = $2 AND deleted_at IS NULL")
        .bind(&reason).bind(rid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "FLAG_BUDGET_REQUEST", Some("budget_requests"), Some(rid), None, Some(serde_json::json!({ "reason": reason }))).await;
    Ok(())
}

#[tauri::command]
pub async fn get_expenditure_reports(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if session.role_name == "the_accountant" {
        sqlx::query_as::<_, ExpenditureReportRow>(
            "SELECT id, title, description, total_amount, items, invoice_data, status, reported_by, is_flagged, created_at FROM expenditure_reports WHERE deleted_at IS NULL ORDER BY created_at DESC"
        ).fetch_all(db::get_db()).await
    } else {
        sqlx::query_as::<_, ExpenditureReportRow>(
            "SELECT id, title, description, total_amount, items, invoice_data, status, reported_by, is_flagged, created_at FROM expenditure_reports WHERE reported_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ).bind(session.user_id).fetch_all(db::get_db()).await
    }.map_err(|e| format!("DB error: {}", e))?;

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

    sqlx::query("UPDATE expenditure_reports SET is_flagged = true, flag_reason = $1, status = 'flagged_for_investigation' WHERE id = $2 AND deleted_at IS NULL")
        .bind(&reason).bind(rid)
        .execute(db::get_db())
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

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO budget_investigations (title, description, related_report_id, status, submitted_by) VALUES ($1,$2,$3,'open',$4) RETURNING id"
    )
    .bind(&title).bind(&description).bind(rid).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_INVESTIGATION", Some("budget_investigations"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}
