use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::*;
use crate::types::*;

// ── Database row structs (Queryable) ──

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = games)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Game {
    pub id: String,
    pub created_at: String,
    pub finished_at: Option<String>,
    pub deleted_at: Option<String>,
    pub current_quest: i32,
    pub namespace: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = known_players)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct KnownPlayer {
    pub id: String,
    pub name: String,
    pub namespace: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = players)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Player {
    pub id: String,
    pub game_id: String,
    pub known_player_id: String,
    pub seat_order: i32,
    pub role: Option<Role>,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = game_roles)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct GameRole {
    pub id: String,
    pub game_id: String,
    pub role: Role,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = game_modules)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct GameModule {
    pub id: String,
    pub game_id: String,
    pub module: Module,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = quests)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Quest {
    pub id: String,
    pub game_id: String,
    pub quest_number: i32,
    pub result: Option<QuestResult>,
    pub success_count: Option<i32>,
    pub fail_count: Option<i32>,
    pub magic_count: Option<i32>,
    pub good_message_count: Option<i32>,
    pub evil_message_count: Option<i32>,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = rounds)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Round {
    pub id: String,
    pub quest_id: String,
    pub round_number: i32,
    pub leader_player_id: String,
    pub status: RoundStatus,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = round_teams)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RoundTeam {
    pub id: String,
    pub round_id: String,
    pub player_id: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = round_votes)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RoundVote {
    pub id: String,
    pub round_id: String,
    pub player_id: String,
    pub vote: Vote,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = lady_investigations)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct LadyInvestigation {
    pub id: String,
    pub game_id: String,
    pub quest_id: String,
    pub investigator_player_id: String,
    pub target_player_id: String,
    pub claimed_affiliation: ClaimedAffiliation,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = lady_holders)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct LadyHolder {
    pub id: String,
    pub game_id: String,
    pub player_id: String,
    pub holder_order: i32,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = lancelot_switches)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct LancelotSwitch {
    pub id: String,
    pub game_id: String,
    pub quest_number: i32,
    pub result: LancelotSwitchResult,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = plot_cards)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct PlotCard {
    pub id: String,
    pub game_id: String,
    pub quest_id: String,
    pub player_id: String,
    pub card_name: String,
    pub status: PlotCardStatus,
    pub used_on_player_id: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = assassination_attempts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AssassinationAttempt {
    pub id: String,
    pub game_id: String,
    pub phase: i32,
    pub sniper_player_id: String,
    pub snipe_type: SnipeType,
    pub target_player_ids: String, // JSON array of player IDs
    pub correct: i32,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = notes)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Note {
    pub id: String,
    pub game_id: String,
    pub quest_id: Option<String>,
    pub player_id: Option<String>,
    pub content: String,
    pub created_at: String,
}

// ── Insertable structs ──

#[derive(Debug, Insertable)]
#[diesel(table_name = games)]
pub struct NewGame {
    pub id: String,
    pub current_quest: i32,
    pub namespace: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = known_players)]
pub struct NewKnownPlayer {
    pub id: String,
    pub name: String,
    pub namespace: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = players)]
pub struct NewPlayer {
    pub id: String,
    pub game_id: String,
    pub known_player_id: String,
    pub seat_order: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = game_roles)]
pub struct NewGameRole {
    pub id: String,
    pub game_id: String,
    pub role: Role,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = game_modules)]
pub struct NewGameModule {
    pub id: String,
    pub game_id: String,
    pub module: Module,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = quests)]
pub struct NewQuest {
    pub id: String,
    pub game_id: String,
    pub quest_number: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = rounds)]
pub struct NewRound {
    pub id: String,
    pub quest_id: String,
    pub round_number: i32,
    pub leader_player_id: String,
    pub status: RoundStatus,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = round_teams)]
pub struct NewRoundTeam {
    pub id: String,
    pub round_id: String,
    pub player_id: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = round_votes)]
pub struct NewRoundVote {
    pub id: String,
    pub round_id: String,
    pub player_id: String,
    pub vote: Vote,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = lady_investigations)]
pub struct NewLadyInvestigation {
    pub id: String,
    pub game_id: String,
    pub quest_id: String,
    pub investigator_player_id: String,
    pub target_player_id: String,
    pub claimed_affiliation: ClaimedAffiliation,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = lady_holders)]
pub struct NewLadyHolder {
    pub id: String,
    pub game_id: String,
    pub player_id: String,
    pub holder_order: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = lancelot_switches)]
