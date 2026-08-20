DROP INDEX organizations_status_idx;

ALTER TABLE organizations
    DROP CONSTRAINT organizations_status_check;

ALTER TABLE organizations
    DROP COLUMN status,
    DROP COLUMN rejection_reason,
    DROP COLUMN reviewed_at,
    DROP COLUMN reviewed_by;
