use crate::auth::{require_role_name, validate_session_command};
use crate::queries::auth::write_audit_log;
use crate::queries::medical as medical_queries;
use uuid::Uuid;

#[tauri::command]
pub async fn register_patient(
    token: String,
    patient_id: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "psychiatrist")?;

    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;

    let id = medical_queries::insert_psychiatric_patient(pid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REGISTER_PATIENT", Some("psychiatric_patients"), Some(id), None, Some(serde_json::json!({ "patient_id": patient_id }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_patients(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "psychiatrist" && session.role_name != "psychiatrist_assistant" {
        return Err("Only psychiatrist or assistant can view patients".to_string());
    }

    let rows = medical_queries::get_all_psychiatric_patients()
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn schedule_appointment(
    token: String,
    patient_id: String,
    scheduled_at: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "psychiatrist")?;

    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;
    let sched = chrono::DateTime::parse_from_rfc3339(&scheduled_at)
        .map(|d| d.with_timezone(&chrono::Utc))
        .map_err(|_| "Invalid scheduled_at format".to_string())?;

    let id = medical_queries::insert_appointment(pid, sched, session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SCHEDULE_APPOINTMENT", Some("appointments"), Some(id), None, Some(serde_json::json!({ "patient_id": patient_id }))).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_appointments(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if session.role_name == "psychiatrist" || session.role_name == "psychiatrist_assistant" {
        medical_queries::get_all_appointments().await
    } else {
        medical_queries::get_patient_appointments(session.user_id).await
    }
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn complete_appointment(
    token: String,
    appointment_id: String,
    findings: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "psychiatrist")?;

    let aid = Uuid::parse_str(&appointment_id).map_err(|_| "Invalid appointment ID".to_string())?;

    medical_queries::complete_appointment(aid, findings.as_deref(), session.user_id)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "COMPLETE_APPOINTMENT", Some("appointments"), Some(aid), None, None).await;
    Ok(())
}

#[tauri::command]
pub async fn add_recovery_log(
    token: String,
    patient_id: String,
    entry_date: String,
    status: String,
    notes: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "psychiatrist")?;

    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;
    let edate = chrono::NaiveDate::parse_from_str(&entry_date, "%Y-%m-%d")
        .map_err(|_| "Invalid entry_date format (expected YYYY-MM-DD)".to_string())?;

    let id = medical_queries::insert_recovery_log(pid, session.user_id, edate, &status, notes.as_deref())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ADD_RECOVERY_LOG", Some("patient_recovery_log"), Some(id), None, None).await;
    Ok(id.to_string())
}

#[tauri::command]
pub async fn get_recovery_log(
    token: String,
    patient_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "psychiatrist" && session.role_name != "psychiatrist_assistant" {
        return Err("Only psychiatrist or assistant can view recovery logs".to_string());
    }

    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;

    let rows = medical_queries::get_recovery_log_for_patient(pid)
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}