pub struct NewLancelotSwitch {
    pub id: String,
    pub game_id: String,
    pub quest_number: i32,
    pub result: LancelotSwitchResult,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = plot_cards)]
pub struct NewPlotCard {
    pub id: String,
    pub game_id: String,
    pub quest_id: String,
    pub player_id: String,
    pub card_name: String,
    pub status: PlotCardStatus,
    pub used_on_player_id: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = assassination_attempts)]
pub struct NewAssassinationAttempt {
    pub id: String,
    pub game_id: String,
    pub phase: i32,
    pub sniper_player_id: String,
    pub snipe_type: SnipeType,
    pub target_player_ids: String,
    pub correct: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = notes)]
pub struct NewNote {
    pub id: String,
    pub game_id: String,
    pub quest_id: Option<String>,
    pub player_id: Option<String>,
    pub content: String,
}

// ── API request structs ──

fn default_namespace() -> String {
    "SGW".to_string()
}

#[derive(Debug, Deserialize)]
pub struct NamespaceQuery {
    #[serde(default = "default_namespace")]
    pub namespace: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateGameRequest {
    #[serde(default)]
    pub player_names: Vec<String>,
    #[serde(default)]
    pub roles: Vec<Role>,
    #[serde(default)]
    pub modules: Vec<Module>,
    pub lady_holder_player_index: Option<usize>,
    #[serde(default = "default_namespace")]
    pub namespace: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGameRequest {
    pub finished_at: Option<String>,
    pub current_quest: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlayerRequest {
    pub seat_order: Option<i32>,
    pub role: Option<Role>,
    #[serde(default)]
    pub clear_role: bool,
}

#[derive(Debug, Deserialize)]
pub struct AddPlayerRequest {
    // Either provide known_player_id OR name (creates new known player)
    pub known_player_id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ReorderPlayersRequest {
    pub player_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddRoleRequest {
    pub role: Role,
}

#[derive(Debug, Deserialize)]
pub struct AddModuleRequest {
    pub module: Module,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoundRequest {
    pub leader_player_id: String,
    pub team_player_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoundRequest {
    pub leader_player_id: Option<String>,
    pub status: Option<RoundStatus>,
    pub team_player_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct PlayerVoteInput {
    pub player_id: String,
    pub vote: Vote,
}

#[derive(Debug, Deserialize)]
pub struct RecordVotesRequest {
    pub votes: Vec<PlayerVoteInput>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateQuestRequest {
    pub result: Option<QuestResult>,
    pub success_count: Option<i32>,
    pub fail_count: Option<i32>,
    pub magic_count: Option<i32>,
    pub good_message_count: Option<i32>,
    pub evil_message_count: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLadyInvestigationRequest {
    pub quest_id: String,
    pub investigator_player_id: String,
    pub target_player_id: String,
    pub claimed_affiliation: ClaimedAffiliation,
}

#[derive(Debug, Deserialize)]
pub struct CreateLancelotSwitchRequest {
    pub quest_number: i32,
    pub result: LancelotSwitchResult,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlotCardRequest {
    pub quest_id: String,
    pub player_id: String,
    pub card_name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePlotCardRequest {
    pub status: Option<PlotCardStatus>,
    pub used_on_player_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssassinationAttemptRequest {
    pub phase: i32,
    pub sniper_player_id: String,
    pub snipe_type: SnipeType,
    pub target_player_ids: Vec<String>,
    pub correct: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub quest_id: Option<String>,
    pub player_id: Option<String>,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateNoteRequest {
    pub content: Option<String>,
}

// ── Response for game list (lightweight summary) ──

#[derive(Debug, Clone, Serialize)]
pub struct GameSummary {
    pub game: Game,
    pub player_names: Vec<String>,
    pub player_roles: Vec<Option<String>>,
    pub has_started: bool,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FullGameState {
    pub game: Game,
    pub players: Vec<Player>,
    pub known_players: Vec<KnownPlayer>,
    pub roles: Vec<GameRole>,
    pub modules: Vec<GameModule>,
    pub quests: Vec<QuestState>,
    pub lady_holders: Vec<LadyHolder>,
    pub lady_investigations: Vec<LadyInvestigation>,
    pub lancelot_switches: Vec<LancelotSwitch>,
    pub plot_cards: Vec<PlotCard>,
    pub assassination_attempts: Vec<AssassinationAttempt>,
    pub notes: Vec<Note>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuestState {
    pub quest: Quest,
    pub rounds: Vec<RoundState>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoundState {
    pub round: Round,
    pub team: Vec<RoundTeam>,
    pub votes: Vec<RoundVote>,
}
