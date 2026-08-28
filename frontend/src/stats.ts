import type { FullGameState, Role, Team } from './types';
import { deriveGameResult, deriveQuestResult, teamForRole, totalGoodMessages, totalEvilMessages } from './derived';
import { ROLE_DISPLAY_NAMES } from './constants';

// ── Facts layer ──────────────────────────────────────────────────────────────
// A normalized, reusable substrate computed once from finished game states. Stat
// blocks (the "questions") are pure functions over Facts. Add fields here as new
// kinds of questions need them (e.g. votes, leaders) — existing blocks are unaffected.

export interface Participation {
  knownPlayerId: string;
  name: string;
  role: Role | null;
  team: Team | null;
  won: boolean | null; // null when result or team is unknown
}

export interface AssassinationFact {
  sniperKnownId: string | null;
  sniperName: string;
  snipeType: string; // 'merlin' | 'messengers' | 'untrustworthy_servant'
  correct: boolean;
}

export interface GameFact {
  gameId: string;
  result: 'good' | 'evil' | null;
  createdAt: string;
  finishedAt: string | null;
  questsDecided: number;
  participations: Participation[];
  assassinations: AssassinationFact[];
}

export interface RosterEntry {
  knownPlayerId: string;
  name: string;
}

export interface Facts {
  roster: RosterEntry[];
  games: GameFact[];
}

export function buildFacts(games: FullGameState[]): Facts {
  const rosterMap = new Map<string, string>();

  const gameFacts: GameFact[] = games.map((g) => {
    const result = deriveGameResult(g);
    const nameByKnownId = new Map(g.known_players.map((kp) => [kp.id, kp.name]));
    const knownIdByPlayerId = new Map(g.players.map((p) => [p.id, p.known_player_id]));
    const msgTotals = { good: totalGoodMessages(g), evil: totalEvilMessages(g) };

    let questsDecided = 0;
    for (const q of g.quests) {
      const r = deriveQuestResult(q.quest, g.players.length, q.quest.quest_number === 5 ? msgTotals : undefined);
      if (r) questsDecided++;
    }

    const participations: Participation[] = g.players.map((p) => {
      const name = nameByKnownId.get(p.known_player_id) ?? '???';
      rosterMap.set(p.known_player_id, name);
      const role = (p.role ?? null) as Role | null;
      const team = role ? teamForRole(role) : null;
      const won = team && result ? team === result : null;
      return { knownPlayerId: p.known_player_id, name, role, team, won };
    });

    const assassinations: AssassinationFact[] = g.assassination_attempts.map((a) => {
      const sniperKnownId = knownIdByPlayerId.get(a.sniper_player_id) ?? null;
      return {
        sniperKnownId,
        sniperName: sniperKnownId ? (nameByKnownId.get(sniperKnownId) ?? '???') : '???',
        snipeType: a.snipe_type,
        correct: a.correct === 1,
      };
    });

    return {
      gameId: g.game.id,
      result,
      createdAt: g.game.created_at,
      finishedAt: g.game.finished_at,
      questsDecided,
      participations,
      assassinations,
    };
  });

  const roster = [...rosterMap.entries()]
    .map(([knownPlayerId, name]) => ({ knownPlayerId, name }))
    .sort((a, b) => a.name.localeCompare(b.name));

  return { roster, games: gameFacts };
}

// ── Widgets & blocks ─────────────────────────────────────────────────────────
// A stat block declares a section + a compute that returns a WidgetSpec (a tagged
// union of reusable display shapes). The renderer maps `kind` → component, so a new
// question just picks a widget and supplies data — display is automatic.

export type WidgetSpec =
  | { kind: 'kpis'; items: { label: string; value: string }[] }
  | { kind: 'bars'; segments: { label: string; value: number; tone: 'good' | 'evil' | 'neutral' }[] }
  | { kind: 'leaderboard'; rows: { label: string; value: number; display: string }[] }
  | { kind: 'table'; columns: { key: string; label: string; align?: 'left' | 'right' }[]; rows: Record<string, string | number>[] };

export interface StatResult {
  view: WidgetSpec;
  note?: string;
}

export interface StatBlock {
  id: string;
  title: string;
  section: 'group' | 'players';
  compute: (facts: Facts) => StatResult;
}

const pct = (n: number, d: number): string => (d ? `${Math.round((n / d) * 100)}%` : '—');

const MIN_GAMES = 3;

