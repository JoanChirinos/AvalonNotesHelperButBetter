// ── Enums (mirror backend types.rs unless noted) ──

// Frontend-only: derived from Role, not stored in backend
export type Team = 'good' | 'evil';

// Backend enums
export type QuestResult = 'success' | 'fail';
export type RoundStatus = 'proposed' | 'approved' | 'rejected';
export type Vote = 'approve' | 'reject';
export type CardType = 'success' | 'fail' | 'magic' | 'good_message' | 'evil_message';
export type LancelotSwitchResult = 'switch' | 'no_switch';
export type PlotCardStatus = 'dealt' | 'used';
export type SnipeType = 'merlin' | 'messengers' | 'untrustworthy_servant';
export type ClaimedAffiliation = 'good' | 'evil';

export type Role =
  | 'loyal_servant' | 'merlin' | 'percival' | 'cleric' | 'troublemaker'
  | 'untrustworthy_servant' | 'senior_messenger' | 'junior_messenger'
  | 'good_sorcerer' | 'good_lancelot'
  | 'minion_of_mordred' | 'assassin' | 'morgana' | 'mordred' | 'oberon'
  | 'trickster' | 'brute' | 'lunatic' | 'revealer'
  | 'evil_messenger' | 'evil_sorcerer' | 'evil_lancelot';

export type Module = 'lady_of_the_lake' | 'lancelot_switching' | 'plot_cards';

// ── DB row types (mirror backend models.rs) ──

export interface Game {
  id: string;
  created_at: string;
  finished_at: string | null;
  current_quest: number;
  namespace: string;
}

export interface KnownPlayer {
  id: string;
  name: string;
  namespace: string;
}

export interface Player {
  id: string;
  game_id: string;
  known_player_id: string;
  seat_order: number;
  role: Role | null;
}

export interface GameRole {
  id: string;
  game_id: string;
  role: Role;
}

export interface GameModule {
  id: string;
  game_id: string;
  module: Module;
}

export interface Quest {
  id: string;
  game_id: string;
  quest_number: number;
  result: QuestResult | null;
  success_count: number | null;
  fail_count: number | null;
  magic_count: number | null;
  good_message_count: number | null;
  evil_message_count: number | null;
}

export interface Round {
  id: string;
  quest_id: string;
  round_number: number;
  leader_player_id: string;
  status: RoundStatus;
}

export interface RoundTeam {
  id: string;
  round_id: string;
  player_id: string;
}

export interface RoundVote {
  id: string;
  round_id: string;
  player_id: string;
  vote: Vote;
}

export interface LadyInvestigation {
  id: string;
  game_id: string;
  quest_id: string;
  investigator_player_id: string;
  target_player_id: string;
  claimed_affiliation: ClaimedAffiliation;
}

export interface LadyHolder {
  id: string;
  game_id: string;
  player_id: string;
  holder_order: number;
}

export interface LancelotSwitch {
  id: string;
  game_id: string;
  quest_number: number;
  result: LancelotSwitchResult;
}

export interface PlotCard {
  id: string;
  game_id: string;
  quest_id: string;
  player_id: string;
  card_name: string;
  status: PlotCardStatus;
  used_on_player_id: string | null;
}

export interface AssassinationAttempt {
  id: string;
  game_id: string;
  phase: number;
  sniper_player_id: string;
  snipe_type: SnipeType;
  target_player_ids: string; // JSON array — use JSON.parse() to get string[]
  correct: number; // 0 or 1 (SQLite integer, not boolean)
}

export interface Note {
  id: string;
  game_id: string;
  quest_id: string | null;
  player_id: string | null;
  content: string;
  created_at: string;
}

// ── Composite types ──

export interface GameSummary {
  game: Game;
  player_names: string[];
  player_roles: (Role | null)[];
  has_started: boolean;
  result: 'good' | 'evil' | null;
}

export interface RoundState {
  round: Round;
  team: RoundTeam[];
  votes: RoundVote[];
}

export interface QuestState {
  quest: Quest;
  rounds: RoundState[];
}

export interface FullGameState {
  game: Game;
  players: Player[];
  known_players: KnownPlayer[];
  roles: GameRole[];
  modules: GameModule[];
  quests: QuestState[];
  lady_holders: LadyHolder[];
  lady_investigations: LadyInvestigation[];
  lancelot_switches: LancelotSwitch[];
  plot_cards: PlotCard[];
  assassination_attempts: AssassinationAttempt[];
  notes: Note[];
}

// ── API request types ──

export interface CreateGame {
  player_names?: string[];
  roles?: Role[];
  modules?: Module[];
  lady_holder_player_index?: number;
  namespace?: string;
}

export interface UpdateGame {
  finished_at?: string;
  current_quest?: number;
}

export interface UpdatePlayer {
  seat_order?: number;
  role?: Role;
  clear_role?: boolean;
}

export interface CreateRound {
  leader_player_id: string;
  team_player_ids: string[];
}

export interface UpdateRound {
  leader_player_id?: string;
  status?: RoundStatus;
  team_player_ids?: string[];
}

export interface PlayerVote {
  player_id: string;
  vote: Vote;
}

export interface RecordVotes {
  votes: PlayerVote[];
}

export interface UpdateQuest {
  result?: QuestResult;
  success_count?: number;
  fail_count?: number;
  magic_count?: number;
  good_message_count?: number;
  evil_message_count?: number;
}

export interface CreateLadyInvestigation {
  quest_id: string;
  investigator_player_id: string;
  target_player_id: string;
  claimed_affiliation: ClaimedAffiliation;
}

export interface CreateLancelotSwitch {
  quest_number: number;
  result: LancelotSwitchResult;
}

export interface CreatePlotCard {
  quest_id: string;
  player_id: string;
  card_name: string;
}

export interface UpdatePlotCard {
  status?: PlotCardStatus;
  used_on_player_id?: string;
}

export interface CreateAssassinationAttempt {
  phase: number;
  sniper_player_id: string;
  snipe_type: SnipeType;
  target_player_ids: string[];
  correct: boolean;
}

export interface CreateNote {
  quest_id?: string;
  player_id?: string;
  content: string;
}

export interface UpdateNote {
  content?: string;
}

// ── WebSocket message types ──

export type WsMessage =
  | { type: 'game_state'; data: FullGameState }
  | { type: 'error'; message: string };
