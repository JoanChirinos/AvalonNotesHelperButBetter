// @generated automatically by Diesel CLI.

diesel::table! {
    assassination_attempts (id) {
        id -> Text,
        game_id -> Text,
        phase -> Integer,
        sniper_player_id -> Text,
        snipe_type -> Text,
        target_player_ids -> Text,
        correct -> Integer,
    }
}

diesel::table! {
    game_modules (id) {
        id -> Text,
        game_id -> Text,
        module -> Text,
    }
}

diesel::table! {
    game_roles (id) {
        id -> Text,
        game_id -> Text,
        role -> Text,
    }
}

diesel::table! {
    games (id) {
        id -> Text,
        created_at -> Text,
        finished_at -> Nullable<Text>,
        deleted_at -> Nullable<Text>,
        current_quest -> Integer,
        namespace -> Text,
    }
}

diesel::table! {
    known_players (id) {
        id -> Text,
        name -> Text,
        namespace -> Text,
    }
}

diesel::table! {
    lady_holders (id) {
        id -> Text,
        game_id -> Text,
        player_id -> Text,
        holder_order -> Integer,
    }
}

diesel::table! {
    lady_investigations (id) {
        id -> Text,
        game_id -> Text,
        quest_id -> Text,
        investigator_player_id -> Text,
        target_player_id -> Text,
        claimed_affiliation -> Text,
    }
}

diesel::table! {
    lancelot_switches (id) {
        id -> Text,
        game_id -> Text,
        quest_number -> Integer,
        result -> Text,
    }
}

diesel::table! {
    notes (id) {
        id -> Text,
        game_id -> Text,
        quest_id -> Nullable<Text>,
        player_id -> Nullable<Text>,
        content -> Text,
        created_at -> Text,
    }
}

diesel::table! {
    players (id) {
        id -> Text,
        game_id -> Text,
        known_player_id -> Text,
        seat_order -> Integer,
        role -> Nullable<Text>,
    }
}

diesel::table! {
    plot_cards (id) {
        id -> Text,
        game_id -> Text,
        quest_id -> Text,
        player_id -> Text,
        card_name -> Text,
        status -> Text,
        used_on_player_id -> Nullable<Text>,
    }
}

diesel::table! {
    quests (id) {
        id -> Text,
        game_id -> Text,
        quest_number -> Integer,
        result -> Nullable<Text>,
        success_count -> Nullable<Integer>,
        fail_count -> Nullable<Integer>,
        magic_count -> Nullable<Integer>,
        good_message_count -> Nullable<Integer>,
        evil_message_count -> Nullable<Integer>,
    }
}

diesel::table! {
    round_teams (id) {
        id -> Text,
        round_id -> Text,
        player_id -> Text,
    }
}

diesel::table! {
    round_votes (id) {
        id -> Text,
        round_id -> Text,
        player_id -> Text,
        vote -> Text,
    }
}

diesel::table! {
    rounds (id) {
        id -> Text,
        quest_id -> Text,
        round_number -> Integer,
        leader_player_id -> Text,
        status -> Text,
    }
}

diesel::joinable!(assassination_attempts -> games (game_id));
diesel::joinable!(assassination_attempts -> players (sniper_player_id));
diesel::joinable!(game_modules -> games (game_id));
diesel::joinable!(game_roles -> games (game_id));
diesel::joinable!(lady_holders -> games (game_id));
diesel::joinable!(lady_holders -> players (player_id));
diesel::joinable!(lady_investigations -> games (game_id));
diesel::joinable!(lady_investigations -> quests (quest_id));
diesel::joinable!(lancelot_switches -> games (game_id));
diesel::joinable!(notes -> games (game_id));
diesel::joinable!(notes -> players (player_id));
diesel::joinable!(notes -> quests (quest_id));
diesel::joinable!(players -> games (game_id));
diesel::joinable!(players -> known_players (known_player_id));
diesel::joinable!(plot_cards -> games (game_id));
diesel::joinable!(plot_cards -> quests (quest_id));
diesel::joinable!(quests -> games (game_id));
diesel::joinable!(round_teams -> players (player_id));
diesel::joinable!(round_teams -> rounds (round_id));
diesel::joinable!(round_votes -> players (player_id));
diesel::joinable!(round_votes -> rounds (round_id));
diesel::joinable!(rounds -> players (leader_player_id));
diesel::joinable!(rounds -> quests (quest_id));

diesel::allow_tables_to_appear_in_same_query!(
    assassination_attempts,
    game_modules,
    game_roles,
    games,
    known_players,
    lady_holders,
    lady_investigations,
    lancelot_switches,
    notes,
    players,
    plot_cards,
    quests,
    round_teams,
    round_votes,
    rounds,
);
