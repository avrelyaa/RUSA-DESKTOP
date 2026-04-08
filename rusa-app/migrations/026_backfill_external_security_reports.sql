-- Migration 026: Backfill external security reports from research security_reports
-- Copies existing department security reports into the external_reports queue so
-- Security can review legacy submissions alongside new mirrored submissions.
-- Execution date: 2026-04-08

WITH source_reports AS (
    SELECT
        sr.id AS security_report_id,
        sr.title,
        sr.description,
        sr.submitted_by,
        u.full_name AS reporter_name,
        u.email AS reporter_email,
        sr.category,
        sr.severity,
        sr.status AS security_status,
        sr.security_staff_notes,
        sr.resolved_at,
        sr.created_at,
        CASE
            WHEN sr.status = 'new' THEN 'PENDING_REVIEW'
            WHEN sr.status IN ('acknowledged', 'investigating') THEN 'UNDER_INVESTIGATION'
            WHEN sr.status IN ('resolved', 'closed') THEN 'CLOSED_RESOLVED'
            ELSE 'PENDING_REVIEW'
        END AS external_status,
        format(
            'Category: %s\nSeverity: %s\n\n%s',
            sr.category,
            sr.severity,
            sr.description
        ) AS external_description,
        CASE
            WHEN sr.status = 'new' THEN NULL
            ELSE COALESCE(sr.resolved_at, sr.created_at)
        END AS status_changed_at
    FROM security_reports sr
    LEFT JOIN users u ON u.id = sr.submitted_by
    WHERE sr.deleted_at IS NULL
),
inserted_external_reports AS (
    INSERT INTO external_reports (
        title,
        description,
        submitted_by,
        reporter_name,
        reporter_email,
        security_type,
        status,
        created_at,
        updated_at,
        status_changed_at,
        closed_reason,
        public_update_message
    )
    SELECT
        src.title,
        src.external_description,
        src.submitted_by,
        src.reporter_name,
        src.reporter_email,
        'earth',
        src.external_status,
        src.created_at,
        src.created_at,
        src.status_changed_at,
        CASE
            WHEN src.external_status = 'CLOSED_RESOLVED' THEN src.security_staff_notes
            ELSE NULL
        END,
        NULL
    FROM source_reports src
    WHERE NOT EXISTS (
        SELECT 1
        FROM external_reports er
        WHERE er.deleted_at IS NULL
          AND er.submitted_by = src.submitted_by
          AND er.title = src.title
          AND er.description = src.external_description
    )
    RETURNING id, title, submitted_by, created_at
)
INSERT INTO external_security_report_events (
    report_id,
    actor_id,
    event_type,
    from_status,
    to_status,
    metadata,
    created_at
)
SELECT
    ier.id,
    ier.submitted_by,
    'CREATED',
    NULL,
    src.external_status,
    jsonb_build_object(
        'source', 'research_security_report_backfill',
        'security_report_id', src.security_report_id,
        'category', src.category,
        'severity', src.severity,
        'security_status', src.security_status
    ),
    ier.created_at
FROM inserted_external_reports ier
JOIN source_reports src
  ON src.title = ier.title
 AND src.submitted_by = ier.submitted_by
 AND src.created_at = ier.created_at;