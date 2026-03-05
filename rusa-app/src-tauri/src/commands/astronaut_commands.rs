use crate::auth::{require_role_name, validate_session_command};
use crate::db;
use crate::queries::auth::write_audit_log;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
struct MissionConclusionRequestRow {
    id: Uuid,
    mission_id: Uuid,
    submitted_by: Option<Uuid>,
    report_summary: Option<String>,
    status: String,
    reviewed_by: Option<Uuid>,
    review_notes: Option<String>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn ensure_conclusion_requests_table() -> Result<(), String> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS mission_conclusion_requests (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            mission_id UUID NOT NULL REFERENCES missions(id),
            submitted_by UUID REFERENCES users(id),
            report_summary TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            reviewed_by UUID REFERENCES users(id),
            review_notes TEXT,
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
struct MissionRow {
    id: Uuid,
    title: String,
    description: Option<String>,
    mission_type: Option<String>,
    status: Option<String>,
    ship_id: Option<Uuid>,
    sector_id: Option<Uuid>,
    planet_id: Option<Uuid>,
    start_date: Option<chrono::DateTime<chrono::Utc>>,
    created_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct MissionReportRow {
    id: Uuid,
    mission_id: Uuid,
    report_type: Option<String>,
    content: Option<String>,
    submitted_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct JournalRow {
    id: Uuid,
    mission_id: Option<Uuid>,
    title: String,
    content: Option<String>,
    is_public: Option<bool>,
    author_id: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct SectorRow {
    id: Uuid,
    name: String,
    description: Option<String>,
    boundaries: Option<String>,
    created_by: Option<Uuid>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
struct ShipRow {
    id: Uuid,
    name: String,
    ship_type: Option<String>,
    status: Option<String>,
    capacity: Option<i32>,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[tauri::command]
pub async fn get_missions(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, MissionRow>(
        "SELECT id, title, description, mission_type, status, ship_id, sector_id, planet_id, start_date, created_by, created_at FROM missions WHERE deleted_at IS NULL ORDER BY created_at DESC"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn create_mission(
    token: String,
    title: String,
    description: Option<String>,
    mission_type: String,
    ship_id: Option<String>,
    sector_id: Option<String>,
    planet_id: Option<String>,
    start_date: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_wanderer")?;

    let sid = ship_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let secid = sector_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let pid = planet_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());
    let sdate = start_date.as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|d| d.with_timezone(&chrono::Utc));

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO missions (title, description, mission_type, status, ship_id, sector_id, planet_id, start_date, created_by) VALUES ($1,$2,$3,'planned',$4,$5,$6,$7,$8) RETURNING id"
    )
    .bind(&title).bind(&description).bind(&mission_type)
    .bind(sid).bind(secid).bind(pid).bind(sdate).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_MISSION", Some("missions"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn update_mission_status(
    token: String,
    mission_id: String,
    status: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "the_wanderer" && session.role_name != "the_taskmaster" {
        return Err("Only wanderer or taskmaster can update mission status".to_string());
    }

    if status == "completed" {
        return Err("Use the conclusion request workflow to mark missions as completed".to_string());
    }

    let mid = Uuid::parse_str(&mission_id).map_err(|_| "Invalid mission ID".to_string())?;

    sqlx::query("UPDATE missions SET status = $1 WHERE id = $2 AND deleted_at IS NULL")
        .bind(&status).bind(mid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "UPDATE_MISSION_STATUS", Some("missions"), Some(mid), None, Some(serde_json::json!({ "status": status }))).await;
    Ok(())
}

#[tauri::command]
pub async fn assign_crew(
    token: String,
    mission_id: String,
    astronaut_ids: Vec<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_wanderer")?;

    let mid = Uuid::parse_str(&mission_id).map_err(|_| "Invalid mission ID".to_string())?;

    for aid_str in &astronaut_ids {
        let aid = Uuid::parse_str(aid_str).map_err(|_| "Invalid astronaut ID".to_string())?;
        sqlx::query(
            "INSERT INTO mission_crew (mission_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        )
        .bind(mid).bind(aid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;
    }

    let _ = write_audit_log(Some(session.user_id), "ASSIGN_CREW", Some("mission_crew"), Some(mid), None, Some(serde_json::json!({ "mission_id": mission_id, "count": astronaut_ids.len() }))).await;
    Ok(())
}

#[tauri::command]
pub async fn submit_mission_report(
    token: String,
    mission_id: String,
    report_type: String,
    content: String,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let mid = Uuid::parse_str(&mission_id).map_err(|_| "Invalid mission ID".to_string())?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO mission_reports (mission_id, report_type, content, submitted_by) VALUES ($1,$2,$3,$4) RETURNING id"
    )
    .bind(mid).bind(&report_type).bind(&content).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "SUBMIT_MISSION_REPORT", Some("mission_reports"), Some(row.0), None, None).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_mission_reports(
    token: String,
    mission_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;
    let mid = Uuid::parse_str(&mission_id).map_err(|_| "Invalid mission ID".to_string())?;

    let rows = sqlx::query_as::<_, MissionReportRow>(
        "SELECT id, mission_id, report_type, content, submitted_by, created_at FROM mission_reports WHERE mission_id = $1 ORDER BY created_at DESC"
    )
    .bind(mid)
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn create_exploration_journal(
    token: String,
    mission_id: Option<String>,
    title: String,
    content: String,
    is_public: bool,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    let mid = mission_id.as_deref().and_then(|s| Uuid::parse_str(s).ok());

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO exploration_journals (mission_id, title, content, is_public, author_id) VALUES ($1,$2,$3,$4,$5) RETURNING id"
    )
    .bind(mid).bind(&title).bind(&content).bind(is_public).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_JOURNAL", Some("exploration_journals"), Some(row.0), None, Some(serde_json::json!({ "title": title }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_journals(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, JournalRow>(
        "SELECT id, mission_id, title, content, is_public, author_id, created_at FROM exploration_journals WHERE (is_public = true OR author_id = $1) AND deleted_at IS NULL ORDER BY created_at DESC"
    )
    .bind(session.user_id)
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn get_sectors(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, SectorRow>(
        "SELECT id, name, description, boundaries, created_by, created_at FROM sectors WHERE deleted_at IS NULL ORDER BY name"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn create_sector(
    token: String,
    name: String,
    description: Option<String>,
    boundaries: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_wanderer")?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO sectors (name, description, boundaries, created_by) VALUES ($1,$2,$3,$4) RETURNING id"
    )
    .bind(&name).bind(&description).bind(&boundaries).bind(session.user_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "CREATE_SECTOR", Some("sectors"), Some(row.0), None, Some(serde_json::json!({ "name": name }))).await;
    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn rename_sector(
    token: String,
    sector_id: String,
    new_name: String,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    require_role_name(&session, "the_wanderer")?;

    let sid = Uuid::parse_str(&sector_id).map_err(|_| "Invalid sector ID".to_string())?;

    sqlx::query("UPDATE sectors SET name = $1 WHERE id = $2 AND deleted_at IS NULL")
        .bind(&new_name).bind(sid)
        .execute(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(Some(session.user_id), "RENAME_SECTOR", Some("sectors"), Some(sid), None, Some(serde_json::json!({ "new_name": new_name }))).await;
    Ok(())
}

#[tauri::command]
pub async fn get_ships(token: String) -> Result<Vec<serde_json::Value>, String> {
    let _session = validate_session_command(&token).await?;

    let rows = sqlx::query_as::<_, ShipRow>(
        "SELECT id, name, ship_type, status, capacity, created_at FROM ships WHERE deleted_at IS NULL ORDER BY name"
    )
    .fetch_all(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(rows.into_iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect())
}

#[tauri::command]
pub async fn get_astronaut_stats(
    token: String,
    astronaut_id: Option<String>,
) -> Result<serde_json::Value, String> {
    let session = validate_session_command(&token).await?;
    let target_id = astronaut_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(session.user_id);

    #[derive(sqlx::FromRow, serde::Serialize)]
    struct AstronautStats {
        total_missions: Option<i64>,
        completed_missions: Option<i64>,
        total_reports: Option<i64>,
        total_journals: Option<i64>,
    }

    let stats = sqlx::query_as::<_, AstronautStats>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM mission_crew WHERE user_id = $1) as total_missions,
            (SELECT COUNT(*) FROM mission_crew mc JOIN missions m ON mc.mission_id = m.id WHERE mc.user_id = $1 AND m.status = 'completed') as completed_missions,
            (SELECT COUNT(*) FROM mission_reports WHERE submitted_by = $1) as total_reports,
            (SELECT COUNT(*) FROM exploration_journals WHERE author_id = $1 AND deleted_at IS NULL) as total_journals
        "#
    )
    .bind(target_id)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    Ok(serde_json::to_value(stats).unwrap_or_default())
}

#[tauri::command]
pub async fn submit_conclusion_request(
    token: String,
    mission_id: String,
    report_summary: Option<String>,
) -> Result<String, String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "astronaut" {
        return Err("Only astronauts can submit conclusion requests".to_string());
    }

    let summary = report_summary
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "Report summary is required".to_string())?;

    ensure_conclusion_requests_table().await?;

    let mid = Uuid::parse_str(&mission_id).map_err(|_| "Invalid mission ID".to_string())?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO mission_conclusion_requests (mission_id, submitted_by, report_summary, status) VALUES ($1,$2,$3,'pending') RETURNING id"
    )
    .bind(mid)
    .bind(session.user_id)
    .bind(&summary)
    .fetch_one(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let _ = write_audit_log(
        Some(session.user_id),
        "SUBMIT_CONCLUSION_REQUEST",
        Some("mission_conclusion_requests"),
        Some(row.0),
        None,
        Some(serde_json::json!({ "mission_id": mission_id })),
    )
    .await;

    Ok(row.0.to_string())
}

#[tauri::command]
pub async fn get_conclusion_requests(token: String) -> Result<Vec<serde_json::Value>, String> {
    let session = validate_session_command(&token).await?;

    ensure_conclusion_requests_table().await?;

    let rows = if session.role_name == "the_wanderer" || session.role_name == "the_taskmaster" {
        sqlx::query_as::<_, MissionConclusionRequestRow>(
            "SELECT id, mission_id, submitted_by, report_summary, status, reviewed_by, review_notes, created_at FROM mission_conclusion_requests WHERE deleted_at IS NULL ORDER BY created_at DESC"
        )
        .fetch_all(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?
    } else {
        sqlx::query_as::<_, MissionConclusionRequestRow>(
            "SELECT id, mission_id, submitted_by, report_summary, status, reviewed_by, review_notes, created_at FROM mission_conclusion_requests WHERE submitted_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC"
        )
        .bind(session.user_id)
        .fetch_all(db::get_db())
        .await
        .map_err(|e| format!("DB error: {}", e))?
    };

    Ok(rows
        .into_iter()
        .map(|r| serde_json::to_value(r).unwrap_or_default())
        .collect())
}

#[tauri::command]
pub async fn review_conclusion_request(
    token: String,
    request_id: String,
    decision: String,
    review_notes: Option<String>,
) -> Result<(), String> {
    let session = validate_session_command(&token).await?;
    if session.role_name != "the_wanderer" && session.role_name != "the_taskmaster" {
        return Err(
            "Only the_wanderer or the_taskmaster can review conclusion requests".to_string(),
        );
    }

    ensure_conclusion_requests_table().await?;

    let rid = Uuid::parse_str(&request_id).map_err(|_| "Invalid request ID".to_string())?;

    if decision != "approved" && decision != "rejected" {
        return Err("Decision must be 'approved' or 'rejected'".to_string());
    }

    let req: Option<(Uuid,)> = sqlx::query_as(
        "SELECT mission_id FROM mission_conclusion_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(rid)
    .fetch_optional(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    let (mission_id,) = req.ok_or_else(|| "Conclusion request not found".to_string())?;

    sqlx::query(
        "UPDATE mission_conclusion_requests SET status = $1, reviewed_by = $2, review_notes = $3 WHERE id = $4 AND deleted_at IS NULL"
    )
    .bind(&decision)
    .bind(session.user_id)
    .bind(&review_notes)
    .bind(rid)
    .execute(db::get_db())
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    if decision == "approved" {
        sqlx::query("UPDATE missions SET status = 'completed' WHERE id = $1 AND deleted_at IS NULL")
            .bind(mission_id)
            .execute(db::get_db())
            .await
            .map_err(|e| format!("DB error: {}", e))?;
    }

    let _ = write_audit_log(
        Some(session.user_id),
        "REVIEW_CONCLUSION_REQUEST",
        Some("mission_conclusion_requests"),
        Some(rid),
        None,
        Some(serde_json::json!({ "decision": decision })),
    )
    .await;

    Ok(())
}
