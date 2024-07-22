-- Add migration script here
CREATE TABLE clusters (
    observation BIGINT PRIMARY KEY,
    abstraction BIGINT PRIMARY KEY,
    street SMALLINT,
);