-- Step 1: Add a new SERIAL column
ALTER TABLE channels ADD COLUMN new_id SERIAL;

-- Step 2: Update the new column with sequential values
UPDATE channels SET new_id = DEFAULT;

-- Step 3: Drop the old UUID column
ALTER TABLE channels DROP COLUMN id;

-- Step 4: (Optional) Rename the new column to match the original column name
ALTER TABLE channels RENAME COLUMN new_id TO id;