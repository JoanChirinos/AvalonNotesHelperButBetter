import type { Quest, QuestState, FullGameState, Role, RoundState, Team } from './types';
import { questSize, failsRequired, GOOD_ROLES, teamForRole } from './constants';

export { teamForRole } from './constants';

export function deriveQuestResult(
  quest: Quest,
  playerCount: number,
  messageTotals?: { good: number; evil: number }
): 'success' | 'fail' | null {
  if (quest.success_count === null && quest.fail_count === null) return quest.result ?? null;

  const regularFails = quest.fail_count ?? 0;
  const magicCount = quest.magic_count ?? 0;

  let effectiveFails = regularFails + (quest.evil_message_count ?? 0);

  // Quest 5 message resolution
  if (quest.quest_number === 5 && messageTotals) {
    const goodApplies = messageTotals.good >= 3;
    const evilApplies = messageTotals.evil >= 2;
    if (goodApplies && !evilApplies) {
      effectiveFails = Math.max(0, regularFails - 1) + (quest.evil_message_count ?? 0);
    } else if (evilApplies && !goodApplies) {
      effectiveFails = regularFails + 1 + (quest.evil_message_count ?? 0);
    }
  }

  const threshold = failsRequired(playerCount, quest.quest_number);
  let result: 'success' | 'fail' = effectiveFails >= threshold ? 'fail' : 'success';

  if (magicCount % 2 === 1) result = result === 'success' ? 'fail' : 'success';

  return result;
}

export function currentQuestState(state: FullGameState): QuestState | null {
  return state.quests.find(q => q.quest.quest_number === state.game.current_quest) ?? null;
}

export function currentRoundState(state: FullGameState): RoundState | null {
  const quest = currentQuestState(state);
  if (!quest || quest.rounds.length === 0) return null;
  return quest.rounds[quest.rounds.length - 1];
}

export function nextLeaderIndex(state: FullGameState): number {
  // Find the last round's leader, advance by 1 in seat order
  const round = currentRoundState(state);
  if (!round) return 0;
  const leaderIdx = state.players.findIndex(p => p.id === round.round.leader_player_id);
  return (leaderIdx + 1) % state.players.length;
}

export function hasRole(state: FullGameState, role: Role): boolean {
  return state.roles.some(r => r.role === role);
}

export function hasSorcerers(state: FullGameState): boolean {
  return hasRole(state, 'good_sorcerer') || hasRole(state, 'evil_sorcerer');
}

export function hasMessengers(state: FullGameState): boolean {
  return hasRole(state, 'senior_messenger') || hasRole(state, 'evil_messenger');
}

export function playerName(state: FullGameState, knownPlayerId: string): string {
  return state.known_players.find(kp => kp.id === knownPlayerId)?.name ?? '???';
}

export function playerNameById(state: FullGameState, playerId: string): string {
  const player = state.players.find(p => p.id === playerId);
  if (!player) return '???';
  return playerName(state, player.known_player_id);
}

export function totalGoodMessages(state: FullGameState): number {
  return state.quests.reduce((sum, q) => sum + (q.quest.good_message_count ?? 0), 0);
}

export function totalEvilMessages(state: FullGameState): number {
  return state.quests.reduce((sum, q) => sum + (q.quest.evil_message_count ?? 0), 0);
}
