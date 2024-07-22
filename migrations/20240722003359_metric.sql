-- Add migration script here
CREATE TABLE IF NOT EXISTS metric (
    xor_pair BIGINT PRIMARY KEY,
    distance NUMERIC,
    street SMALLINT,
);