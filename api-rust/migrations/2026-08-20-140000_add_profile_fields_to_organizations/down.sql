ALTER TABLE organizations ADD COLUMN social VARCHAR NOT NULL DEFAULT '';

UPDATE organizations SET social = COALESCE(instagram, '');

DROP INDEX organizations_genres_idx;

ALTER TABLE organizations
    DROP COLUMN genres,
    DROP COLUMN address,
    DROP COLUMN instagram,
    DROP COLUMN soundcloud,
    DROP COLUMN bandcamp,
    DROP COLUMN youtube,
    DROP COLUMN spotify,
    DROP COLUMN website,
    DROP COLUMN is_active,
    DROP COLUMN frequency;
