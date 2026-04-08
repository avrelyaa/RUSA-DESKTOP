-- Add formal logbook metadata fields required by Astronautics Journal v2.
ALTER TABLE exploration_journals
  ADD COLUMN IF NOT EXISTS mission_location TEXT,
  ADD COLUMN IF NOT EXISTS mission_type VARCHAR(30),
  ADD COLUMN IF NOT EXISTS progress_report TEXT,
  ADD COLUMN IF NOT EXISTS next_steps TEXT,
  ADD COLUMN IF NOT EXISTS approval_status VARCHAR(30) NOT NULL DEFAULT 'draft';

ALTER TABLE exploration_journals
  DROP CONSTRAINT IF EXISTS exploration_journals_mission_type_check;

ALTER TABLE exploration_journals
  ADD CONSTRAINT exploration_journals_mission_type_check
  CHECK (mission_type IS NULL OR mission_type IN ('terraneous', 'interstellar'));

ALTER TABLE exploration_journals
  DROP CONSTRAINT IF EXISTS exploration_journals_approval_status_check;

ALTER TABLE exploration_journals
  ADD CONSTRAINT exploration_journals_approval_status_check
  CHECK (approval_status IN ('draft', 'submitted', 'approved'));

-- Backfill from legacy title/content records.
UPDATE exploration_journals
SET mission_location = COALESCE(NULLIF(mission_location, ''), title)
WHERE mission_location IS NULL;

UPDATE exploration_journals
SET progress_report = COALESCE(progress_report, content)
WHERE progress_report IS NULL;

UPDATE exploration_journals
SET next_steps = COALESCE(next_steps, 'Pending next steps update')
WHERE next_steps IS NULL;

UPDATE exploration_journals ej
SET mission_type = CASE
  WHEN mission_type IS NOT NULL THEN mission_type
  WHEN LOWER(COALESCE(m.mission_type, '')) LIKE '%interstellar%' THEN 'interstellar'
  WHEN LOWER(COALESCE(m.mission_type, '')) LIKE '%sector%' THEN 'interstellar'
  ELSE 'terraneous'
END
FROM missions m
WHERE ej.mission_id = m.id
  AND ej.mission_type IS NULL;

UPDATE exploration_journals
SET mission_type = 'terraneous'
WHERE mission_type IS NULL;

UPDATE exploration_journals ej
SET approval_status = CASE
  WHEN approval_status = 'approved' THEN approval_status
  WHEN m.status = 'completed' THEN 'approved'
  ELSE 'draft'
END
FROM missions m
WHERE ej.mission_id = m.id;

CREATE INDEX IF NOT EXISTS idx_exploration_journals_author_approval_type
  ON exploration_journals(author_id, approval_status, mission_type)
  WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_exploration_journals_author_mission
  ON exploration_journals(author_id, mission_id)
  WHERE deleted_at IS NULL;
