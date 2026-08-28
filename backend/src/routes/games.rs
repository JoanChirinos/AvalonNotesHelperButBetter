use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use diesel::prelude::*;

use crate::models::*;
use crate::queries;
use crate::schema;
use crate::state::AppState;
use crate::types::*;

type ApiResult<T> = Result<T, (StatusCode, String)>;

fn db_err(e: diesel::result::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

fn get_conn(pool: &crate::db::DbPool) -> Result<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::SqliteConnection>>, (StatusCode, String)> {
    pool.get().map_err(|e| (StatusCode::SERVICE_UNAVAILABLE, format!("DB pool exhausted: {e}")))
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Broadcast full game state to all WebSocket clients, returning the loaded state
async fn broadcast(state: &AppState, game_id: &str) -> Result<FullGameState, (StatusCode, String)> {
    let game_state = queries::load_full_game_state(&state.db, game_id).map_err(db_err)?;
    let msg = serde_json::json!({ "type": "game_state", "data": game_state });
    let tx = state.get_channel(game_id).await;
    let _ = tx.send(msg.to_string());
    Ok(game_state)
}

// ── Games ──

pub async fn create_game(
    State(state): State<AppState>,
    Json(req): Json<CreateGameRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;
    let game_id = new_id();

    let namespace = req.namespace.clone();

    conn.transaction(|conn| {
        diesel::insert_into(schema::games::table)
            .values(&NewGame {
                id: game_id.clone(),
                current_quest: 1,
                namespace: namespace.clone(),
            })
            .execute(conn)?;

        for (i, name) in req.player_names.iter().enumerate() {
            // Reuse an existing roster entry for this (namespace, name), else create
            // one. The roster is shared within a namespace, so re-submitting known
            // names (e.g. the "play again" duplicate) must not violate the unique key.
            let existing: Option<KnownPlayer> = schema::known_players::table
                .filter(schema::known_players::namespace.eq(&namespace))
                .filter(schema::known_players::name.eq(name))
                .first(conn)
                .optional()?;
            let kp_id = if let Some(kp) = existing {
                kp.id
            } else {
                let id = new_id();
                diesel::insert_into(schema::known_players::table)
                    .values(&NewKnownPlayer {
                        id: id.clone(),
                        name: name.clone(),
                        namespace: namespace.clone(),
                    })
                    .execute(conn)?;
                id
            };

            diesel::insert_into(schema::players::table)
                .values(&NewPlayer {
                    id: new_id(),
                    game_id: game_id.clone(),
                    known_player_id: kp_id,
                    seat_order: (i + 1) as i32,
                })
                .execute(conn)?;
        }

        for role in &req.roles {
            diesel::insert_into(schema::game_roles::table)
                .values(&NewGameRole {
                    id: new_id(),
                    game_id: game_id.clone(),
                    role: *role,
                })
                .execute(conn)?;
        }

        for module in &req.modules {
            diesel::insert_into(schema::game_modules::table)
                .values(&NewGameModule {
                    id: new_id(),
                    game_id: game_id.clone(),
                    module: *module,
                })
                .execute(conn)?;
        }

        for q in 1..=5 {
            diesel::insert_into(schema::quests::table)
                .values(&NewQuest {
                    id: new_id(),
                    game_id: game_id.clone(),
                    quest_number: q,
                })
                .execute(conn)?;
        }

        if req.modules.contains(&Module::LadyOfTheLake) {
            if let Some(idx) = req.lady_holder_player_index {
                let players = schema::players::table
                    .filter(schema::players::game_id.eq(&game_id))
                    .order(schema::players::seat_order.asc())
                    .load::<Player>(conn)?;

                if let Some(player) = players.get(idx) {
                    diesel::insert_into(schema::lady_holders::table)
                        .values(&NewLadyHolder {
                            id: new_id(),
                            game_id: game_id.clone(),
                            player_id: player.id.clone(),
                            holder_order: 0,
                        })
                        .execute(conn)?;
                }
            }
        }

        Ok(())
    }).map_err(db_err)?;

    let game_state = queries::load_full_game_state(&state.db, &game_id).map_err(db_err)?;
    Ok((StatusCode::CREATED, Json(game_state)))
}

pub async fn list_games(
    State(state): State<AppState>,
    Query(q): Query<NamespaceQuery>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;
    let games = schema::games::table
        .filter(schema::games::deleted_at.is_null())
        .filter(schema::games::namespace.eq(&q.namespace))
        .order(schema::games::created_at.desc())
        .load::<Game>(&mut conn)
        .map_err(db_err)?;

    let mut summaries = Vec::new();
    for game in games {
        let players_data: Vec<(String, Option<String>)> = schema::players::table
            .inner_join(schema::known_players::table)
            .filter(schema::players::game_id.eq(&game.id))
            .order(schema::players::seat_order.asc())
            .select((schema::known_players::name, schema::players::role))
            .load(&mut conn)
            .map_err(db_err)?;

        let names: Vec<String> = players_data.iter().map(|(n, _)| n.clone()).collect();
        let roles: Vec<Option<String>> = players_data.iter().map(|(_, r)| r.clone()).collect();

        let quest_ids: Vec<String> = schema::quests::table
            .filter(schema::quests::game_id.eq(&game.id))
            .select(schema::quests::id)
            .load(&mut conn)
            .map_err(db_err)?;

        let round_count: i64 = schema::rounds::table
            .filter(schema::rounds::quest_id.eq_any(&quest_ids))
            .count()
            .get_result(&mut conn)
            .map_err(db_err)?;

        let game_id_ref = game.id.clone();
        summaries.push(GameSummary {
            game,
            player_names: names,
            player_roles: roles,
            has_started: round_count > 0,
            result: {
                let quest_results: Vec<Option<String>> = schema::quests::table
                    .filter(schema::quests::game_id.eq(&game_id_ref))
                    .select(schema::quests::result)
                    .load(&mut conn)
                    .unwrap_or_default();
                let successes = quest_results.iter().filter(|r| r.as_deref() == Some("success")).count() as i64;
                let fails = quest_results.iter().filter(|r| r.as_deref() == Some("fail")).count() as i64;
                if fails >= 3 {
                    Some("evil".to_string())
                } else if successes >= 3 {
                    // Check assassination
                    let snipe = schema::assassination_attempts::table
                        .filter(schema::assassination_attempts::game_id.eq(&game_id_ref))
                        .filter(schema::assassination_attempts::phase.eq(2))
                        .first::<AssassinationAttempt>(&mut conn)
                        .ok();
                    match snipe {
                        Some(a) if a.correct == 1 => Some("evil".to_string()),
                        Some(_) => Some("good".to_string()),
                        None => {
                            let has_target: bool = schema::game_roles::table
                                .filter(schema::game_roles::game_id.eq(&game_id_ref))
                                .filter(
                                    schema::game_roles::role.eq("merlin")
                                        .or(schema::game_roles::role.eq("senior_messenger"))
                                )
                                .count()
                                .get_result::<i64>(&mut conn)
                                .unwrap_or(0) > 0;
                            if has_target {
                                None
                            } else {
                                Some("good".to_string())
                            }
                        }
                    }
                } else {
                    None
                }
            },
        });
    }

    Ok(Json(summaries))
}

pub async fn get_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let game_state = queries::load_full_game_state(&state.db, &game_id).map_err(db_err)?;
    Ok(Json(game_state))
}

pub async fn update_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<UpdateGameRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    if let Some(ref finished_at) = req.finished_at {
        diesel::update(schema::games::table.find(&game_id))
            .set(schema::games::finished_at.eq(finished_at))
            .execute(&mut conn)
            .map_err(db_err)?;
    }
    if let Some(current_quest) = req.current_quest {
        diesel::update(schema::games::table.find(&game_id))
            .set(schema::games::current_quest.eq(current_quest))
            .execute(&mut conn)
            .map_err(db_err)?;
    }

    let game_state = broadcast(&state, &game_id).await?;
    Ok(Json(game_state))
}

