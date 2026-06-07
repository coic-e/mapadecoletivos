-- Create images table
CREATE TABLE images (
    id SERIAL PRIMARY KEY,
    path VARCHAR NOT NULL,
    organization_id INTEGER NOT NULL,
    CONSTRAINT fk_organization 
        FOREIGN KEY (organization_id) 
        REFERENCES organizations(id) 
        ON UPDATE CASCADE 
        ON DELETE CASCADE
);
