-- Moderadores do site. Não há cadastro público: contas são criadas pelo
-- binário create_admin, rodado por quem tem acesso ao servidor.
CREATE TABLE admins (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    password_hash VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
