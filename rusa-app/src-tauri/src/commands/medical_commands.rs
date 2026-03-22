use crate::auth::{require_role_name, validate_session_command};
use crate::queries::auth::write_audit_log;
use crate::queries::budget as budget_queries;
use crate::queries::medical as medical_queries;
use uuid::Uuid;

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

    let id = medical_queries::insert_medical_shift(sid, sstart, send, notes.as_deref(), session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ALLOCATE_SHIFT", Some("medical_shifts"), Some(id), None, Some(serde_json::json!({ "staff_id": staff_id }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_shifts(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if session.role_name == "head_of_medicine" {
        medical_queries::get_all_medical_shifts().await
    } else {
        medical_queries::get_user_medical_shifts(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn get_medical_inventory(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = medical_queries::get_all_medical_inventory()
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

    let id = medical_queries::insert_medical_inventory(
        &item_name,
        category.as_deref(),
        quantity,
        unit.as_deref(),
        edate,
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_MEDICAL_INVENTORY", Some("medical_inventory"), Some(id), None, Some(serde_json::json!({ "item_name": item_name, "quantity": quantity }))).await;
    Ok(id.to_string())
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

    let id = medical_queries::insert_patient_record(
        pid,
        diagnosis.as_deref(),
        treatment.as_deref(),
        medications.as_deref(),
        notes.as_deref(),
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_PATIENT_RECORD", Some("patient_records"), Some(id), None, None).await;
    Ok(id.to_string())
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

    let rows = medical_queries::get_patient_records(pid)
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
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());

    let id = medical_queries::insert_staff_specialization(sid, &specialization, cdate)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ADD_SPECIALIZATION", Some("staff_specializations"), Some(id), None, Some(serde_json::json!({ "staff_id": staff_id, "specialization": specialization }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_staff_specializations(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = medical_queries::get_all_staff_specializations()
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

    let id = budget_queries::insert_budget_request(
        &title,
        &description,
        amount,
        items.as_deref(),
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_BUDGET_REQUEST", Some("budget_requests"), Some(id), None, Some(serde_json::json!({ "title": title, "amount": amount }))).await;
    Ok(id.to_string())
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

    let id = budget_queries::insert_expenditure_report(
        &title,
        &description,
        total_amount,
        items.as_deref(),
        invoice_data.as_deref(),
        session.user_id,
    )
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_EXPENDITURE_REPORT", Some("expenditure_reports"), Some(id), None, Some(serde_json::json!({ "title": title, "total_amount": total_amount }))).await;
    Ok(id.to_string())
}
