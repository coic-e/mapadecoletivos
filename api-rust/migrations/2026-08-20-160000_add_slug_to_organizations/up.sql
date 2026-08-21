-- URL com o nome do rolê em vez do id: /raves/bunker-034. Vale mais para busca
-- e é o que a pessoa reconhece ao ver o link compartilhado.
ALTER TABLE organizations ADD COLUMN slug VARCHAR;

-- Backfill do que já existe. A geração de verdade vive no Rust (slugify);
-- aqui é só o suficiente para a coluna poder virar NOT NULL. translate() no
-- lugar de unaccent() para a migração não depender de extensão do Postgres.
UPDATE organizations
SET slug = trim(both '-' from regexp_replace(
        lower(translate(
            name,
            'áàâãäéèêëíìîïóòôõöúùûüçÁÀÂÃÄÉÈÊËÍÌÎÏÓÒÔÕÖÚÙÛÜÇñÑ',
            'aaaaaeeeeiiiiooooouuuucAAAAAEEEEIIIIOOOOOUUUUCnN'
        )),
        '[^a-zA-Z0-9]+', '-', 'g'
    )) || '-' || id
WHERE slug IS NULL;

ALTER TABLE organizations ALTER COLUMN slug SET NOT NULL;

CREATE UNIQUE INDEX organizations_slug_idx ON organizations (slug);
