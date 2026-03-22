use crate::db::get_db;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct IncidentReportRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub incident_date: Option<chrono::DateTime<chrono::Utc>>,
    pub severity: Option<String>,
    pub status: Option<String>,
    pub reported_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct LostFoundRow {
    pub id: Uuid,
    pub item_name: String,
    pub description: Option<String>,
    pub found_location: Option<String>,
    pub found_date: Option<chrono::NaiveDate>,
    pub status: Option<String>,
    pub logged_by: Option<Uuid>,
    pub claimed_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct BroadcastRequestRow {
    pub id: Uuid,
    pub title: String,
    pub content: Option<String>,
    pub target_audience: Option<String>,
    pub status: Option<String>,
    pub requested_by: Option<Uuid>,
    pub reviewed_by: Option<Uuid>,
    pub review_notes: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct SecurityFindingRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub findings_date: Option<chrono::NaiveDate>,
    pub reported_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn insert_incident_report(
    title: &str,
    description: &str,
    location: Option<&str>,
    incident_date: Option<chrono::DateTime<chrono::Utc>>,
    severity: &str,
    reported_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO incident_reports (title, description, location, incident_date, severity, status, reported_by) VALUES ($1,$2,$3,$4,$5,'open',$6) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(location)
    .bind(incident_date)
    .bind(severity)
    .bind(reported_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_incident_reports() -> Result<Vec<IncidentReportRow>, sqlx::Error> {
    sqlx::query_as::<_, IncidentReportRow>(
        "SELECT id, title, description, location, incident_date, severity, status, reported_by, created_at FROM incident_reports WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn update_incident_status(
    report_id: Uuid,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE incident_reports SET status = $1 WHERE id = $2 AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(report_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_lost_found_item(
    item_name: &str,
    description: Option<&str>,
    found_location: Option<&str>,
    found_date: Option<chrono::NaiveDate>,
    logged_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO lost_and_found (item_name, description, found_location, found_date, status, logged_by) VALUES ($1,$2,$3,$4,'unclaimed',$5) RETURNING id",
    )
    .bind(item_name)
    .bind(description)
    .bind(found_location)
    .bind(found_date)
    .bind(logged_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_lost_found() -> Result<Vec<LostFoundRow>, sqlx::Error> {
    sqlx::query_as::<_, LostFoundRow>(
        "SELECT id, item_name, description, found_location, found_date, status, logged_by, claimed_by, created_at FROM lost_and_found WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn claim_lost_found_item(
    item_id: Uuid,
    claimed_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE lost_and_found SET status = 'claimed', claimed_by = $1 WHERE id = $2 AND deleted_at IS NULL AND status = 'unclaimed'",
    )
    .bind(claimed_by)
    .bind(item_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_broadcast_request(
    title: &str,
    content: &str,
    target_audience: Option<&str>,
    requested_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO broadcast_requests (title, content, target_audience, status, requested_by) VALUES ($1,$2,$3,'pending',$4) RETURNING id",
    )
    .bind(title)
    .bind(content)
    .bind(target_audience)
    .bind(requested_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_broadcast_requests() -> Result<Vec<BroadcastRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, BroadcastRequestRow>(
        "SELECT id, title, content, target_audience, status, requested_by, reviewed_by, review_notes, created_at FROM broadcast_requests WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn update_broadcast_request_review(
    request_id: Uuid,
    status: &str,
    reviewed_by: Uuid,
    notes: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE broadcast_requests SET status = $1, reviewed_by = $2, review_notes = $3 WHERE id = $4 AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(reviewed_by)
    .bind(notes)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_security_finding(
    title: &str,
    description: Option<&str>,
    findings_date: Option<chrono::NaiveDate>,
    reported_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO security_findings (title, description, findings_date, reported_by) VALUES ($1,$2,$3,$4) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(findings_date)
    .bind(reported_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_security_findings() -> Result<Vec<SecurityFindingRow>, sqlx::Error> {
    sqlx::query_as::<_, SecurityFindingRow>(
        "SELECT id, title, description, findings_date, reported_by, created_at FROM security_findings WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct SecurityTaskRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub assigned_by: Option<Uuid>,
    pub status: String,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn ensure_security_tasks_table() -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS security_tasks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title TEXT NOT NULL,
            description TEXT,
            assigned_to UUID REFERENCES users(id),
            assigned_by UUID REFERENCES users(id),
            status TEXT NOT NULL DEFAULT 'pending',
            due_date TIMESTAMPTZ,
            created_at TIMESTAMPTZ DEFAULT NOW(),
            deleted_at TIMESTAMPTZ
        )"#,
    )
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_security_task(
    title: &str,
    description: Option<&str>,
    assigned_to: Uuid,
    assigned_by: Uuid,
    due_date: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO security_tasks (title, description, assigned_to, assigned_by, status, due_date) VALUES ($1,$2,$3,$4,'pending',$5) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(assigned_to)
    .bind(assigned_by)
    .bind(due_date)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_security_tasks_for_assigner(assigner_id: Uuid) -> Result<Vec<SecurityTaskRow>, sqlx::Error> {
    sqlx::query_as::<_, SecurityTaskRow>(
        "SELECT id, title, description, assigned_to, assigned_by, status, due_date, created_at FROM security_tasks WHERE assigned_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .bind(assigner_id)
    .fetch_all(get_db())
    .await
}

pub async fn get_security_tasks_for_user(user_id: Uuid) -> Result<Vec<SecurityTaskRow>, sqlx::Error> {
    sqlx::query_as::<_, SecurityTaskRow>(
        "SELECT id, title, description, assigned_to, assigned_by, status, due_date, created_at FROM security_tasks WHERE assigned_to = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await
}

pub async fn update_security_task_status(task_id: Uuid, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE security_tasks SET status = $1 WHERE id = $2 AND deleted_at IS NULL")
        .bind(status)
        .bind(task_id)
        .execute(get_db())
        .await?;
    Ok(())
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ExternalReportRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub submitted_by: Option<Uuid>,
    pub security_type: Option<String>,
    pub status: Option<String>,
    pub incident_report_id: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn insert_external_report(
    title: &str,
    description: &str,
    submitted_by: Uuid,
    security_type: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO external_reports (title, description, submitted_by, security_type, status) VALUES ($1,$2,$3,$4,'submitted') RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(submitted_by)
    .bind(security_type)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_external_reports() -> Result<Vec<ExternalReportRow>, sqlx::Error> {
    sqlx::query_as::<_, ExternalReportRow>(
        "SELECT id, title, description, submitted_by, security_type, status, incident_report_id, created_at FROM external_reports WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}
