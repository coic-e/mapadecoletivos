-- Cadastro novo entra como 'pending' e só aparece no site depois que um admin
-- aprova. O CHECK segura os três estados possíveis no próprio banco.
ALTER TABLE organizations
    ADD COLUMN status VARCHAR NOT NULL DEFAULT 'pending',
    ADD COLUMN rejection_reason TEXT,
    ADD COLUMN reviewed_at TIMESTAMP,
    ADD COLUMN reviewed_by INTEGER REFERENCES admins (id) ON DELETE SET NULL;

ALTER TABLE organizations
    ADD CONSTRAINT organizations_status_check
    CHECK (status IN ('pending', 'approved', 'rejected'));

-- As duas leituras quentes filtram por status: o mapa público e a fila.
CREATE INDEX organizations_status_idx ON organizations (status);
