use crate::db::get_db;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct VoteRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
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
    scheduled_session: &str,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO votes (title, description, status, initiated_by, deadline, scheduled_session) VALUES ($1,$2,'open',$3, NOW() + INTERVAL '15 minutes', $4) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(initiated_by)
    .bind(scheduled_session)
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
    reason: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO vote_ballots (vote_id, voter_id, decision, reason, voted_at)
           VALUES ($1, $2, $3, $4, NOW())
           ON CONFLICT (vote_id, voter_id) DO UPDATE SET decision = $3, reason = $4, voted_at = NOW()"#,
    )
    .bind(vote_id)
    .bind(voter_id)
    .bind(decision)
    .bind(reason)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn resolve_expired_votes() -> Result<(), sqlx::Error> {
    // Auto-deny if all participating votes are 'abstain'
    sqlx::query(
        r#"UPDATE votes SET status = 'failed'
        WHERE status = 'open' AND deadline IS NOT NULL AND deadline < NOW()
          AND (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = votes.id) > 0
          AND (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = votes.id AND decision != 'abstain') = 0"#,
    )
    .execute(get_db())
    .await?;

    // Normal resolution: yes > no → passed, otherwise failed
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

/// Return (initiated_by, title) for all newly resolved votes so callers can notify requesters.
pub async fn get_newly_resolved_votes_for_notification() -> Result<Vec<(Uuid, Uuid, String, String)>, sqlx::Error> {
    // Returns (id, initiated_by, title, status) for votes resolved within the last minute
    let rows: Vec<(Uuid, Uuid, String, String)> = sqlx::query_as(
        r#"SELECT id, initiated_by, title, status FROM votes
           WHERE status IN ('passed','failed')
             AND deadline IS NOT NULL
             AND deadline > NOW() - INTERVAL '1 minute'
             AND initiated_by IS NOT NULL"#,
    )
    .fetch_all(get_db())
    .await?;
    Ok(rows)
}

pub async fn get_all_votes() -> Result<Vec<VoteRow>, sqlx::Error> {
    sqlx::query_as::<_, VoteRow>(
        "SELECT id, title, description, status, initiated_by, interrupted_by, created_at, deadline FROM votes WHERE deleted_at IS NULL ORDER BY created_at DESC",
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
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO meetings (title, description, scheduled_at, location, created_by) VALUES ($1,$2,$3,$4,$5) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(scheduled_at)
    .bind(location)
    .bind(created_by)
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
        "SELECT id, title, description, scheduled_at, location, created_by, created_at FROM meetings WHERE deleted_at IS NULL ORDER BY scheduled_at DESC",
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

// ─────────────────── All-abstain auto-deny ───────────────────────

/// If every ballot on this vote is 'abstain', immediately mark it failed.
pub async fn auto_deny_all_abstain(vote_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE votes SET status = 'failed'
           WHERE id = $1 AND status = 'open'
             AND (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = $1) > 0
             AND (SELECT COUNT(*) FROM vote_ballots WHERE vote_id = $1 AND decision != 'abstain') = 0"#,
    )
    .bind(vote_id)
    .execute(get_db())
    .await?;
    Ok(())
}

// ─────────────────── Librarian helpers ───────────────────────────

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct RedactedRecordRow {
    pub id: Uuid,
    pub table_name: String,
    pub record_id: Uuid,
    pub redact_reason: Option<String>,
    pub redacted_by: Option<Uuid>,
    pub redacted_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn insert_redacted_record(
    table_name: &str,
    record_id: Uuid,
    reason: &str,
    redacted_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO redacted_records (table_name, record_id, redact_reason, redacted_by) VALUES ($1,$2,$3,$4)",
    )
    .bind(table_name)
    .bind(record_id)
    .bind(reason)
    .bind(redacted_by)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn insert_deleted_record_log(
    table_name: &str,
    record_id: Uuid,
    reason: &str,
    deleted_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO deleted_records_log (table_name, record_id, delete_reason, deleted_by) VALUES ($1,$2,$3,$4)",
    )
    .bind(table_name)
    .bind(record_id)
    .bind(reason)
    .bind(deleted_by)
    .execute(get_db())
    .await?;
    Ok(())
}

/// Attempt to soft-delete a record in the given table by setting deleted_at.
/// This is best-effort; not all tables have deleted_at.
pub async fn soft_delete_record(table_name: &str, record_id: Uuid) -> Result<(), sqlx::Error> {
    let query = format!(
        "UPDATE {} SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        table_name
    );
    sqlx::query(&query)
        .bind(record_id)
        .execute(get_db())
        .await?;
    Ok(())
}

pub async fn get_all_redacted_records() -> Result<Vec<RedactedRecordRow>, sqlx::Error> {
    sqlx::query_as::<_, RedactedRecordRow>(
        "SELECT id, table_name, record_id, redact_reason, redacted_by, redacted_at FROM redacted_records ORDER BY redacted_at DESC",
    )
    .fetch_all(get_db())
    .await
}
