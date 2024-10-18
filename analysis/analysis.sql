-- Create and prepare abstraction table
CREATE TABLE IF NOT EXISTS turn_abs (obs BIGINT, abs BIGINT);
TRUNCATE TABLE turn_abs;
ALTER TABLE turn_abs
SET UNLOGGED;
DROP INDEX IF EXISTS idx_turn_abs_observation;
COPY turn_abs (obs, abs)
FROM '/Users/krukah/Code/robopoker/turn.abstraction.pgcopy' WITH (FORMAT BINARY);
ALTER TABLE turn_abs
SET LOGGED;
CREATE INDEX IF NOT EXISTS idx_turn_abs_observation ON turn_abs (obs);
-- Create and prepare metric table
CREATE TABLE IF NOT EXISTS turn_met (xab BIGINT, dst REAL);
TRUNCATE TABLE turn_met;
ALTER TABLE turn_met
SET UNLOGGED;
DROP INDEX IF EXISTS idx_turn_xab_abstraction;
COPY turn_met (xab, dst)
FROM '/Users/krukah/Code/robopoker/preflop.metric.pgcopy' WITH (FORMAT BINARY);
ALTER TABLE turn_met
SET LOGGED;
CREATE INDEX IF NOT EXISTS idx_turn_xab_abstraction ON turn_met (xab);