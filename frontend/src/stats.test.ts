import { describe, it, expect } from 'vitest';
import { buildFacts, STAT_BLOCKS, type Facts, type GameFact, type Participation } from './stats';
import { deriveGameResult } from './derived';
import type { FullGameState, Role } from './types';

// ── Minimal builders ─────────────────────────────────────────────────────────
function quest(n: number, result: 'success' | 'fail' | null) {
  return {
    quest: {
      id: `q${n}`, game_id: 'g', quest_number: n, result,
      success_count: null, fail_count: null, magic_count: null,
      good_message_count: null, evil_message_count: null,
    },
    rounds: [],
  };
}

interface MkOpts {
  id?: string;
  finished?: boolean;
  questResults?: ('success' | 'fail' | null)[];
  players?: { kid: string; name: string; role?: Role | null }[];
  phase2?: { sniperId: string; correct: boolean };
}

function mkGame(o: MkOpts): FullGameState {
  const players = (o.players ?? []).map((p, i) => ({
    id: `p${i}`, game_id: o.id ?? 'g', known_player_id: p.kid, seat_order: i + 1, role: p.role ?? null,
  }));
  const known_players = (o.players ?? []).map((p) => ({ id: p.kid, name: p.name, namespace: 'SGW' }));
  const quests = (o.questResults ?? []).map((r, i) => quest(i + 1, r));
  const assassination_attempts = o.phase2
    ? [{ id: 'a', game_id: o.id ?? 'g', phase: 2, sniper_player_id: `p${(o.players ?? []).findIndex((x) => x.kid === o.phase2!.sniperId)}`, snipe_type: 'merlin', target_player_ids: '[]', correct: o.phase2.correct ? 1 : 0 }]
    : [];
  return {
    game: { id: o.id ?? 'g', created_at: '2026-01-01T00:00:00Z', finished_at: o.finished ? '2026-01-02T00:00:00Z' : null, deleted_at: null, current_quest: 5, namespace: 'SGW' },
    players, known_players, roles: [], modules: [], quests,
    lady_holders: [], lady_investigations: [], lancelot_switches: [], plot_cards: [], assassination_attempts, notes: [],
  } as unknown as FullGameState;
}

describe('deriveGameResult', () => {
  it('evil when 3 quests fail', () => {
    expect(deriveGameResult(mkGame({ questResults: ['fail', 'fail', 'fail'] }))).toBe('evil');
  });
  it('null when 3 succeed but no phase-2 snipe recorded', () => {
    expect(deriveGameResult(mkGame({ questResults: ['success', 'success', 'success'] }))).toBeNull();
  });
  it('good when 3 succeed and the snipe is wrong', () => {
    const g = mkGame({ questResults: ['success', 'success', 'success'], players: [{ kid: 'k1', name: 'A' }], phase2: { sniperId: 'k1', correct: false } });
    expect(deriveGameResult(g)).toBe('good');
  });
  it('evil when 3 succeed and the snipe is correct', () => {
    const g = mkGame({ questResults: ['success', 'success', 'success'], players: [{ kid: 'k1', name: 'A' }], phase2: { sniperId: 'k1', correct: true } });
    expect(deriveGameResult(g)).toBe('evil');
  });
  it('null while undecided', () => {
    expect(deriveGameResult(mkGame({ questResults: ['success', 'fail'] }))).toBeNull();
  });
});

describe('buildFacts', () => {
  it('resolves per-player team and won from role + result', () => {
    const g = mkGame({
      finished: true,
      questResults: ['fail', 'fail', 'fail'], // evil wins
      players: [
        { kid: 'k1', name: 'Merl', role: 'merlin' },        // good -> lost
        { kid: 'k2', name: 'Mord', role: 'minion_of_mordred' }, // evil -> won
      ],
    });
    const facts = buildFacts([g]);
    expect(facts.roster.map((r) => r.name)).toEqual(['Merl', 'Mord']);
    const gf = facts.games[0];
    expect(gf.result).toBe('evil');
    const merl = gf.participations.find((p) => p.knownPlayerId === 'k1')!;
    const mord = gf.participations.find((p) => p.knownPlayerId === 'k2')!;
    expect(merl.team).toBe('good');
    expect(merl.won).toBe(false);
    expect(mord.team).toBe('evil');
    expect(mord.won).toBe(true);
  });
});

// ── Block compute tests over hand-built Facts ────────────────────────────────
const P = (kid: string, name: string, role: Role | null, team: 'good' | 'evil' | null, won: boolean | null): Participation =>
  ({ knownPlayerId: kid, name, role, team, won });

function game(result: 'good' | 'evil' | null, participations: Participation[], assassinations: GameFact['assassinations'] = []): GameFact {
  return { gameId: 'g', result, createdAt: '', finishedAt: '', questsDecided: 3, participations, assassinations };
}

const block = (id: string) => STAT_BLOCKS.find((b) => b.id === id)!;

describe('win-rates block', () => {
  it('computes win % and applies the min-games filter', () => {
    const facts: Facts = {
      roster: [],
      games: [
        game('good', [P('k1', 'Ann', 'merlin', 'good', true)]),
        game('evil', [P('k1', 'Ann', 'merlin', 'good', false)]),
        game('good', [P('k1', 'Ann', 'merlin', 'good', true)]),
        game('good', [P('k2', 'Bo', 'assassin', 'evil', false)]), // only 1 game -> filtered out
      ],
    };
    const rows = (block('win-rates').compute(facts).view as any).rows;
    expect(rows).toHaveLength(1);
    expect(rows[0].label).toBe('Ann');
    expect(rows[0].display).toBe('67% (2/3)');
  });
});

describe('role-performance block', () => {
  it('counts appearances and wins per (player, role)', () => {
    const facts: Facts = {
      roster: [],
      games: [
        game('good', [P('k1', 'Ann', 'merlin', 'good', true)]),
        game('evil', [P('k1', 'Ann', 'merlin', 'good', false)]),
        game('evil', [P('k1', 'Ann', 'morgana', 'evil', true)]),
      ],
    };
    const rows = (block('role-performance').compute(facts).view as any).rows as Record<string, string | number>[];
    const merlinRow = rows.find((r) => r.Role === 'Merlin')!;
    expect(merlinRow.Times).toBe(2);
    expect(merlinRow.Wins).toBe(1);
    expect(merlinRow['Win %']).toBe('50%');
  });
});

describe('snipe-accuracy block', () => {
  it('aggregates phase-2 snipes per sniper', () => {
    const facts: Facts = {
      roster: [],
      games: [
        game('good', [], [{ sniperKnownId: 'k9', sniperName: 'Zed', snipeType: 'merlin', correct: false }]),
        game('evil', [], [{ sniperKnownId: 'k9', sniperName: 'Zed', snipeType: 'merlin', correct: true }]),
      ],
    };
    const rows = (block('snipe-accuracy').compute(facts).view as any).rows;
    expect(rows[0].label).toBe('Zed');
    expect(rows[0].display).toBe('50% (1/2)');
  });
});

describe('overview block', () => {
  it('reports totals and win rates over decided games', () => {
    const facts: Facts = {
      roster: [],
      games: [game('good', []), game('evil', []), game(null, [])],
    };
    const res = block('overview').compute(facts);
    const items = (res.view as any).items as { label: string; value: string }[];
    expect(items.find((i) => i.label === 'Games')!.value).toBe('3');
    expect(items.find((i) => i.label === 'Good win rate')!.value).toBe('50%');
    expect(res.note).toMatch(/2 of 3/);
  });
});
