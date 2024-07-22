-- Add migration script here
CREATE TABLE IF NOT EXISTS clusters (
    observation BIGINT PRIMARY KEY,
    abstraction BIGINT PRIMARY KEY,
    street SMALLINT,
);