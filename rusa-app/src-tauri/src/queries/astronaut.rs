use crate::db::get_db;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MissionRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub mission_type: Option<String>,
    pub status: Option<String>,
    pub ship_id: Option<Uuid>,
    pub sector_id: Option<Uuid>,
    pub planet_id: Option<Uuid>,
    pub start_date: Option<chrono::NaiveDate>,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MissionReportRow {
    pub id: Uuid,
    pub mission_id: Uuid,
    pub report_type: Option<String>,
    pub content: Option<String>,
    pub submitted_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct JournalRow {
    pub id: Uuid,
    pub mission_id: Option<Uuid>,
    pub title: String,
    pub content: Option<String>,
    pub is_public: Option<bool>,
    pub mission_location: Option<String>,
    pub mission_type: Option<String>,
    pub progress_report: Option<String>,
    pub next_steps: Option<String>,
    pub approval_status: Option<String>,
    pub author_id: Option<Uuid>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct SectorRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub boundaries: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ShipRow {
    pub id: Uuid,
    pub name: String,
    pub ship_type: Option<String>,
    pub status: Option<String>,
    pub capacity: Option<i32>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MissionConclusionRequestRow {
    pub id: Uuid,
    pub mission_id: Uuid,
    pub submitted_by: Option<Uuid>,
    pub report_summary: Option<String>,
    pub status: String,
    pub reviewed_by: Option<Uuid>,
    pub review_notes: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct AstronautStatsRow {
    pub total_missions: Option<i64>,
    pub completed_missions: Option<i64>,
    pub total_reports: Option<i64>,
    pub total_journals: Option<i64>,
    pub terrain_missions: Option<i64>,
    pub interstellar_missions: Option<i64>,
    pub pending_approval: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct AstronautDirectoryRow {
    pub astronaut_id: Uuid,
    pub full_name: String,
    pub username: String,
    pub current_status: String,
    pub current_assignment: Option<String>,
    pub terrain_count: i64,
    pub interstellar_count: i64,
    pub visited_locations: Vec<String>,
    pub comm_link: Option<String>,
}

pub async fn get_all_missions() -> Result<Vec<MissionRow>, sqlx::Error> {
    sqlx::query_as::<_, MissionRow>(
        "SELECT id, title, description, mission_type, status, ship_id, sector_id, planet_id, start_date, created_by, created_at FROM missions WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn insert_mission(
    title: &str,
    description: Option<&str>,
    mission_type: &str,
    ship_id: Option<Uuid>,
    sector_id: Option<Uuid>,
    planet_id: Option<Uuid>,
    start_date: Option<chrono::NaiveDate>,
    created_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO missions (title, description, mission_type, status, ship_id, sector_id, planet_id, start_date, created_by) VALUES ($1,$2,$3,'planned',$4,$5,$6,$7,$8) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(mission_type)
    .bind(ship_id)
    .bind(sector_id)
    .bind(planet_id)
    .bind(start_date)
    .bind(created_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn update_mission_status(
    mission_id: Uuid,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE missions SET status = $1 WHERE id = $2 AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(mission_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_mission_crew(
    mission_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO mission_crew (mission_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(mission_id)
    .bind(user_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_mission_report(
    mission_id: Uuid,
    report_type: &str,
    content: &str,
    submitted_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO mission_reports (mission_id, report_type, content, submitted_by) VALUES ($1,$2,$3,$4) RETURNING id",
    )
    .bind(mission_id)
    .bind(report_type)
    .bind(content)
    .bind(submitted_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_mission_reports_by_mission(
    mission_id: Uuid,
) -> Result<Vec<MissionReportRow>, sqlx::Error> {
    sqlx::query_as::<_, MissionReportRow>(
        "SELECT id, mission_id, report_type, content, submitted_by, created_at FROM mission_reports WHERE mission_id = $1 ORDER BY created_at DESC",
    )
    .bind(mission_id)
    .fetch_all(get_db())
    .await
}

pub async fn insert_journal(
    mission_id: Option<Uuid>,
    mission_location: &str,
    mission_type: &str,
    progress_report: &str,
    next_steps: &str,
    approval_status: &str,
    author_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let title = format!("{} mission log", mission_location);
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO exploration_journals (mission_id, title, content, is_public, mission_location, mission_type, progress_report, next_steps, approval_status, author_id) VALUES ($1,$2,$3,false,$4,$5,$6,$7,$8,$9) RETURNING id",
    )
    .bind(mission_id)
    .bind(title)
    .bind(progress_report)
    .bind(mission_location)
    .bind(mission_type)
    .bind(progress_report)
    .bind(next_steps)
    .bind(approval_status)
    .bind(author_id)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_journals_for_user(user_id: Uuid) -> Result<Vec<JournalRow>, sqlx::Error> {
    sqlx::query_as::<_, JournalRow>(
        "SELECT id, mission_id, title, content, is_public, mission_location, mission_type, progress_report, next_steps, approval_status, author_id, updated_at, created_at FROM exploration_journals WHERE author_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await
}

pub async fn get_all_sectors() -> Result<Vec<SectorRow>, sqlx::Error> {
    sqlx::query_as::<_, SectorRow>(
        "SELECT id, name, description, boundaries, created_by, created_at FROM sectors WHERE deleted_at IS NULL ORDER BY name",
    )
    .fetch_all(get_db())
    .await
}

pub async fn insert_sector(
    name: &str,
    description: Option<&str>,
    boundaries: Option<&str>,
    created_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO sectors (name, description, boundaries, created_by) VALUES ($1,$2,$3,$4) RETURNING id",
    )
    .bind(name)
    .bind(description)
    .bind(boundaries)
    .bind(created_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn update_sector_name(
    sector_id: Uuid,
    new_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE sectors SET name = $1 WHERE id = $2 AND deleted_at IS NULL",
    )
    .bind(new_name)
    .bind(sector_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn get_all_ships() -> Result<Vec<ShipRow>, sqlx::Error> {
    sqlx::query_as::<_, ShipRow>(
        "SELECT id, name, ship_type, status, capacity, created_at FROM ships WHERE deleted_at IS NULL ORDER BY name",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_astronaut_stats(user_id: Uuid) -> Result<AstronautStatsRow, sqlx::Error> {
    sqlx::query_as::<_, AstronautStatsRow>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM mission_crew WHERE user_id = $1) as total_missions,
            (SELECT COUNT(*) FROM mission_crew mc JOIN missions m ON mc.mission_id = m.id WHERE mc.user_id = $1 AND m.status = 'completed') as completed_missions,
            (SELECT COUNT(*) FROM mission_reports WHERE submitted_by = $1) as total_reports,
            (SELECT COUNT(*) FROM exploration_journals WHERE author_id = $1 AND deleted_at IS NULL) as total_journals,
            (SELECT COUNT(*) FROM exploration_journals WHERE author_id = $1 AND deleted_at IS NULL AND approval_status = 'approved' AND mission_type = 'terraneous') as terrain_missions,
            (SELECT COUNT(*) FROM exploration_journals WHERE author_id = $1 AND deleted_at IS NULL AND approval_status = 'approved' AND mission_type = 'interstellar') as interstellar_missions,
            (SELECT COUNT(*) FROM exploration_journals WHERE author_id = $1 AND deleted_at IS NULL AND approval_status = 'submitted') as pending_approval
        "#,
    )
    .bind(user_id)
    .fetch_one(get_db())
    .await
}

pub async fn get_astronaut_directory() -> Result<Vec<AstronautDirectoryRow>, sqlx::Error> {
    sqlx::query_as::<_, AstronautDirectoryRow>(
        r#"
        SELECT
            u.id as astronaut_id,
            u.full_name,
            u.username,
            COALESCE(last_log.approval_status, 'available') as current_status,
            last_log.mission_location as current_assignment,
            COALESCE(counters.terrain_count, 0) as terrain_count,
            COALESCE(counters.interstellar_count, 0) as interstellar_count,
            COALESCE(locs.visited_locations, ARRAY[]::TEXT[]) as visited_locations,
            COALESCE(u.department_email, u.email) as comm_link
        FROM users u
        JOIN roles r ON r.id = u.role_id
        LEFT JOIN LATERAL (
            SELECT ej.approval_status, ej.mission_location
            FROM exploration_journals ej
            WHERE ej.author_id = u.id AND ej.deleted_at IS NULL
            ORDER BY COALESCE(ej.updated_at, ej.created_at) DESC
            LIMIT 1
        ) last_log ON TRUE
        LEFT JOIN LATERAL (
            SELECT
                COUNT(*) FILTER (WHERE ej.approval_status = 'approved' AND ej.mission_type = 'terraneous')::BIGINT as terrain_count,
                COUNT(*) FILTER (WHERE ej.approval_status = 'approved' AND ej.mission_type = 'interstellar')::BIGINT as interstellar_count
            FROM exploration_journals ej
            WHERE ej.author_id = u.id AND ej.deleted_at IS NULL
        ) counters ON TRUE
        LEFT JOIN LATERAL (
            SELECT ARRAY_AGG(DISTINCT ej.mission_location) FILTER (
                WHERE ej.mission_location IS NOT NULL AND ej.mission_location <> '' AND ej.approval_status = 'approved'
            ) as visited_locations
            FROM exploration_journals ej
            WHERE ej.author_id = u.id AND ej.deleted_at IS NULL
        ) locs ON TRUE
        WHERE u.deleted_at IS NULL
          AND COALESCE(u.is_active, true) = true
          AND r.name = 'astronaut'
        ORDER BY u.full_name ASC
        "#,
    )
    .fetch_all(get_db())
    .await
}

pub async fn ensure_conclusion_requests_table() -> Result<(), sqlx::Error> {
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
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_conclusion_request(
    mission_id: Uuid,
    submitted_by: Uuid,
    report_summary: &str,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO mission_conclusion_requests (mission_id, submitted_by, report_summary, status) VALUES ($1,$2,$3,'pending') RETURNING id",
    )
    .bind(mission_id)
    .bind(submitted_by)
    .bind(report_summary)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_conclusion_requests() -> Result<Vec<MissionConclusionRequestRow>, sqlx::Error>
{
    sqlx::query_as::<_, MissionConclusionRequestRow>(
        "SELECT id, mission_id, submitted_by, report_summary, status, reviewed_by, review_notes, created_at FROM mission_conclusion_requests WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_user_conclusion_requests(
    user_id: Uuid,
) -> Result<Vec<MissionConclusionRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, MissionConclusionRequestRow>(
        "SELECT id, mission_id, submitted_by, report_summary, status, reviewed_by, review_notes, created_at FROM mission_conclusion_requests WHERE submitted_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await
}

pub async fn get_conclusion_request_mission_id(
    request_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT mission_id FROM mission_conclusion_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(request_id)
    .fetch_optional(get_db())
    .await?;
    Ok(row.map(|(id,)| id))
}

pub async fn get_mission_created_by(mission_id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Option<Uuid>,)> = sqlx::query_as(
        "SELECT created_by FROM missions WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(mission_id)
    .fetch_optional(get_db())
    .await?;
    Ok(row.and_then(|(id,)| id))
}

pub async fn update_conclusion_request_review(
    request_id: Uuid,
    decision: &str,
    reviewed_by: Uuid,
    review_notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE mission_conclusion_requests SET status = $1, reviewed_by = $2, review_notes = $3 WHERE id = $4 AND deleted_at IS NULL",
    )
    .bind(decision)
    .bind(reviewed_by)
    .bind(review_notes)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn update_conclusion_request_review_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    request_id: Uuid,
    decision: &str,
    reviewed_by: Uuid,
    review_notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE mission_conclusion_requests SET status = $1, reviewed_by = $2, review_notes = $3 WHERE id = $4 AND deleted_at IS NULL",
    )
    .bind(decision)
    .bind(reviewed_by)
    .bind(review_notes)
    .bind(request_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn complete_mission(mission_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE missions SET status = 'completed' WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(mission_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn complete_mission_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    mission_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE missions SET status = 'completed' WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(mission_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Retrieve the user who submitted a given conclusion request.
pub async fn get_conclusion_request_submitter(
    request_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Option<Uuid>,)> = sqlx::query_as(
        "SELECT submitted_by FROM mission_conclusion_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(request_id)
    .fetch_optional(get_db())
    .await?;
    Ok(row.and_then(|(id,)| id))
}

pub async fn get_conclusion_request_submitter_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    request_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    let row: Option<(Option<Uuid>,)> = sqlx::query_as(
        "SELECT submitted_by FROM mission_conclusion_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(request_id)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(row.and_then(|(id,)| id))
}

/// Increment the mission_counter for a user (called when an astronaut's conclusion is approved).
pub async fn increment_mission_counter(user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users SET mission_counter = mission_counter + 1 WHERE id = $1",
    )
    .bind(user_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn increment_mission_counter_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users SET mission_counter = mission_counter + 1 WHERE id = $1",
    )
    .bind(user_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn set_latest_journal_approval_status_for_author_mission_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    author_id: Uuid,
    mission_id: Uuid,
    approval_status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE exploration_journals
        SET approval_status = $1,
            updated_at = NOW()
        WHERE id = (
            SELECT id
            FROM exploration_journals
            WHERE author_id = $2
              AND mission_id = $3
              AND deleted_at IS NULL
            ORDER BY COALESCE(updated_at, created_at) DESC
            LIMIT 1
        )
        "#,
    )
    .bind(approval_status)
    .bind(author_id)
    .bind(mission_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct PlanetRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub star_system: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn ensure_planets_table() -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS planets (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name TEXT NOT NULL,
            description TEXT,
            star_system TEXT,
            created_by UUID REFERENCES users(id),
            created_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"#,
    )
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn get_all_planets() -> Result<Vec<PlanetRow>, sqlx::Error> {
    ensure_planets_table().await?;
    sqlx::query_as::<_, PlanetRow>(
        "SELECT id, name, description, star_system, created_by, created_at FROM planets WHERE deleted_at IS NULL ORDER BY name ASC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn insert_planet(
    name: &str,
    description: Option<&str>,
    star_system: Option<&str>,
    created_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    ensure_planets_table().await?;
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO planets (name, description, star_system, created_by) VALUES ($1,$2,$3,$4) RETURNING id",
    )
    .bind(name)
    .bind(description)
    .bind(star_system)
    .bind(created_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn update_planet_name(planet_id: Uuid, new_name: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE planets SET name = $1 WHERE id = $2 AND deleted_at IS NULL")
        .bind(new_name)
        .bind(planet_id)
        .execute(get_db())
        .await?;
    Ok(())
}
