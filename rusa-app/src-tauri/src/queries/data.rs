use crate::db::get_db;
use uuid::Uuid;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct DataRequestRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub data_type: Option<String>,
    pub status: Option<String>,
    pub requested_by: Option<Uuid>,
    pub reviewed_by: Option<Uuid>,
    pub review_notes: Option<String>,
    pub response_data: Option<String>,
    pub responded_by: Option<Uuid>,
    pub analyst_notes: Option<String>,
    pub analyst_rejection_reason: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub delivery_notes: Option<String>,
    pub delivered_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn insert_data_request(
    title: &str,
    description: &str,
    data_type: Option<&str>,
    requested_by: Uuid,
) -> Result<Uuid, sqlx::Error> {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO data_requests (title, description, data_type, status, requested_by) VALUES ($1,$2,$3,'pending',$4) RETURNING id",
    )
    .bind(title)
    .bind(description)
    .bind(data_type)
    .bind(requested_by)
    .fetch_one(get_db())
    .await?;
    Ok(row.0)
}

pub async fn get_all_data_requests() -> Result<Vec<DataRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, DataRequestRow>(
        "SELECT id, title, description, data_type, status, requested_by, reviewed_by, review_notes, response_data, responded_by, analyst_notes, analyst_rejection_reason, assigned_to, delivery_notes, delivered_at, created_at FROM data_requests WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_all_delivered_requests() -> Result<Vec<DataRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, DataRequestRow>(
        "SELECT id, title, description, data_type, status, requested_by, reviewed_by, review_notes, response_data, responded_by, analyst_notes, analyst_rejection_reason, assigned_to, delivery_notes, delivered_at, created_at FROM data_requests WHERE status = 'delivered' AND deleted_at IS NULL ORDER BY delivered_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_analyst_data_requests() -> Result<Vec<DataRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, DataRequestRow>(
        "SELECT id, title, description, data_type, status, requested_by, reviewed_by, review_notes, response_data, responded_by, analyst_notes, analyst_rejection_reason, assigned_to, delivery_notes, delivered_at, created_at FROM data_requests WHERE status IN ('approved', 'processing', 'analyst_submitted', 'analyst_rejected') AND (data_type IS NULL OR LOWER(data_type) != 'medical') AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(get_db())
    .await
}

pub async fn get_user_data_requests(user_id: Uuid) -> Result<Vec<DataRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, DataRequestRow>(
        "SELECT id, title, description, data_type, status, requested_by, reviewed_by, review_notes, response_data, responded_by, analyst_notes, analyst_rejection_reason, assigned_to, delivery_notes, delivered_at, created_at FROM data_requests WHERE requested_by = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await
}

pub async fn get_data_request_data_type(request_id: Uuid) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT data_type FROM data_requests WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(request_id)
    .fetch_optional(get_db())
    .await?;
    Ok(row.and_then(|(dt,)| dt))
}

pub async fn update_data_request_review(
    request_id: Uuid,
    status: &str,
    reviewed_by: Uuid,
    notes: Option<&str>,
    assigned_to: Option<Uuid>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE data_requests SET status = $1, reviewed_by = $2, review_notes = $3, assigned_to = COALESCE($4, assigned_to) WHERE id = $5 AND deleted_at IS NULL",
    )
    .bind(status)
    .bind(reviewed_by)
    .bind(notes)
    .bind(assigned_to)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn update_data_request_processing(
    request_id: Uuid,
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE data_requests SET status = 'processing', assigned_to = COALESCE(assigned_to, $1) WHERE id = $2 AND status = 'approved' AND deleted_at IS NULL",
    )
    .bind(user_id)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn update_data_request_analyst_response(
    request_id: Uuid,
    response_data: &str,
    analyst_notes: Option<&str>,
    responded_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE data_requests SET response_data = $1, analyst_notes = $2, responded_by = $3, status = 'analyst_submitted' WHERE id = $4 AND status IN ('approved', 'processing') AND deleted_at IS NULL",
    )
    .bind(response_data)
    .bind(analyst_notes)
    .bind(responded_by)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn update_data_request_delivered(request_id: Uuid, delivery_notes: Option<&str>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE data_requests SET status = 'delivered', delivered_at = NOW(), delivery_notes = $2 WHERE id = $1 AND status = 'analyst_submitted' AND deleted_at IS NULL",
    )
    .bind(request_id)
    .bind(delivery_notes)
    .execute(get_db())
    .await?;
    Ok(())
}

pub async fn get_delivered_requests_for_user(
    user_id: Uuid,
) -> Result<Vec<DataRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, DataRequestRow>(
        "SELECT id, title, description, data_type, status, requested_by, reviewed_by, review_notes, response_data, responded_by, analyst_notes, analyst_rejection_reason, assigned_to, delivery_notes, delivered_at, created_at FROM data_requests WHERE status = 'delivered' AND requested_by = $1 AND deleted_at IS NULL ORDER BY delivered_at DESC",
    )
    .bind(user_id)
    .fetch_all(get_db())
    .await
}

pub async fn update_data_request_analyst_rejection(
    request_id: Uuid,
    rejection_reason: &str,
    responded_by: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE data_requests SET status = 'analyst_rejected', analyst_rejection_reason = $1, responded_by = $2 WHERE id = $3 AND status IN ('approved', 'processing') AND (data_type IS NULL OR LOWER(data_type) != 'medical') AND deleted_at IS NULL",
    )
    .bind(rejection_reason)
    .bind(responded_by)
    .bind(request_id)
    .execute(get_db())
    .await?;
    Ok(())
}
