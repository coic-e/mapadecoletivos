-- Pedido de correção vindo de fora. Editar mesmo, só admin: aqui é a porta de
-- entrada de quem enxerga o erro mas não tem acesso ao painel.
CREATE TABLE edit_requests (
    id SERIAL PRIMARY KEY,
    organization_id INTEGER NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    -- Só os campos que a pessoa quer mudar. Guardado como JSON para o pedido
    -- não virar uma cópia inteira da organização, e para o painel conseguir
    -- mostrar o antes e o depois campo a campo.
    changes JSONB NOT NULL,
    -- Explicação livre: "esse coletivo acabou", "o Instagram mudou".
    message TEXT,
    -- Opcional: por onde responder a quem sugeriu.
    requester_email VARCHAR,
    status VARCHAR NOT NULL DEFAULT 'pending',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    reviewed_at TIMESTAMP,
    reviewed_by INTEGER REFERENCES admins (id) ON DELETE SET NULL
);

ALTER TABLE edit_requests
    ADD CONSTRAINT edit_requests_status_check
    CHECK (status IN ('pending', 'applied', 'rejected'));

CREATE INDEX edit_requests_status_idx ON edit_requests (status);
CREATE INDEX edit_requests_organization_idx ON edit_requests (organization_id);
