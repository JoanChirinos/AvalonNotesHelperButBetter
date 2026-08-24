-- Add per-namespace isolation. All existing data belongs to namespace 'SGW'.
--
-- `games` gets a namespace column (root of every game's data; everything else
-- keys off game_id and is transitively namespaced).
--
-- `known_players` (the shared roster) must change its uniqueness from UNIQUE(name)
-- to UNIQUE(namespace, name) so different namespaces can each have e.g. a "Joan".
-- SQLite can't drop a table-level constraint in place, so we rebuild the table.
-- `players` has a FK to known_players(id); defer_foreign_keys lets the drop/rename
-- happen inside diesel's migration transaction (ids are preserved, so FKs hold at
-- commit). defer_foreign_keys (unlike foreign_keys) is settable inside a transaction.

PRAGMA defer_foreign_keys = ON;

ALTER TABLE games ADD COLUMN namespace TEXT NOT NULL DEFAULT 'SGW';

CREATE TABLE known_players_new (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    namespace TEXT NOT NULL DEFAULT 'SGW',
    UNIQUE (namespace, name)
);

INSERT INTO known_players_new (id, name, namespace)
    SELECT id, name, 'SGW' FROM known_players;

DROP TABLE known_players;

ALTER TABLE known_players_new RENAME TO known_players;
