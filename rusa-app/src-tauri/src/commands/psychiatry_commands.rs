use crate::auth::{require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct PatientRow {
    id: Uuid,
    patient_id: Uuid,
    registered_by: Option<Uuid>,
    status: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct AppointmentRow {
    id: Uuid,
    patient_id: Option<Uuid>,
    scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    status: Option<String>,
    findings: Option<String>,
    psychiatrist_id: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct RecoveryLogRow {
    id: Uuid,
    patient_id: Option<Uuid>,
    entry_date: Option<chrono::DateTime<chrono::Utc>>,
    status: Option<String>,
    notes: Option<String>,
    logged_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn register_patient(
    token: String,
    patient_id: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_psychiatrist")?;

    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO psychiatric_patients (patient_id, registered_by, status) VALUES ($1,$2,'active') RETURNING id"
    )
    .bind(pid).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "REGISTER_PATIENT", Some("psychiatric_patients"), Some(row.0), None, Some(serde_json::json!({ "patient_id": patient_id }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_patients(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "the_psychiatrist" && session.role_name != "psychiatrist_assistant" {
        return Err("Only psychiatrist or assistant can view patients".to_string());
    }

    let rows = sqlx::query_as::<_, PatientRow>(
        "SELECT id, patient_id, registered_by, status, created_at FROM psychiatric_patients WHERE deleted_at IS NULL ORDER BY created_at DESC"
    )
    .fetch_all(db::get_db())
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
    require_role_name(&session, "the_psychiatrist")?;

    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;
    let sched = chrono::DateTime::parse_from_rfc3339(&scheduled_at)
        .map(|d| d.with_timezone(&chrono::Utc))
        .map_err(|_| "Invalid scheduled_at format".to_string())?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO psychiatric_appointments (patient_id, scheduled_at, status, psychiatrist_id) VALUES ($1,$2,'scheduled',$3) RETURNING id"
    )
    .bind(pid).bind(sched).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SCHEDULE_APPOINTMENT", Some("psychiatric_appointments"), Some(row.0), None, Some(serde_json::json!({ "patient_id": patient_id }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_appointments(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = if session.role_name == "the_psychiatrist" || session.role_name == "psychiatrist_assistant" {
        sqlx::query_as::<_, AppointmentRow>(
            "SELECT id, patient_id, scheduled_at, status, findings, psychiatrist_id, created_at FROM psychiatric_appointments WHERE deleted_at IS NULL ORDER BY scheduled_at DESC"
        ).fetch_all(db::get_db()).await
    } else {
        sqlx::query_as::<_, AppointmentRow>(
            "SELECT id, patient_id, scheduled_at, status, findings, psychiatrist_id, created_at FROM psychiatric_appointments WHERE patient_id = $1 AND deleted_at IS NULL ORDER BY scheduled_at DESC"
        ).bind(session.user_id).fetch_all(db::get_db()).await
    }.map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn complete_appointment(
    token: String,
    appointment_id: String,
    findings: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_psychiatrist")?;

    let aid = Uuid::parse_str(&appointment_id).map_err(|_| "Invalid appointment ID".to_string())?;

    sqlx::query("UPDATE psychiatric_appointments SET status = 'completed', findings = $1 WHERE id = $2 AND psychiatrist_id = $3 AND deleted_at IS NULL")
        .bind(&findings).bind(aid).bind(session.user_id)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "COMPLETE_APPOINTMENT", Some("psychiatric_appointments"), Some(aid), None, None).await;
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
    require_role_name(&session, "the_psychiatrist")?;

    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;
    let edate = chrono::DateTime::parse_from_rfc3339(&entry_date)
        .map(|d| d.with_timezone(&chrono::Utc))
        .map_err(|_| "Invalid entry_date format".to_string())?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO recovery_logs (patient_id, entry_date, status, notes, logged_by) VALUES ($1,$2,$3,$4,$5) RETURNING id"
    )
    .bind(pid).bind(edate).bind(&status).bind(&notes).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "ADD_RECOVERY_LOG", Some("recovery_logs"), Some(row.0), None, None).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_recovery_log(
    token: String,
    patient_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "the_psychiatrist" && session.role_name != "psychiatrist_assistant" {
        return Err("Only psychiatrist or assistant can view recovery logs".to_string());
    }

    let pid = Uuid::parse_str(&patient_id).map_err(|_| "Invalid patient ID".to_string())?;

    let rows = sqlx::query_as::<_, RecoveryLogRow>(
        "SELECT id, patient_id, entry_date, status, notes, logged_by, created_at FROM recovery_logs WHERE patient_id = $1 ORDER BY entry_date DESC"
    )
    .bind(pid)
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}
