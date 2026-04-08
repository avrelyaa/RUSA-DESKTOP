-- Migration 022: Add sanitary_shifts table for My Schedule feature
-- Allows the Head of Sanitary to allocate shifts to crew members,
-- and crew members to view their own assigned shifts (read-only).

CREATE TABLE IF NOT EXISTS sanitary_shifts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    staff_id UUID NOT NULL REFERENCES users(id),
    allocated_by UUID NOT NULL REFERENCES users(id),
    shift_start TIMESTAMPTZ NOT NULL,
    shift_end TIMESTAMPTZ NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_sanitary_shifts_staff_id ON sanitary_shifts(staff_id);
CREATE INDEX IF NOT EXISTS idx_sanitary_shifts_shift_start ON sanitary_shifts(shift_start);
