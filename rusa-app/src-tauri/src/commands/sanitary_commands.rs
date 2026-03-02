use crate::auth::{require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct SanitaryTaskRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    division: Option<String>,
    status: Option<String>,
    assigned_to: Option<Uuid>,
    assigned_by: Option<Uuid>,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct SanitaryInventoryRow {
    id: Uuid,
    item_name: String,
    category: Option<String>,
    quantity: Option<i32>,
    unit: Option<String>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct DisposalLogRow {
    id: Uuid,
    item_name: String,
    quantity: Option<f64>,
    unit: Option<String>,
    disposal_method: Option<String>,
    hazard_level: Option<String>,
    notes: Option<String>,
    logged_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct WastewaterLogRow {
    id: Uuid,
    volume_treated: Option<f64>,
    unit: Option<String>,
    treatment_method: Option<String>,
    ph_level: Option<f64>,
    quality_notes: Option<String>,
    logged_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct DivisionTransferRow {
    id: Uuid,
    from_division: String,
    to_division: String,
    reason: Option<String>,
    status: Option<String>,
    requested_by: Option<Uuid>,
    reviewed_by: Option<Uuid>,
    notes: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct InspectionReportRow {
    id: Uuid,
    location: String,
    inspection_date: Option<chrono::DateTime<chrono::Utc>>,
    findings: Option<String>,
    violations: Option<String>,
    recommendations: Option<String>,
    inspector_id: Option<Uuid>,
    sent_to_head: Option<bool>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn get_sanitary_tasks(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if session.role_name == "head_of_sanitary" {
        sqlx::query_as::<_, SanitaryTaskRow>(
            "SELECT id, title, description, division, status, assigned_to, assigned_by, due_date, created_at FROM sanitary_tasks WHERE deleted_at IS NULL ORDER BY created_at DESC"
        ).fetch_all(db::get_db()).await
    } else {
        sqlx::query_as::<_, SanitaryTaskRow>(
            "SELECT id, title, description, division, status, assigned_to, assigned_by, due_date, created_at FROM sanitary_tasks WHERE assigned_to = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        ).bind(session.user_id).fetch_all(db::get_db()).await
    }.map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn assign_sanitary_task(
    token: String,
    assigned_to: String,
    title: String,
    description: Option<String>,
    division: Option<String>,
    due_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "head_of_sanitary")?;

    let atid = Uuid::parse_str(&assigned_to).map_err(|_| "Invalid user ID".to_string())?;
    let ddate = due_date.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO sanitary_tasks (title, description, division, status, assigned_to, assigned_by, due_date) VALUES ($1,$2,$3,'pending',$4,$5,$6) RETURNING id"
    )
    .bind(&title).bind(&description).bind(&division).bind(atid).bind(session.user_id).bind(ddate)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ASSIGN_SANITARY_TASK", Some("sanitary_tasks"), Some(row.0), None, Some(serde_json::json!({ "title": title, "assigned_to": assigned_to }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn update_sanitary_task(
    token: String,
    task_id: String,
    status: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    let tid = Uuid::parse_str(&task_id).map_err(|_| "Invalid task ID".to_string())?;

    sqlx::query("UPDATE sanitary_tasks SET status = $1 WHERE id = $2 AND (assigned_to = $3 OR $4 = 'head_of_sanitary') AND deleted_at IS NULL")
        .bind(&status).bind(tid).bind(session.user_id).bind(&session.role_name)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_SANITARY_TASK", Some("sanitary_tasks"), Some(tid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

#[tauri::command]
pub async fn get_sanitary_inventory(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, SanitaryInventoryRow>(
        "SELECT id, item_name, category, quantity, unit, updated_at FROM sanitary_inventory WHERE deleted_at IS NULL ORDER BY item_name"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn update_sanitary_inventory(
    token: String,
    item_name: String,
    category: Option<String>,
    quantity: i32,
    unit: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO sanitary_inventory (item_name, category, quantity, unit, updated_by) VALUES ($1,$2,$3,$4,$5) RETURNING id"
    )
    .bind(&item_name).bind(&category).bind(quantity).bind(&unit).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_SANITARY_INVENTORY", Some("sanitary_inventory"), Some(row.0), None, Some(serde_json::json!({ "item_name": item_name, "quantity": quantity }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn add_disposal_log(
    token: String,
    item_name: String,
    quantity: f64,
    unit: Option<String>,
    disposal_method: Option<String>,
    hazard_level: Option<String>,
    notes: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "disposal_crew")?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO disposal_logs (item_name, quantity, unit, disposal_method, hazard_level, notes, logged_by) VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id"
    )
    .bind(&item_name).bind(quantity).bind(&unit).bind(&disposal_method).bind(&hazard_level).bind(&notes).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ADD_DISPOSAL_LOG", Some("disposal_logs"), Some(row.0), None, Some(serde_json::json!({ "item_name": item_name, "quantity": quantity }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_disposal_logs(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, DisposalLogRow>(
        "SELECT id, item_name, quantity, unit, disposal_method, hazard_level, notes, logged_by, created_at FROM disposal_logs ORDER BY created_at DESC"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn add_wastewater_log(
    token: String,
    volume_treated: f64,
    unit: Option<String>,
    treatment_method: Option<String>,
    ph_level: Option<f64>,
    quality_notes: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "wastewater_crew")?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO wastewater_logs (volume_treated, unit, treatment_method, ph_level, quality_notes, logged_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id"
    )
    .bind(volume_treated).bind(&unit).bind(&treatment_method).bind(ph_level).bind(&quality_notes).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ADD_WASTEWATER_LOG", Some("wastewater_logs"), Some(row.0), None, None).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_wastewater_logs(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, WastewaterLogRow>(
        "SELECT id, volume_treated, unit, treatment_method, ph_level, quality_notes, logged_by, created_at FROM wastewater_logs ORDER BY created_at DESC"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn submit_division_transfer(
    token: String,
    from_division: String,
    to_division: String,
    reason: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO division_transfers (from_division, to_division, reason, status, requested_by) VALUES ($1,$2,$3,'pending',$4) RETURNING id"
    )
    .bind(&from_division).bind(&to_division).bind(&reason).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_DIVISION_TRANSFER", Some("division_transfers"), Some(row.0), None, Some(serde_json::json!({ "from": from_division, "to": to_division }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn review_division_transfer(
    token: String,
    request_id: String,
    decision: String,
    notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "head_of_sanitary")?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    sqlx::query("UPDATE division_transfers SET status = $1, reviewed_by = $2, notes = $3 WHERE id = $4")
        .bind(&decision).bind(session.user_id).bind(&notes).bind(rid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REVIEW_DIVISION_TRANSFER", Some("division_transfers"), Some(rid), None, Some(serde_json::json!({ "decision": decision }))).await;
    Ok(())
}

#[tauri::command]
pub async fn set_division_quota(
    token: String,
    division: String,
    quota_type: String,
    target_value: i32,
    period: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "head_of_sanitary")?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO division_quotas (division, quota_type, target_value, period, set_by) VALUES ($1,$2,$3,$4,$5) RETURNING id"
    )
    .bind(&division).bind(&quota_type).bind(target_value).bind(&period).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SET_DIVISION_QUOTA", Some("division_quotas"), Some(row.0), None, Some(serde_json::json!({ "division": division, "quota_type": quota_type, "target_value": target_value }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn create_inspection_report(
    token: String,
    location: String,
    inspection_date: String,
    findings: String,
    violations: Option<String>,
    recommendations: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "sanitary_inspector")?;

    let idate = chrono::DateTime::parse_from_rfc3339(&inspection_date)
        .map(|d| d.with_timezone(&chrono::Utc))
        .map_err(|_| "Invalid inspection_date format".to_string())?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO inspection_reports (location, inspection_date, findings, violations, recommendations, inspector_id) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id"
    )
    .bind(&location).bind(idate).bind(&findings).bind(&violations).bind(&recommendations).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_INSPECTION_REPORT", Some("inspection_reports"), Some(row.0), None, Some(serde_json::json!({ "location": location }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_inspection_reports(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, InspectionReportRow>(
        "SELECT id, location, inspection_date, findings, violations, recommendations, inspector_id, sent_to_head, created_at FROM inspection_reports WHERE deleted_at IS NULL ORDER BY inspection_date DESC"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn send_inspection_to_head(
    token: String,
    report_id: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "sanitary_inspector")?;

    let rid = Uuid::parse_str(&report_id).map_err(|_| "Invalid report ID".to_string())?;

    sqlx::query("UPDATE inspection_reports SET sent_to_head = true WHERE id = $1 AND inspector_id = $2")
        .bind(rid).bind(session.user_id)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SEND_INSPECTION_TO_HEAD", Some("inspection_reports"), Some(rid), None, None).await;
    Ok(())
}
