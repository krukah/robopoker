CREATE UNLOGGED TABLE IF NOT EXISTS centroid (
    observation BIGINT PRIMARY KEY,
    abstraction BIGINT,
    street CHAR(1)
);
CREATE UNLOGGED TABLE IF NOT EXISTS distance (
    xor BIGINT PRIMARY KEY,
    distance FLOAT,
    street CHAR(1)
);