use crate::auth::{require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct ShiftRow {
    id: Uuid,
    staff_id: Option<Uuid>,
    shift_start: Option<chrono::DateTime<chrono::Utc>>,
    shift_end: Option<chrono::DateTime<chrono::Utc>>,
    notes: Option<String>,
    allocated_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct MedicalInventoryRow {
    id: Uuid,
    item_name: String,
    category: Option<String>,
    quantity: Option<i32>,
    unit: Option<String>,
    expiry_date: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct PatientRecordRow {
    id: Uuid,
    patient_id: Option<Uuid>,
    diagnosis: Option<String>,
    treatment: Option<String>,
    medications: Option<String>,
    notes: Option<String>,
    created_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct SpecializationRow {
    id: Uuid,
    staff_id: Option<Uuid>,
    specialization: String,
    certified_at: Option<chrono::DateTime<chrono::Utc>>,
    added_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct BudgetRequestRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    amount: Option<f64>,
    items: Option<String>,
    status: Option<String>,
    submitted_by: Option<Uuid>,
    reviewed_by: Option<Uuid>,
    notes: Option<String>,
    flagged: Option<bool>,
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
    submitted_by: Option<Uuid>,
    flagged: Option<bool>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn allocate_shift(
    token: String,
    staff_id: String,
    shift_start: String,
    shift_end: String,
    notes: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "head_of_medicine")?;

    let sid = Uuid::parse_str(&staff_id).map_err(|_| "Invalid staff ID".to_string())?;
    let sstart = chrono::DateTime::parse_from_rfc3339(&shift_start)
        .map(|d| d.with_timezone(&chrono::Utc))
        .map_err(|_| "Invalid shift_start format".to_string())?;
    let send = chrono::DateTime::parse_from_rfc3339(&shift_end)
        .map(|d| d.with_timezone(&chrono::Utc))
        .map_err(|_| "Invalid shift_end format".to_string())?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO medical_shifts (staff_id, shift_start, shift_end, notes, allocated_by) VALUES ($1,$2,$3,$4,$5) RETURNING id"
    )
    .bind(sid).bind(sstart).bind(send).bind(&notes).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ALLOCATE_SHIFT", Some("medical_shifts"), Some(row.0), None, Some(serde_json::json!({ "staff_id": staff_id }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_shifts(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if session.role_name == "head_of_medicine" {
        sqlx::query_as::<_, ShiftRow>(
            "SELECT id, staff_id, shift_start, shift_end, notes, allocated_by, created_at FROM medical_shifts ORDER BY shift_start DESC"
        ).fetch_all(db::get_db()).await
    } else {
        sqlx::query_as::<_, ShiftRow>(
            "SELECT id, staff_id, shift_start, shift_end, notes, allocated_by, created_at FROM medical_shifts WHERE staff_id = $1 ORDER BY shift_start DESC"
        ).bind(session.user_id).fetch_all(db::get_db()).await
    }.map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn get_medical_inventory(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, MedicalInventoryRow>(
        "SELECT id, item_name, category, quantity, unit, expiry_date, updated_at FROM medical_inventory WHERE deleted_at IS NULL ORDER BY item_name"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn update_medical_inventory(
    token: String,
    item_name: String,
    category: Option<String>,
    quantity: i32,
    unit: Option<String>,
    expiry_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let edate = expiry_date.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO medical_inventory (item_name, category, quantity, unit, expiry_date, updated_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id"
    )
    .bind(&item_name).bind(&category).bind(quantity).bind(&unit).bind(edate).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_MEDICAL_INVENTORY", Some("medical_inventory"), Some(row.0), None, Some(serde_json::json!({ "item_name": item_name, "quantity": quantity }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn create_patient_record(
    token: String,
    patient_id: String,
    diagnosis: Option<String>,
    treatment: Option<String>,
    medications: Option<String>,
    notes: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO patient_records (patient_id, diagnosis, treatment, medications, notes, created_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id"
    )
    .bind(pid).bind(&diagnosis).bind(&treatment).bind(&medications).bind(&notes).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_PATIENT_RECORD", Some("patient_records"), Some(row.0), None, None).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_patient_records(
    token: String,
    patient_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    // Must be medical staff (subsystem check) or admin
    if session.subsystem.as_deref() != Some("medical") && session.tier < 4 {
        return Err("Only medical staff can view patient records".to_string());
    }

    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;

    let rows = sqlx::query_as::<_, PatientRecordRow>(
        "SELECT id, patient_id, diagnosis, treatment, medications, notes, created_by, created_at FROM patient_records WHERE patient_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
    )
    .bind(pid)
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn add_specialization(
    token: String,
    staff_id: String,
    specialization: String,
    certified_at: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let sid = Uuid::parse_str(&staff_id).map_err(|_| "Invalid staff ID".to_string())?;
    let cdate = certified_at.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO staff_specializations (staff_id, specialization, certified_at, added_by) VALUES ($1,$2,$3,$4) RETURNING id"
    )
    .bind(sid).bind(&specialization).bind(cdate).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ADD_SPECIALIZATION", Some("staff_specializations"), Some(row.0), None, Some(serde_json::json!({ "staff_id": staff_id, "specialization": specialization }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_staff_specializations(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, SpecializationRow>(
        "SELECT id, staff_id, specialization, certified_at, added_by, created_at FROM staff_specializations ORDER BY specialization"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn submit_budget_request(
    token: String,
    title: String,
    description: String,
    amount: f64,
    items: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "head_of_medicine" && session.role_name != "head_of_sanitary" {
        return Err("Only head of medicine or sanitary can submit budget requests".to_string());
    }

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO budget_requests (title, description, amount, items, status, submitted_by) VALUES ($1,$2,$3,$4,'pending',$5) RETURNING id"
    )
    .bind(&title).bind(&description).bind(amount).bind(&items).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_BUDGET_REQUEST", Some("budget_requests"), Some(row.0), None, Some(serde_json::json!({ "title": title, "amount": amount }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn submit_expenditure_report(
    token: String,
    title: String,
    description: String,
    total_amount: f64,
    items: Option<String>,
    invoice_data: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO expenditure_reports (title, description, total_amount, items, invoice_data, status, submitted_by) VALUES ($1,$2,$3,$4,$5,'pending',$6) RETURNING id"
    )
    .bind(&title).bind(&description).bind(total_amount).bind(&items).bind(&invoice_data).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_EXPENDITURE_REPORT", Some("expenditure_reports"), Some(row.0), None, Some(serde_json::json!({ "title": title, "total_amount": total_amount }))).await;
    Ok(row.0.to_string())
}
