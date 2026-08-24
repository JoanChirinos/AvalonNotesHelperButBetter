import type { FullGameState, Game, KnownPlayer, GameSummary, CreateGame, UpdateGame, UpdatePlayer, CreateRound, UpdateRound, RecordVotes, UpdateQuest, CreateLadyInvestigation, CreateLancelotSwitch, CreatePlotCard, UpdatePlotCard, CreateAssassinationAttempt, CreateNote, UpdateNote } from './types';

export const BASE_PATH = import.meta.env.BASE_URL;
const BASE = `${BASE_PATH}api`;

async function request<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: body ? { 'Content-Type': 'application/json' } : {},
    body: body ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`${res.status}: ${text}`);
  }
  if (res.status === 204) return undefined as T;
  return res.json();
}

export const api = {
  // Namespaces
  listNamespaces: () => request<string[]>('GET', '/namespaces'),

  // Games
  listGames: (namespace: string) =>
    request<GameSummary[]>('GET', `/games?namespace=${encodeURIComponent(namespace)}`),
  getGame: (id: string) => request<FullGameState>('GET', `/games/${id}`),
  createGame: (data: CreateGame) => request<FullGameState>('POST', '/games', data),
  updateGame: (id: string, data: UpdateGame) => request<FullGameState>('PATCH', `/games/${id}`, data),
  deleteGame: (id: string) => request<void>('DELETE', `/games/${id}`),

  // Known players
  listKnownPlayers: (namespace: string) =>
    request<KnownPlayer[]>('GET', `/known-players?namespace=${encodeURIComponent(namespace)}`),

  // Players
  addPlayer: (gameId: string, data: { known_player_id?: string; name?: string }) =>
    request<FullGameState>('POST', `/games/${gameId}/players`, data),
  reorderPlayers: (gameId: string, playerIds: string[]) =>
    request<FullGameState>('POST', `/games/${gameId}/players/reorder`, { player_ids: playerIds }),
  updatePlayer: (gameId: string, playerId: string, data: UpdatePlayer) =>
    request<FullGameState>('PATCH', `/games/${gameId}/players/${playerId}`, data),
  deletePlayer: (gameId: string, playerId: string) =>
    request<void>('DELETE', `/games/${gameId}/players/${playerId}`),

  // Roles & Modules
  addRole: (gameId: string, role: string) =>
    request<FullGameState>('POST', `/games/${gameId}/roles`, { role }),
  deleteRole: (gameId: string, roleId: string) =>
    request<FullGameState>('DELETE', `/games/${gameId}/roles/${roleId}`),
  addModule: (gameId: string, module: string) =>
    request<FullGameState>('POST', `/games/${gameId}/modules`, { module }),
  deleteModule: (gameId: string, moduleId: string) =>
    request<FullGameState>('DELETE', `/games/${gameId}/modules/${moduleId}`),

  // Quests
  updateQuest: (gameId: string, questId: string, data: UpdateQuest) =>
    request<FullGameState>('PATCH', `/games/${gameId}/quests/${questId}`, data),

  // Rounds
  createRound: (gameId: string, questId: string, data: CreateRound) =>
    request<FullGameState>('POST', `/games/${gameId}/quests/${questId}/rounds`, data),
  updateRound: (gameId: string, roundId: string, data: UpdateRound) =>
    request<FullGameState>('PATCH', `/games/${gameId}/rounds/${roundId}`, data),

  // Votes
  recordVotes: (gameId: string, roundId: string, data: RecordVotes) =>
    request<FullGameState>('PUT', `/games/${gameId}/rounds/${roundId}/votes`, data),

  // Lady of the Lake
  createLadyInvestigation: (gameId: string, data: CreateLadyInvestigation) =>
    request<FullGameState>('POST', `/games/${gameId}/lady-investigations`, data),

  // Lancelot
  createLancelotSwitch: (gameId: string, data: CreateLancelotSwitch) =>
    request<FullGameState>('POST', `/games/${gameId}/lancelot-switches`, data),

  // Plot Cards
  createPlotCard: (gameId: string, data: CreatePlotCard) =>
    request<FullGameState>('POST', `/games/${gameId}/plot-cards`, data),
  updatePlotCard: (gameId: string, plotCardId: string, data: UpdatePlotCard) =>
    request<FullGameState>('PATCH', `/games/${gameId}/plot-cards/${plotCardId}`, data),

  // Assassination
  createAssassinationAttempt: (gameId: string, data: CreateAssassinationAttempt) =>
    request<FullGameState>('POST', `/games/${gameId}/assassination-attempts`, data),

  // Notes
  createNote: (gameId: string, data: CreateNote) =>
    request<FullGameState>('POST', `/games/${gameId}/notes`, data),
  updateNote: (gameId: string, noteId: string, data: UpdateNote) =>
    request<FullGameState>('PATCH', `/games/${gameId}/notes/${noteId}`, data),
  deleteNote: (gameId: string, noteId: string) =>
    request<void>('DELETE', `/games/${gameId}/notes/${noteId}`),
};
