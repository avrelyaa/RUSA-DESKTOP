-- Add accountant_notes to expenditure_reports for review feedback
ALTER TABLE expenditure_reports ADD COLUMN IF NOT EXISTS accountant_notes TEXT;

-- Expand status options to cover the full review lifecycle
-- (original constraint only covered 'submitted','reviewed','approved','flagged_for_investigation')
ALTER TABLE expenditure_reports DROP CONSTRAINT IF EXISTS expenditure_reports_status_check;
ALTER TABLE expenditure_reports
    ADD CONSTRAINT expenditure_reports_status_check
    CHECK (status IN ('pending', 'submitted', 'under_review', 'reviewed', 'approved', 'rejected', 'flagged_for_investigation'));
