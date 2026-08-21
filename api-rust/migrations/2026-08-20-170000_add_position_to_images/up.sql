-- Ordem das fotos, e por consequência a capa: menor posição é a que aparece
-- primeiro. Antes disso a capa era a primeira foto por acaso de insert.
ALTER TABLE images ADD COLUMN position INTEGER NOT NULL DEFAULT 0;

CREATE INDEX images_organization_position_idx ON images (organization_id, position);
