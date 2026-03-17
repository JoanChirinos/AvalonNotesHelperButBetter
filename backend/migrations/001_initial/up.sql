-- Known players (global roster across all games)
CREATE TABLE known_players (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE
);

-- Games
CREATE TABLE games (
    id TEXT PRIMARY KEY NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    finished_at TEXT,
    current_quest INTEGER NOT NULL DEFAULT 1 CHECK (current_quest BETWEEN 1 AND 5)
);

-- Players in a game
CREATE TABLE players (
    id TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    known_player_id TEXT NOT NULL REFERENCES known_players(id),
    seat_order INTEGER NOT NULL CHECK (seat_order >= 1),
    -- nullable until end-of-game reveal
    role TEXT,
    UNIQUE (game_id, seat_order),
    UNIQUE (game_id, known_player_id)
);

-- Which roles are in the game (one row per role slot, duplicates allowed)
CREATE TABLE game_roles (
    id TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    role TEXT NOT NULL
);

-- Which modules are enabled for a game
CREATE TABLE game_modules (
    id TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    module TEXT NOT NULL,
    UNIQUE (game_id, module)
);

-- Quests (1-5 per game)
CREATE TABLE quests (
    id TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    quest_number INTEGER NOT NULL CHECK (quest_number BETWEEN 1 AND 5),
    -- null, success, fail
    result TEXT,
    -- anonymous card counts (null = not yet recorded)
    success_count INTEGER,
    fail_count INTEGER,
    magic_count INTEGER,
    good_message_count INTEGER,
    evil_message_count INTEGER,
    UNIQUE (game_id, quest_number)
);

-- Rounds (1-5 per quest, each is a proposal attempt)
CREATE TABLE rounds (
    id TEXT PRIMARY KEY NOT NULL,
    quest_id TEXT NOT NULL REFERENCES quests(id) ON DELETE CASCADE,
    round_number INTEGER NOT NULL CHECK (round_number BETWEEN 1 AND 5),
    leader_player_id TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    -- proposed, approved, rejected
    status TEXT NOT NULL DEFAULT 'proposed',
    UNIQUE (quest_id, round_number)
);

-- Who is on the proposed team for a round
CREATE TABLE round_teams (
    id TEXT PRIMARY KEY NOT NULL,
    round_id TEXT NOT NULL REFERENCES rounds(id) ON DELETE CASCADE,
    player_id TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    UNIQUE (round_id, player_id)
);

-- Votes on a round's proposed team
CREATE TABLE round_votes (
    id TEXT PRIMARY KEY NOT NULL,
    round_id TEXT NOT NULL REFERENCES rounds(id) ON DELETE CASCADE,
    player_id TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    -- approve, reject
    vote TEXT NOT NULL,
    UNIQUE (round_id, player_id)
);

-- Lady of the Lake investigations
CREATE TABLE lady_investigations (
    id TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    quest_id TEXT NOT NULL REFERENCES quests(id) ON DELETE CASCADE,
    investigator_player_id TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    target_player_id TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    -- good, evil (what the investigator CLAIMED)
    claimed_affiliation TEXT NOT NULL
);

-- Who currently holds the Lady of the Lake token
-- Tracked as a chain: the last entry's target becomes the next holder
-- First holder is set during game setup

CREATE TABLE lady_holders (
    id TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    player_id TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    -- order in the chain (0 = initial holder from setup)
    holder_order INTEGER NOT NULL,
    UNIQUE (game_id, holder_order)
);

-- Lancelot switch cards revealed per round
CREATE TABLE lancelot_switches (
    id TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    quest_number INTEGER NOT NULL CHECK (quest_number BETWEEN 1 AND 5),
    -- switch, no_switch
    result TEXT NOT NULL,
    UNIQUE (game_id, quest_number)
);

-- Plot card tracking (distribution and usage only)
CREATE TABLE plot_cards (
    id TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    quest_id TEXT NOT NULL REFERENCES quests(id) ON DELETE CASCADE,
    player_id TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    card_name TEXT NOT NULL,
    -- dealt, used
    status TEXT NOT NULL DEFAULT 'dealt',
    used_on_player_id TEXT REFERENCES players(id) ON DELETE SET NULL
);

-- Assassination phase tracking
CREATE TABLE assassination_attempts (
    id TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    -- 1 = untrustworthy servant snipe, 2 = merlin/messenger snipe
    phase INTEGER NOT NULL,
    -- who is doing the sniping
    sniper_player_id TEXT NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    -- merlin, messengers, untrustworthy_servant
    snipe_type TEXT NOT NULL,
    -- JSON array of player IDs being guessed (1 for merlin/untrustworthy, 2 for messengers)
    target_player_ids TEXT NOT NULL,
    correct INTEGER NOT NULL DEFAULT 0,
    UNIQUE (game_id, phase)
);

-- Free-form notes
CREATE TABLE notes (
    id TEXT PRIMARY KEY NOT NULL,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    quest_id TEXT REFERENCES quests(id) ON DELETE CASCADE,
    player_id TEXT REFERENCES players(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
