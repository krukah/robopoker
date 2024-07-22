-- Add migration script here
CREATE TABLE IF NOT EXISTS metric (
    xor BIGINT PRIMARY KEY,
    distance FLOAT,
    street SMALLINT
);