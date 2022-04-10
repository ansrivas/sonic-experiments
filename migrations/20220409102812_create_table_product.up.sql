-- Add up migration script here

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS product (
    id BIGSERIAL PRIMARY KEY,
    details text NOT NULL,
    object_id uuid NOT NULL UNIQUE
);