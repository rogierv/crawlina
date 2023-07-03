ALTER TABLE entries ADD COLUMN new_id SERIAL;

UPDATE entries SET new_id = DEFAULT;

ALTER TABLE entries DROP CONSTRAINT entries_pkey;

ALTER TABLE entries RENAME COLUMN id TO entry_id;

ALTER TABLE entries ADD UNIQUE(entry_id);

ALTER TABLE entries RENAME COLUMN new_id TO id;

ALTER TABLE entries ALTER COLUMN read SET DEFAULT 'FALSE';