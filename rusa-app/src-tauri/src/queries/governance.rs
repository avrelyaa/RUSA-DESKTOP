use crate::db::get_db;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct VoteRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub vote_type: Option<String>,
    pub initiated_by: Option<Uuid>,
    pub interrupted_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct VoteDetailRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub initiated_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub yes_count: Option<i64>,
    pub no_count: Option<i64>,
    pub abstain_count: Option<i64>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MeetingRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub location: Option<String>,
    pub created_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub to_recipients: Option<serde_json::Value>,
    pub cc_recipients: Option<serde_json::Value>,
    pub bcc_recipients: Option<serde_json::Value>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct RelocationRow {
    pub id: Uuid,
    pub staff_id: Option<Uuid>,
    pub to_location: String,
    pub relocation_type: Option<String>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub reason: Option<String>,
    pub requested_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct EventDocumentRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub event_date: Option<chrono::DateTime<chrono::Utc>>,
    pub venue: Option<String>,
    pub venue_invoice: Option<String>,
    pub logged_by: Option<Uuid>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn insert_vote(
    title: &str,
    description: Option<&str>,
    initiated_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    insert_vote_typed(title, description, initiated_by, "pressing_issue").await
}

/// Insert a vote with an explicit vote_type.
/// vote_type must be one of: 'general', 'budget', 'pressing_issue'
pub async fn insert_vote_typed(
    title: &str,
    description: Option<&str>,
    initiated_by: Uuid,
    vote_type: &str,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO votes (title, description, status, initiated_by, deadline, vote_type) \
         VALUES ($1,$2,'open',$3, NOW() + INTERVAL '15 minutes',$4) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(initiated_by)
    .bind(vote_type)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn check_vote_status(
    vote_id: Uuid,
) -> Result<Option<(String, Option<chrono::DateTime<chrono::Utc>>)>, sqlx::Error> {
    let row: Option<(String, Option<chrono::DateTime<chrono::Utc>>)> = sqlx::query_as(
        "SELECT status, deadline FROM votes WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(vote_id)
    .fetch_optional(get_db())
    .await?;
    Ok(row)
}

pub async fn upsert_vote_ballot(
    vote_id: Uuid,
    voter_id: Uuid,
    decision: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO vote_ballots (vote_id, voter_id, decision, voted_at)
           VALUES ($1, $2, $3, NOW())
           ON CONFLICT (vote_id, voter_id) DO UPDATE SET decision = $3, voted_at = NOW()"#,
    )
    .bind(vote_id)
    .bind(voter_id)
    .bind(decision)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn resolve_expired_votes() -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE votes SET status = CASE
            WHEN (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = votes.id AND decision = 'yes') >
                 (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = votes.id AND decision = 'no')
            THEN 'passed'
            ELSE 'failed'
        END
        WHERE status = 'open' AND deadline IS NOT NULL AND deadline < NOW()"#,
    )
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn get_all_votes() -> Result<Vec<VoteRow>, sqlx::Error> {
    sqlx::query_as::<_, VoteRow>(
        "SELECT id, title, description, status, \
         COALESCE(vote_type, 'pressing_issue') as vote_type, \
         initiated_by, interrupted_by, created_at, deadline \
         FROM votes WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_vote_details(vote_id: Uuid) -> Result<Option<VoteDetailRow>, sqlx::Error> {
    sqlx::query_as::<_, VoteDetailRow>(
        r#"SELECT v.id, v.title, v.description, v.status, v.initiated_by, v.created_at, v.deadline,
                  (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = v.id AND decision = 'yes') as yes_count,
                  (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = v.id AND decision = 'no') as no_count,
                  (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = v.id AND decision = 'abstain') as abstain_count
           FROM votes v WHERE v.id = $1"#,
    )
    .bind(vote_id)
    .fetch_optional(get_db())
    .await
}

pub async fn interrupt_vote(vote_id: Uuid, interrupted_by: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE votes SET status = 'interrupted', interrupted_by = $1 WHERE id = $2 AND status = 'open'",
    )
    .bind(interrupted_by)
    .bind(vote_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_meeting(
    title: &str,
    description: Option<&str>,
    scheduled_at: chrono::DateTime<chrono::Utc>,
    location: Option<&str>,
    created_by: Uuid,
    to_recipients: &serde_json::Value,
    cc_recipients: &serde_json::Value,
    bcc_recipients: &serde_json::Value,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO meetings (title, description, scheduled_at, location, created_by, to_recipients, cc_recipients, bcc_recipients) VALUES ($1,$2,$3,$4,$5,$6,$7,$8) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(scheduled_at)
    .bind(location)
    .bind(created_by)
    .bind(to_recipients)
    .bind(cc_recipients)
    .bind(bcc_recipients)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn insert_meeting_attendee(
    meeting_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO meeting_attendees (meeting_id, user_id) VALUES ($1,$2) ON CONFLICT DO NOTHING",
    )
    .bind(meeting_id)
    .bind(user_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn get_all_meetings() -> Result<Vec<MeetingRow>, sqlx::Error> {
    sqlx::query_as::<_, MeetingRow>(
        "SELECT id, title, description, scheduled_at, location, created_by, created_at, \
         to_recipients, cc_recipients, bcc_recipients \
         FROM meetings WHERE deleted_at IS NULL ORDER BY scheduled_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn insert_staff_relocation(
    staff_id: Uuid,
    to_location: &str,
    relocation_type: &str,
    start_date: Option<chrono::DateTime<chrono::Utc>>,
    end_date: Option<chrono::DateTime<chrono::Utc>>,
    reason: Option<&str>,
    requested_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO staff_relocations (staff_id, to_location, relocation_type, start_date, end_date, reason, requested_by) VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id",
    )
    .bind(staff_id)
    .bind(to_location)
    .bind(relocation_type)
    .bind(start_date)
    .bind(end_date)
    .bind(reason)
    .bind(requested_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_relocations() -> Result<Vec<RelocationRow>, sqlx::Error> {
    sqlx::query_as::<_, RelocationRow>(
        "SELECT id, staff_id, to_location, relocation_type, start_date, end_date, reason, requested_by, created_at FROM staff_relocations ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn upsert_archive_permission(
    table_name: &str,
    record_id: Uuid,
    access_level: &str,
    set_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO archive_permissions (table_name, record_id, access_level, set_by) VALUES ($1,$2,$3,$4) ON CONFLICT (table_name, record_id) DO UPDATE SET access_level = $3, set_by = $4",
    )
    .bind(table_name)
    .bind(record_id)
    .bind(access_level)
    .bind(set_by)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_event_document(
    title: &str,
    description: Option<&str>,
    event_date: Option<chrono::DateTime<chrono::Utc>>,
    venue: Option<&str>,
    venue_invoice: Option<&str>,
    logged_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO event_documents (title, description, event_date, venue, venue_invoice, logged_by) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(event_date)
    .bind(venue)
    .bind(venue_invoice)
    .bind(logged_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_event_documents() -> Result<Vec<EventDocumentRow>, sqlx::Error> {
    sqlx::query_as::<_, EventDocumentRow>(
        "SELECT id, title, description, event_date, venue, venue_invoice, logged_by, created_at FROM event_documents WHERE deleted_at IS NULL ORDER BY event_date DESC",
    )
    .fetch_all(get_db())
    .await
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ArchivePermissionRow {
    pub id: Uuid,
    pub table_name: String,
    pub record_id: Uuid,
    pub access_level: String,
    pub set_by: Uuid,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn get_all_archive_permissions() -> Result<Vec<ArchivePermissionRow>, sqlx::Error> {
    sqlx::query_as::<_, ArchivePermissionRow>(
        "SELECT id, table_name, record_id, access_level, set_by, created_at FROM archive_permissions ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}
