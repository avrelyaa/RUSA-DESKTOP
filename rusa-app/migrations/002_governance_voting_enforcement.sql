-- 002_governance_voting_enforcement.sql
-- Enforces that all budget requests and test proposals are resolved exclusively
-- through Director votes in the Governance voting module.

-- 1a. Add `reasoning` column to vote_ballots, defaulting existing rows to empty string,
--     then drop the default so future inserts must supply a value explicitly.
ALTER TABLE vote_ballots ADD COLUMN IF NOT EXISTS reasoning TEXT NOT NULL DEFAULT '';
-- Set existing rows to a placeholder value before dropping the default
UPDATE vote_ballots SET reasoning = '' WHERE reasoning IS NULL;
-- Remove the default so future inserts must supply a value
ALTER TABLE vote_ballots ALTER COLUMN reasoning DROP DEFAULT;

-- 1b. Add polymorphic link columns to votes table so each vote knows
--     which request it originated from
ALTER TABLE votes ADD COLUMN IF NOT EXISTS target_id UUID;
ALTER TABLE votes ADD COLUMN IF NOT EXISTS target_table VARCHAR(100);

-- 1c. Ensure budget_requests has a vote_id FK (nullable)
ALTER TABLE budget_requests ADD COLUMN IF NOT EXISTS vote_id UUID REFERENCES votes(id);

-- 1d. Ensure test_proposals has a vote_id FK
ALTER TABLE test_proposals ADD COLUMN IF NOT EXISTS vote_id UUID REFERENCES votes(id);

-- 1e. Ensure the votes table has the snooze_count column
ALTER TABLE votes ADD COLUMN IF NOT EXISTS snooze_count INTEGER NOT NULL DEFAULT 0;
