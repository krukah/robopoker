-- Add migration script here
CREATE TABLE distance (
    xor_pair BIGINT PRIMARY KEY,
    distance NUMERIC,
    street SMALLINT,
);