pub async fn delete_game(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;
    diesel::update(schema::games::table.find(&game_id))
        .set(schema::games::deleted_at.eq(diesel::dsl::sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("strftime('%Y-%m-%dT%H:%M:%SZ', 'now')")))
        .execute(&mut conn)
        .map_err(db_err)?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Players ──

/// Full state of every finished game in a namespace, for the stats dashboard.
/// One call so the frontend can aggregate with its existing derivation logic.
pub async fn list_full_games(
    State(state): State<AppState>,
    Query(q): Query<NamespaceQuery>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;
    let game_ids: Vec<String> = schema::games::table
        .filter(schema::games::deleted_at.is_null())
        .filter(schema::games::finished_at.is_not_null())
        .filter(schema::games::namespace.eq(&q.namespace))
        .order(schema::games::created_at.asc())
        .select(schema::games::id)
        .load(&mut conn)
        .map_err(db_err)?;
    drop(conn);

    let mut out = Vec::with_capacity(game_ids.len());
    for id in game_ids {
        out.push(queries::load_full_game_state(&state.db, &id).map_err(db_err)?);
    }
    Ok(Json(out))
}

pub async fn list_namespaces(
    State(state): State<AppState>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;
    let namespaces: Vec<String> = schema::games::table
        .select(schema::games::namespace)
        .distinct()
        .order(schema::games::namespace.asc())
        .load::<String>(&mut conn)
        .map_err(db_err)?;
    Ok(Json(namespaces))
}

pub async fn list_known_players(
    State(state): State<AppState>,
    Query(q): Query<NamespaceQuery>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;
    let players = schema::known_players::table
        .filter(schema::known_players::namespace.eq(&q.namespace))
        .order(schema::known_players::name.asc())
        .load::<KnownPlayer>(&mut conn)
        .map_err(db_err)?;
    Ok(Json(players))
}

pub async fn add_player(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<AddPlayerRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    // The known-player roster is scoped to the game's namespace.
    let namespace: String = schema::games::table
        .find(&game_id)
        .select(schema::games::namespace)
        .first(&mut conn)
        .map_err(db_err)?;

    // Resolve or create known player
    let kp_id = if let Some(id) = req.known_player_id {
        id
    } else if let Some(name) = req.name {
        // Look up existing known_player by (namespace, name), or create new
        let existing: Option<KnownPlayer> = schema::known_players::table
            .filter(schema::known_players::namespace.eq(&namespace))
            .filter(schema::known_players::name.eq(&name))
            .first(&mut conn)
            .ok();
        if let Some(kp) = existing {
            kp.id
        } else {
            let id = new_id();
            diesel::insert_into(schema::known_players::table)
                .values(&NewKnownPlayer {
                    id: id.clone(),
                    name,
                    namespace: namespace.clone(),
                })
                .execute(&mut conn)
                .map_err(db_err)?;
            id
        }
    } else {
        return Err((StatusCode::BAD_REQUEST, "Provide known_player_id or name".to_string()));
    };

    // Next seat order
    let max_seat: Option<i32> = schema::players::table
        .filter(schema::players::game_id.eq(&game_id))
        .select(diesel::dsl::max(schema::players::seat_order))
        .first(&mut conn)
        .map_err(db_err)?;

    diesel::insert_into(schema::players::table)
        .values(&NewPlayer {
            id: new_id(),
            game_id: game_id.clone(),
            known_player_id: kp_id,
            seat_order: max_seat.unwrap_or(0) + 1,
        })
        .execute(&mut conn)
        .map_err(db_err)?;

    let game_state = broadcast(&state, &game_id).await?;
    Ok((StatusCode::CREATED, Json(game_state)))
}

pub async fn reorder_players(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<ReorderPlayersRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    conn.transaction(|conn| {
        for (i, pid) in req.player_ids.iter().enumerate() {
            diesel::update(schema::players::table.find(pid))
                .set(schema::players::seat_order.eq((i as i32 + 1) + 1000))
                .execute(conn)?;
        }
        for (i, pid) in req.player_ids.iter().enumerate() {
            diesel::update(schema::players::table.find(pid))
                .set(schema::players::seat_order.eq(i as i32 + 1))
                .execute(conn)?;
        }
        Ok(())
    }).map_err(db_err)?;

    let game_state = broadcast(&state, &game_id).await?;
    Ok(Json(game_state))
}

pub async fn update_player(
    State(state): State<AppState>,
    Path((game_id, player_id)): Path<(String, String)>,
    Json(req): Json<UpdatePlayerRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    if let Some(seat_order) = req.seat_order {
        diesel::update(schema::players::table.find(&player_id))
            .set(schema::players::seat_order.eq(seat_order))
            .execute(&mut conn)
            .map_err(db_err)?;
    }
    if let Some(role) = req.role {
        diesel::update(schema::players::table.find(&player_id))
            .set(schema::players::role.eq(role))
            .execute(&mut conn)
            .map_err(db_err)?;
    } else if req.clear_role {
        diesel::update(schema::players::table.find(&player_id))
            .set(schema::players::role.eq(None::<Role>))
            .execute(&mut conn)
            .map_err(db_err)?;
    }

    let game_state = broadcast(&state, &game_id).await?;
    Ok(Json(game_state))
}

pub async fn delete_player(
    State(state): State<AppState>,
    Path((game_id, player_id)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    diesel::delete(schema::players::table.find(&player_id))
        .execute(&mut conn)
        .map_err(db_err)?;

    let _ = broadcast(&state, &game_id).await;
    Ok(StatusCode::NO_CONTENT)
}

// ── Roles & Modules ──

pub async fn add_role(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<AddRoleRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    diesel::insert_into(schema::game_roles::table)
        .values(&NewGameRole {
            id: new_id(),
            game_id: game_id.clone(),
            role: req.role,
        })
        .execute(&mut conn)
        .map_err(db_err)?;

    let game_state = broadcast(&state, &game_id).await?;
    Ok((StatusCode::CREATED, Json(game_state)))
}

pub async fn delete_role(
    State(state): State<AppState>,
    Path((game_id, role_id)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    diesel::delete(schema::game_roles::table.find(&role_id))
        .execute(&mut conn)
        .map_err(db_err)?;

    let game_state = broadcast(&state, &game_id).await?;
    Ok(Json(game_state))
}

pub async fn add_module(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<AddModuleRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    diesel::insert_into(schema::game_modules::table)
        .values(&NewGameModule {
            id: new_id(),
            game_id: game_id.clone(),
            module: req.module,
        })
        .execute(&mut conn)
        .map_err(db_err)?;

    let game_state = broadcast(&state, &game_id).await?;
    Ok((StatusCode::CREATED, Json(game_state)))
}

pub async fn delete_module(
    State(state): State<AppState>,
    Path((game_id, module_id)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    diesel::delete(schema::game_modules::table.find(&module_id))
        .execute(&mut conn)
        .map_err(db_err)?;

    let game_state = broadcast(&state, &game_id).await?;
    Ok(Json(game_state))
}

// ── Quests ──

pub async fn update_quest(
    State(state): State<AppState>,
    Path((game_id, quest_id)): Path<(String, String)>,
    Json(req): Json<UpdateQuestRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    if let Some(result) = req.result {
        diesel::update(schema::quests::table.find(&quest_id))
            .set(schema::quests::result.eq(result))
            .execute(&mut conn)
            .map_err(db_err)?;
    }
    if let Some(c) = req.success_count {
        diesel::update(schema::quests::table.find(&quest_id))
            .set(schema::quests::success_count.eq(c))
            .execute(&mut conn)
            .map_err(db_err)?;
    }
    if let Some(c) = req.fail_count {
        diesel::update(schema::quests::table.find(&quest_id))
            .set(schema::quests::fail_count.eq(c))
            .execute(&mut conn)
            .map_err(db_err)?;
    }
    if let Some(c) = req.magic_count {
        diesel::update(schema::quests::table.find(&quest_id))
            .set(schema::quests::magic_count.eq(c))
            .execute(&mut conn)
            .map_err(db_err)?;
    }
    if let Some(c) = req.good_message_count {
        diesel::update(schema::quests::table.find(&quest_id))
            .set(schema::quests::good_message_count.eq(c))
            .execute(&mut conn)
            .map_err(db_err)?;
    }
    if let Some(c) = req.evil_message_count {
        diesel::update(schema::quests::table.find(&quest_id))
            .set(schema::quests::evil_message_count.eq(c))
            .execute(&mut conn)
            .map_err(db_err)?;
    }

    let game_state = broadcast(&state, &game_id).await?;
    Ok(Json(game_state))
}

// ── Rounds ──

pub async fn create_round(
    State(state): State<AppState>,
    Path((game_id, quest_id)): Path<(String, String)>,
    Json(req): Json<CreateRoundRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    conn.transaction(|conn| {
        let existing: i64 = schema::rounds::table
            .filter(schema::rounds::quest_id.eq(&quest_id))
            .count()
            .get_result(conn)?;

        let round_id = new_id();
        diesel::insert_into(schema::rounds::table)
            .values(&NewRound {
                id: round_id.clone(),
                quest_id,
                round_number: (existing + 1) as i32,
                leader_player_id: req.leader_player_id,
                status: RoundStatus::Proposed,
            })
            .execute(conn)?;

        for pid in &req.team_player_ids {
            diesel::insert_into(schema::round_teams::table)
                .values(&NewRoundTeam {
                    id: new_id(),
                    round_id: round_id.clone(),
                    player_id: pid.clone(),
                })
                .execute(conn)?;
        }

        Ok(())
    }).map_err(db_err)?;

    let game_state = broadcast(&state, &game_id).await?;
    Ok((StatusCode::CREATED, Json(game_state)))
}

pub async fn update_round(
    State(state): State<AppState>,
    Path((game_id, round_id)): Path<(String, String)>,
    Json(req): Json<UpdateRoundRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    conn.transaction(|conn| {
        if let Some(ref leader) = req.leader_player_id {
            diesel::update(schema::rounds::table.find(&round_id))
                .set(schema::rounds::leader_player_id.eq(leader))
                .execute(conn)?;
        }
        if let Some(status) = req.status {
            diesel::update(schema::rounds::table.find(&round_id))
                .set(schema::rounds::status.eq(status))
                .execute(conn)?;
        }
        if let Some(ref team_ids) = req.team_player_ids {
            diesel::delete(
                schema::round_teams::table.filter(schema::round_teams::round_id.eq(&round_id)),
            )
            .execute(conn)?;

            for pid in team_ids {
                diesel::insert_into(schema::round_teams::table)
                    .values(&NewRoundTeam {
                        id: new_id(),
                        round_id: round_id.clone(),
                        player_id: pid.clone(),
                    })
                    .execute(conn)?;
            }
        }
        Ok(())
    }).map_err(db_err)?;

    let game_state = broadcast(&state, &game_id).await?;
    Ok(Json(game_state))
}

// ── Votes ──

pub async fn record_votes(
    State(state): State<AppState>,
    Path((game_id, round_id)): Path<(String, String)>,
    Json(req): Json<RecordVotesRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    conn.transaction(|conn| {
        diesel::delete(
            schema::round_votes::table.filter(schema::round_votes::round_id.eq(&round_id)),
        )
        .execute(conn)?;

        for v in &req.votes {
            diesel::insert_into(schema::round_votes::table)
                .values(&NewRoundVote {
                    id: new_id(),
                    round_id: round_id.clone(),
                    player_id: v.player_id.clone(),
                    vote: v.vote,
                })
                .execute(conn)?;
        }

        Ok(())
    }).map_err(db_err)?;

    let game_state = broadcast(&state, &game_id).await?;
    Ok(Json(game_state))
}

// ── Lady of the Lake ──

pub async fn create_lady_investigation(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<CreateLadyInvestigationRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    conn.transaction(|conn| {
        diesel::insert_into(schema::lady_investigations::table)
            .values(&NewLadyInvestigation {
                id: new_id(),
                game_id: game_id.clone(),
                quest_id: req.quest_id,
                investigator_player_id: req.investigator_player_id,
                target_player_id: req.target_player_id.clone(),
                claimed_affiliation: req.claimed_affiliation,
            })
            .execute(conn)?;

        let next_order: i64 = schema::lady_holders::table
            .filter(schema::lady_holders::game_id.eq(&game_id))
            .count()
            .get_result(conn)?;

        diesel::insert_into(schema::lady_holders::table)
            .values(&NewLadyHolder {
                id: new_id(),
                game_id: game_id.clone(),
                player_id: req.target_player_id,
                holder_order: next_order as i32,
            })
            .execute(conn)?;

        Ok(())
    }).map_err(db_err)?;

    let game_state = broadcast(&state, &game_id).await?;
    Ok((StatusCode::CREATED, Json(game_state)))
}

// ── Lancelot Switches ──

pub async fn create_lancelot_switch(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<CreateLancelotSwitchRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    diesel::insert_into(schema::lancelot_switches::table)
        .values(&NewLancelotSwitch {
            id: new_id(),
            game_id: game_id.clone(),
            quest_number: req.quest_number,
            result: req.result,
        })
        .execute(&mut conn)
        .map_err(db_err)?;

    let game_state = broadcast(&state, &game_id).await?;
    Ok((StatusCode::CREATED, Json(game_state)))
}

// ── Plot Cards ──

pub async fn create_plot_card(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<CreatePlotCardRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    diesel::insert_into(schema::plot_cards::table)
        .values(&NewPlotCard {
            id: new_id(),
            game_id: game_id.clone(),
            quest_id: req.quest_id,
            player_id: req.player_id,
            card_name: req.card_name,
            status: PlotCardStatus::Dealt,
            used_on_player_id: None,
        })
        .execute(&mut conn)
        .map_err(db_err)?;

    let game_state = broadcast(&state, &game_id).await?;
    Ok((StatusCode::CREATED, Json(game_state)))
}

pub async fn update_plot_card(
    State(state): State<AppState>,
    Path((game_id, plot_card_id)): Path<(String, String)>,
    Json(req): Json<UpdatePlotCardRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    if let Some(status) = req.status {
        diesel::update(schema::plot_cards::table.find(&plot_card_id))
            .set(schema::plot_cards::status.eq(status))
            .execute(&mut conn)
            .map_err(db_err)?;
    }
    if let Some(ref used_on) = req.used_on_player_id {
        diesel::update(schema::plot_cards::table.find(&plot_card_id))
            .set(schema::plot_cards::used_on_player_id.eq(used_on))
            .execute(&mut conn)
            .map_err(db_err)?;
    }

    let game_state = broadcast(&state, &game_id).await?;
    Ok(Json(game_state))
}

// ── Assassination ──

pub async fn create_assassination_attempt(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<CreateAssassinationAttemptRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    // Overwrite any existing attempt for this (game, phase) so a mis-recorded
    // assassination can be corrected (the table has UNIQUE(game_id, phase)).
    conn.transaction(|conn| {
        diesel::delete(
            schema::assassination_attempts::table
                .filter(schema::assassination_attempts::game_id.eq(&game_id))
                .filter(schema::assassination_attempts::phase.eq(req.phase)),
        )
        .execute(conn)?;

        diesel::insert_into(schema::assassination_attempts::table)
            .values(&NewAssassinationAttempt {
                id: new_id(),
                game_id: game_id.clone(),
                phase: req.phase,
                sniper_player_id: req.sniper_player_id,
                snipe_type: req.snipe_type,
                target_player_ids: serde_json::to_string(&req.target_player_ids).unwrap(),
                correct: if req.correct { 1 } else { 0 },
            })
            .execute(conn)?;
        Ok(())
    })
    .map_err(db_err)?;

    let game_state = broadcast(&state, &game_id).await?;
    Ok((StatusCode::CREATED, Json(game_state)))
}

// ── Notes ──

pub async fn create_note(
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(req): Json<CreateNoteRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    diesel::insert_into(schema::notes::table)
        .values(&NewNote {
            id: new_id(),
            game_id: game_id.clone(),
            quest_id: req.quest_id,
            player_id: req.player_id,
            content: req.content,
        })
        .execute(&mut conn)
        .map_err(db_err)?;

    let game_state = broadcast(&state, &game_id).await?;
    Ok((StatusCode::CREATED, Json(game_state)))
}

pub async fn update_note(
    State(state): State<AppState>,
    Path((game_id, note_id)): Path<(String, String)>,
    Json(req): Json<UpdateNoteRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    if let Some(ref content) = req.content {
        diesel::update(schema::notes::table.find(&note_id))
            .set(schema::notes::content.eq(content))
            .execute(&mut conn)
            .map_err(db_err)?;
    }

    let game_state = broadcast(&state, &game_id).await?;
    Ok(Json(game_state))
}

pub async fn delete_note(
    State(state): State<AppState>,
    Path((game_id, note_id)): Path<(String, String)>,
) -> ApiResult<impl IntoResponse> {
    let mut conn = get_conn(&state.db)?;

    diesel::delete(schema::notes::table.find(&note_id))
        .execute(&mut conn)
        .map_err(db_err)?;

    let _ = broadcast(&state, &game_id).await;
    Ok(StatusCode::NO_CONTENT)
}
