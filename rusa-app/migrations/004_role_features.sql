-- ============================================================
-- Migration 004: Role-based feature completion
-- ============================================================

-- 1. Add 'reason' column to vote_ballots (required for every vote cast)
ALTER TABLE vote_ballots ADD COLUMN IF NOT EXISTS reason TEXT;

-- 2. Security tasks table (for The Guardian to assign tasks to security team)
CREATE TABLE IF NOT EXISTS security_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title TEXT NOT NULL,
    description TEXT,
    assigned_to UUID REFERENCES users(id),
    assigned_by UUID REFERENCES users(id),
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending','in_progress','result_submitted','completed','cancelled')),
    result_notes TEXT,
    priority VARCHAR(50) DEFAULT 'medium' CHECK (priority IN ('low','medium','high','critical')),
    due_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- 3. Librarian action logs
CREATE TABLE IF NOT EXISTS redacted_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name TEXT NOT NULL,
    record_id UUID NOT NULL,
    redact_reason TEXT,
    redacted_by UUID REFERENCES users(id),
    redacted_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS deleted_records_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name TEXT NOT NULL,
    record_id UUID NOT NULL,
    delete_reason TEXT,
    deleted_by UUID REFERENCES users(id),
    deleted_at TIMESTAMPTZ DEFAULT NOW()
);

-- 4. Add wanderer_stage to conclusion requests (created dynamically in Rust,
--    so we ALTER it here if it already exists from a prior run)
ALTER TABLE mission_conclusion_requests
    ADD COLUMN IF NOT EXISTS wanderer_recommendation TEXT;
ALTER TABLE mission_conclusion_requests
    ADD COLUMN IF NOT EXISTS wanderer_reviewed_by UUID REFERENCES users(id);
ALTER TABLE mission_conclusion_requests
    ADD COLUMN IF NOT EXISTS wanderer_review_notes TEXT;

-- 5. Vote scheduling: add scheduled_session column ('morning'=11:00, 'evening'=19:00)
ALTER TABLE votes ADD COLUMN IF NOT EXISTS scheduled_session TEXT
    CHECK (scheduled_session IN ('morning','evening'));

-- 6. Notification reference: link votes to their initiating budget/anomaly request
ALTER TABLE votes ADD COLUMN IF NOT EXISTS source_table TEXT;
ALTER TABLE votes ADD COLUMN IF NOT EXISTS source_record_id UUID;
