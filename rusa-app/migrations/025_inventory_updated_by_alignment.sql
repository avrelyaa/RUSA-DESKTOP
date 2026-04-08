-- Align inventory tables with backend inserts that use updated_by
-- Backfill from legacy logged_by to preserve existing attribution data.

ALTER TABLE medical_inventory
ADD COLUMN IF NOT EXISTS updated_by UUID REFERENCES users(id);

UPDATE medical_inventory
SET updated_by = logged_by
WHERE updated_by IS NULL;

ALTER TABLE medical_inventory
ALTER COLUMN updated_by SET NOT NULL;

ALTER TABLE sanitary_inventory
ADD COLUMN IF NOT EXISTS updated_by UUID REFERENCES users(id);

UPDATE sanitary_inventory
SET updated_by = logged_by
WHERE updated_by IS NULL;

ALTER TABLE sanitary_inventory
ALTER COLUMN updated_by SET NOT NULL;

ALTER TABLE settlement_inventory
ADD COLUMN IF NOT EXISTS updated_by UUID REFERENCES users(id);

UPDATE settlement_inventory
SET updated_by = logged_by
WHERE updated_by IS NULL;

ALTER TABLE settlement_inventory
ALTER COLUMN updated_by SET NOT NULL;
