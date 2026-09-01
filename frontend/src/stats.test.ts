import { describe, it, expect } from 'vitest';
import { buildFacts, GLOBAL_BLOCKS, PLAYER_BLOCKS, type Facts, type GameFact, type Participation } from './stats';
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

function game(
  result: 'good' | 'evil' | null,
  participations: Participation[],
  assassinations: GameFact['assassinations'] = []
): GameFact {
  return { gameId: 'g', result, createdAt: '', finishedAt: '', questsDecided: 3, participations, assassinations };
}

const gblock = (id: string) => GLOBAL_BLOCKS.find((b) => b.id === id)!;
const pblock = (id: string) => PLAYER_BLOCKS.find((b) => b.id === id)!;

describe('global: win-rate leaderboard', () => {
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
    const rows = (gblock('win-rates').compute(facts).view as any).rows;
    expect(rows).toHaveLength(1);
    expect(rows[0].label).toBe('Ann');
    expect(rows[0].display).toBe('67% (2/3)');
  });
});

describe('global: snipe accuracy', () => {
  it('aggregates phase-2 snipes per sniper', () => {
    const facts: Facts = {
      roster: [],
      games: [
        game('good', [], [{ sniperKnownId: 'k9', sniperName: 'Zed', targetKnownIds: [], snipeType: 'merlin', correct: false }]),
        game('evil', [], [{ sniperKnownId: 'k9', sniperName: 'Zed', targetKnownIds: ['k1'], snipeType: 'merlin', correct: true }]),
      ],
    };
    const rows = (gblock('snipe-accuracy').compute(facts).view as any).rows;
    expect(rows[0].label).toBe('Zed');
    expect(rows[0].display).toBe('50% (1/2)');
  });
});

describe('global: overview', () => {
  it('reports totals and win rates over decided games', () => {
    const facts: Facts = { roster: [], games: [game('good', []), game('evil', []), game(null, [])] };
    const res = gblock('overview').compute(facts);
    const items = (res.view as any).items as { label: string; value: string }[];
    expect(items.find((i) => i.label === 'Games played')!.value).toBe('3');
    expect(items.find((i) => i.label === 'Good win rate')!.value).toBe('50%');
    expect(res.note).toMatch(/2 of 3/);
  });
});

describe('player: by-role', () => {
  it('counts appearances and wins per role for the selected player', () => {
    const facts: Facts = {
      roster: [],
      games: [
        game('good', [P('k1', 'Ann', 'merlin', 'good', true)]),
        game('evil', [P('k1', 'Ann', 'merlin', 'good', false)]),
        game('evil', [P('k1', 'Ann', 'morgana', 'evil', true)]),
        game('good', [P('k2', 'Bo', 'merlin', 'good', true)]), // other player, ignored
      ],
    };
    const rows = (pblock('by-role').compute(facts, 'k1').view as any).rows as Record<string, string | number>[];
    const merlin = rows.find((r) => r.Role === 'Merlin')!;
    expect(merlin.Times).toBe(2);
    expect(merlin.Wins).toBe(1);
    expect(merlin['Win %']).toBe('50%');
    expect(rows.find((r) => r.Role === 'Morgana')!.Times).toBe(1);
  });
});

describe('player: sniped ("how often you get caught")', () => {
  it('counts games as Merlin where the correct snipe targeted the player', () => {
    const facts: Facts = {
      roster: [],
      games: [
        // Ann is Merlin and is correctly sniped
        game('evil', [P('k1', 'Ann', 'merlin', 'good', false)], [{ sniperKnownId: 'k9', sniperName: 'Z', targetKnownIds: ['k1'], snipeType: 'merlin', correct: true }]),
        // Ann is Merlin and survives (wrong snipe)
        game('good', [P('k1', 'Ann', 'merlin', 'good', true)], [{ sniperKnownId: 'k9', sniperName: 'Z', targetKnownIds: ['k2'], snipeType: 'merlin', correct: false }]),
      ],
    };
    const rows = (pblock('sniped').compute(facts, 'k1').view as any).rows as Record<string, string | number>[];
    const merlin = rows.find((r) => r.Role === 'Merlin')!;
    expect(merlin.Times).toBe(2);
    expect(merlin.Caught).toBe(1);
    expect(merlin.Rate).toBe('50%');
  });
});

import { bucketDates } from './stats';

describe('bucketDates', () => {
  const dates = ['2026-06-01T10:00:00Z', '2026-06-01T20:00:00Z', '2026-06-08T10:00:00Z', '2026-07-02T10:00:00Z'];
  it('buckets by day', () => {
    const b = bucketDates(dates, 'day');
    expect(b.find((x) => x.key === '2026-06-01')!.count).toBe(2);
    expect(b).toHaveLength(3);
  });
  it('buckets by week (Monday start) and month', () => {
    expect(bucketDates(dates, 'week').find((x) => x.key === '2026-06-01')!.count).toBe(2); // Jun 1 2026 is a Monday
    const m = bucketDates(dates, 'month');
    expect(m.find((x) => x.key === '2026-06')!.count).toBe(3);
    expect(m.find((x) => x.key === '2026-07')!.count).toBe(1);
  });
});
