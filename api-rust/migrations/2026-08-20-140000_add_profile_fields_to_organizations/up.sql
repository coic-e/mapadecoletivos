ALTER TABLE organizations
    -- Gêneros musicais. Array em vez de tabela de junção: a lista é fechada e
    -- pequena, e a única consulta é "quais rolês tocam X".
    ADD COLUMN genres TEXT[] NOT NULL DEFAULT '{}',
    -- Logradouro. Faz sentido para club e bar, que são lugares fixos; festa
    -- itinerante deixa em branco.
    ADD COLUMN address VARCHAR,
    ADD COLUMN instagram VARCHAR,
    ADD COLUMN soundcloud VARCHAR,
    ADD COLUMN bandcamp VARCHAR,
    ADD COLUMN youtube VARCHAR,
    ADD COLUMN spotify VARCHAR,
    ADD COLUMN website VARCHAR,
    -- Coletivo acaba, club fecha. Sem isso o mapa vira cemitério com cara de
    -- catálogo atual.
    ADD COLUMN is_active BOOLEAN NOT NULL DEFAULT TRUE,
    -- Semanal, quinzenal, mensal, sazonal ou pontual.
    ADD COLUMN frequency VARCHAR;

-- O campo social era um link só. O conteúdo vai para instagram, que é onde a
-- cena brasileira de fato está.
UPDATE organizations SET instagram = social WHERE social <> '';

ALTER TABLE organizations DROP COLUMN social;

-- O filtro do mapa usa sobreposição de arrays (genres && ARRAY['Techno']).
CREATE INDEX organizations_genres_idx ON organizations USING GIN (genres);
