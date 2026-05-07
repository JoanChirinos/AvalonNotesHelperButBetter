use diesel::prelude::*;

use crate::db::DbPool;
use crate::models::*;
use crate::schema;

/// Load the full game state for WebSocket broadcast
pub fn load_full_game_state(
    pool: &DbPool,
    game_id: &str,
) -> Result<FullGameState, diesel::result::Error> {
    let mut conn = pool.get().map_err(|_| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::Unknown,
        Box::new("Connection pool exhausted".to_string()),
    ))?;

    let game = schema::games::table
        .find(game_id)
        .first::<Game>(&mut conn)?;

    let players = schema::players::table
        .filter(schema::players::game_id.eq(game_id))
        .order(schema::players::seat_order.asc())
        .load::<Player>(&mut conn)?;

    // Fetch known players referenced by this game's players
    let kp_ids: Vec<&str> = players.iter().map(|p| p.known_player_id.as_str()).collect();
    let known_players = schema::known_players::table
        .filter(schema::known_players::id.eq_any(&kp_ids))
        .load::<KnownPlayer>(&mut conn)?;

    let roles = schema::game_roles::table
        .filter(schema::game_roles::game_id.eq(game_id))
        .load::<GameRole>(&mut conn)?;

    let modules = schema::game_modules::table
        .filter(schema::game_modules::game_id.eq(game_id))
        .load::<GameModule>(&mut conn)?;

    let quests_rows = schema::quests::table
        .filter(schema::quests::game_id.eq(game_id))
        .order(schema::quests::quest_number.asc())
        .load::<Quest>(&mut conn)?;

    let quest_ids: Vec<&str> = quests_rows.iter().map(|q| q.id.as_str()).collect();

    let all_rounds = schema::rounds::table
        .filter(schema::rounds::quest_id.eq_any(&quest_ids))
        .order(schema::rounds::round_number.asc())
        .load::<Round>(&mut conn)?;

    let round_ids: Vec<&str> = all_rounds.iter().map(|r| r.id.as_str()).collect();

    let all_teams = schema::round_teams::table
        .filter(schema::round_teams::round_id.eq_any(&round_ids))
        .load::<RoundTeam>(&mut conn)?;

    let all_votes = schema::round_votes::table
        .filter(schema::round_votes::round_id.eq_any(&round_ids))
        .load::<RoundVote>(&mut conn)?;

    // Build nested quest states
    let quests = quests_rows
        .into_iter()
        .map(|quest| {
            let rounds: Vec<RoundState> = all_rounds
                .iter()
                .filter(|r| r.quest_id == quest.id)
                .map(|round| {
                    let team = all_teams
                        .iter()
                        .filter(|t| t.round_id == round.id)
                        .cloned()
                        .collect();
                    let votes = all_votes
                        .iter()
                        .filter(|v| v.round_id == round.id)
                        .cloned()
                        .collect();
                    RoundState {
                        round: round.clone(),
                        team,
                        votes,
                    }
                })
                .collect();

            QuestState { quest, rounds }
        })
        .collect();

    let lady_holders = schema::lady_holders::table
        .filter(schema::lady_holders::game_id.eq(game_id))
        .order(schema::lady_holders::holder_order.asc())
        .load::<LadyHolder>(&mut conn)?;

    let lady_investigations = schema::lady_investigations::table
        .filter(schema::lady_investigations::game_id.eq(game_id))
        .load::<LadyInvestigation>(&mut conn)?;

    let lancelot_switches = schema::lancelot_switches::table
        .filter(schema::lancelot_switches::game_id.eq(game_id))
        .order(schema::lancelot_switches::quest_number.asc())
        .load::<LancelotSwitch>(&mut conn)?;

    let plot_cards = schema::plot_cards::table
        .filter(schema::plot_cards::game_id.eq(game_id))
        .load::<PlotCard>(&mut conn)?;

    let assassination_attempts = schema::assassination_attempts::table
        .filter(schema::assassination_attempts::game_id.eq(game_id))
        .order(schema::assassination_attempts::phase.asc())
        .load::<AssassinationAttempt>(&mut conn)?;

    let notes = schema::notes::table
        .filter(schema::notes::game_id.eq(game_id))
        .order(schema::notes::created_at.asc())
        .load::<Note>(&mut conn)?;

    Ok(FullGameState {
        game,
        players,
        known_players,
        roles,
        modules,
        quests,
        lady_holders,
        lady_investigations,
        lancelot_switches,
        plot_cards,
        assassination_attempts,
        notes,
    })
}
