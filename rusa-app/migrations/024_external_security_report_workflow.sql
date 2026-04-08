-- Migration 024: External Security Report Workflow
-- Adds proper reporter fields, status lifecycle, public token for reporter tracking,
-- and an audit-trail events table for external security reports.
-- Execution date: 2026-04-08

-- 1. Add new columns to external_reports
ALTER TABLE external_reports
  ADD COLUMN IF NOT EXISTS reporter_name         TEXT,
  ADD COLUMN IF NOT EXISTS reporter_email        TEXT,
  ADD COLUMN IF NOT EXISTS updated_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  ADD COLUMN IF NOT EXISTS status_changed_at     TIMESTAMPTZ,
  ADD COLUMN IF NOT EXISTS closed_reason         TEXT,
  ADD COLUMN IF NOT EXISTS public_update_message TEXT,
  ADD COLUMN IF NOT EXISTS public_token          UUID NOT NULL DEFAULT gen_random_uuid();

-- 2. Migrate existing status values to the new canonical set
UPDATE external_reports SET status = 'PENDING_REVIEW'      WHERE status = 'submitted';
UPDATE external_reports SET status = 'UNDER_INVESTIGATION' WHERE status = 'under_review';
UPDATE external_reports SET status = 'CLOSED_RESOLVED'     WHERE status IN ('converted_to_incident', 'archived');

-- 3. Replace old status constraint with the new enum
ALTER TABLE external_reports DROP CONSTRAINT IF EXISTS external_reports_status_check;
ALTER TABLE external_reports
  ADD CONSTRAINT external_reports_status_check
    CHECK (status IN ('PENDING_REVIEW','UNDER_INVESTIGATION','CLOSED_RESOLVED','DISMISSED'));

-- Set proper column default
ALTER TABLE external_reports ALTER COLUMN status SET DEFAULT 'PENDING_REVIEW';

-- 4. Unique index on public_token (for reporter-facing look-ups)
CREATE UNIQUE INDEX IF NOT EXISTS idx_ext_reports_public_token
  ON external_reports(public_token) WHERE deleted_at IS NULL;

-- 5. Audit-trail events table
CREATE TABLE IF NOT EXISTS external_security_report_events (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    report_id   UUID NOT NULL REFERENCES external_reports(id),
    actor_id    UUID REFERENCES users(id),
    event_type  TEXT NOT NULL
                  CHECK (event_type IN ('CREATED','STATUS_CHANGED','COMMENT_ADDED')),
    from_status TEXT,
    to_status   TEXT,
    metadata    JSONB,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ext_report_events_report
  ON external_security_report_events(report_id);
