-- Add migration script here
CREATE TABLE IF NOT EXISTS cluster (
    observation BIGINT PRIMARY KEY,
    abstraction BIGINT,
    street SMALLINT
);