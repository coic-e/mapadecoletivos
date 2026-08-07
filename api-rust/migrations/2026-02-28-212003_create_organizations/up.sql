-- Create organizations table
CREATE TABLE organizations (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    latitude DECIMAL(10, 2) NOT NULL,
    longitude DECIMAL(10, 2) NOT NULL,
    type VARCHAR NOT NULL,
    city VARCHAR NOT NULL,
    uf VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    social VARCHAR NOT NULL,
    about VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
