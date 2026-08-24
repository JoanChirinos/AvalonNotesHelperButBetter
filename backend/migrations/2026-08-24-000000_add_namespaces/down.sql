-- Reverse the namespace migration: restore UNIQUE(name) on known_players and
-- drop games.namespace. Collapses all namespaces back to a flat roster.
-- foreign_keys off around the rebuild (see up.sql / metadata.toml).

PRAGMA foreign_keys = OFF;

CREATE TABLE known_players_old (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO known_players_old (id, name)
    SELECT id, name FROM known_players;

DROP TABLE known_players;

ALTER TABLE known_players_old RENAME TO known_players;

ALTER TABLE games DROP COLUMN namespace;

PRAGMA foreign_keys = ON;