const overview: StatBlock = {
  id: 'overview',
  title: 'Overview',
  section: 'group',
  compute: (f) => {
    const total = f.games.length;
    const decided = f.games.filter((g) => g.result);
    const good = decided.filter((g) => g.result === 'good').length;
    const evil = decided.filter((g) => g.result === 'evil').length;
    const avgQuests = total ? f.games.reduce((s, g) => s + g.questsDecided, 0) / total : 0;
    return {
      view: {
        kind: 'kpis',
        items: [
          { label: 'Games', value: String(total) },
          { label: 'Good win rate', value: pct(good, decided.length) },
          { label: 'Evil win rate', value: pct(evil, decided.length) },
          { label: 'Avg quests / game', value: avgQuests ? avgQuests.toFixed(1) : '—' },
        ],
      },
      note: decided.length < total ? `${decided.length} of ${total} games have a decided result` : undefined,
    };
  },
};

const goodVsEvil: StatBlock = {
  id: 'good-vs-evil',
  title: 'Good vs Evil',
  section: 'group',
  compute: (f) => {
    const decided = f.games.filter((g) => g.result);
    const good = decided.filter((g) => g.result === 'good').length;
    const evil = decided.filter((g) => g.result === 'evil').length;
    return {
      view: {
        kind: 'bars',
        segments: [
          { label: 'Good', value: good, tone: 'good' },
          { label: 'Evil', value: evil, tone: 'evil' },
        ],
      },
    };
  },
};

const winRates: StatBlock = {
  id: 'win-rates',
  title: 'Win rate',
  section: 'players',
  compute: (f) => {
    const agg = new Map<string, { name: string; games: number; wins: number }>();
    for (const g of f.games) {
      for (const p of g.participations) {
        if (p.won === null) continue;
        const e = agg.get(p.knownPlayerId) ?? { name: p.name, games: 0, wins: 0 };
        e.games++;
        if (p.won) e.wins++;
        agg.set(p.knownPlayerId, e);
      }
    }
    const rows = [...agg.values()]
      .filter((e) => e.games >= MIN_GAMES)
      .map((e) => ({ label: e.name, value: e.wins / e.games, display: `${pct(e.wins, e.games)} (${e.wins}/${e.games})` }))
      .sort((a, b) => b.value - a.value);
    return { view: { kind: 'leaderboard', rows }, note: `players with at least ${MIN_GAMES} scored games` };
  },
};

const snipeAccuracy: StatBlock = {
  id: 'snipe-accuracy',
  title: 'Assassination accuracy',
  section: 'players',
  compute: (f) => {
    const agg = new Map<string, { name: string; snipes: number; correct: number }>();
    for (const g of f.games) {
      for (const a of g.assassinations) {
        // Phase-2 kills only (guessing Merlin / the Messengers).
        if (a.snipeType !== 'merlin' && a.snipeType !== 'messengers') continue;
        if (!a.sniperKnownId) continue;
        const e = agg.get(a.sniperKnownId) ?? { name: a.sniperName, snipes: 0, correct: 0 };
        e.snipes++;
        if (a.correct) e.correct++;
        agg.set(a.sniperKnownId, e);
      }
    }
    const rows = [...agg.values()]
      .map((e) => ({ label: e.name, value: e.snipes ? e.correct / e.snipes : 0, display: `${pct(e.correct, e.snipes)} (${e.correct}/${e.snipes})` }))
      .sort((a, b) => b.value - a.value);
    return { view: { kind: 'leaderboard', rows } };
  },
};

const rolePerformance: StatBlock = {
  id: 'role-performance',
  title: 'Role performance',
  section: 'players',
  compute: (f) => {
    const agg = new Map<string, { name: string; role: Role; times: number; wins: number }>();
    for (const g of f.games) {
      for (const p of g.participations) {
        if (!p.role) continue;
        const key = `${p.knownPlayerId}|${p.role}`;
        const e = agg.get(key) ?? { name: p.name, role: p.role, times: 0, wins: 0 };
        e.times++;
        if (p.won) e.wins++;
        agg.set(key, e);
      }
    }
    const rows = [...agg.values()]
      .sort((a, b) => a.name.localeCompare(b.name) || a.role.localeCompare(b.role))
      .map((e) => ({
        Player: e.name,
        Role: ROLE_DISPLAY_NAMES[e.role],
        Times: e.times,
        Wins: e.wins,
        'Win %': pct(e.wins, e.times),
      }));
    return {
      view: {
        kind: 'table',
        columns: [
          { key: 'Player', label: 'Player' },
          { key: 'Role', label: 'Role' },
          { key: 'Times', label: 'Times', align: 'right' },
          { key: 'Wins', label: 'Wins', align: 'right' },
          { key: 'Win %', label: 'Win %', align: 'right' },
        ],
        rows,
      },
    };
  },
};

/** The registry. Add a StatBlock here and it renders automatically. */
export const STAT_BLOCKS: StatBlock[] = [overview, goodVsEvil, winRates, snipeAccuracy, rolePerformance];
