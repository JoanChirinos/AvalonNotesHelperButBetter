import type { FullGameState, Role, Team } from './types';
import { deriveGameResult, deriveQuestResult, teamForRole, totalGoodMessages, totalEvilMessages } from './derived';
import { ROLE_DISPLAY_NAMES } from './constants';

// ── Facts layer ──────────────────────────────────────────────────────────────
// Normalized substrate computed once from finished game states. Blocks (the
// "questions") are pure functions over Facts. Add fields here as new questions
// need them; existing blocks are unaffected.

export interface Participation {
  knownPlayerId: string;
  name: string;
  role: Role | null;
  team: Team | null;
  won: boolean | null;
}

export interface AssassinationFact {
  sniperKnownId: string | null;
  sniperName: string;
  targetKnownIds: string[];
  snipeType: string; // 'merlin' | 'messengers' | 'untrustworthy_servant'
  correct: boolean;
}

export interface LadyCheck {
  investigatorKnownId: string | null;
  investigatorName: string;
  investigatorTeam: Team | null;
  claimed: 'good' | 'evil';
  targetActualTeam: Team | null;
  truth: boolean | null; // claimed matches the target's real team; null if unknown
}

export interface GameFact {
  gameId: string;
  result: 'good' | 'evil' | null;
  createdAt: string;
  finishedAt: string | null;
  questsDecided: number;
  participations: Participation[];
  assassinations: AssassinationFact[];
  ladyChecks: LadyCheck[];
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
      let targetPlayerIds: string[] = [];
      try {
        targetPlayerIds = JSON.parse(a.target_player_ids) as string[];
      } catch {
        targetPlayerIds = [];
      }
      const targetKnownIds = targetPlayerIds
        .map((pid) => knownIdByPlayerId.get(pid))
        .filter((k): k is string => !!k);
      return {
        sniperKnownId,
        sniperName: sniperKnownId ? (nameByKnownId.get(sniperKnownId) ?? '???') : '???',
        targetKnownIds,
        snipeType: a.snipe_type,
        correct: a.correct === 1,
      };
    });

    const infoByPlayerId = new Map(
      g.players.map((p) => {
        const role = (p.role ?? null) as Role | null;
        return [p.id, { knownId: p.known_player_id, name: nameByKnownId.get(p.known_player_id) ?? '???', team: role ? teamForRole(role) : null }];
      })
    );
    const ladyChecks: LadyCheck[] = g.lady_investigations.map((inv) => {
      const investigator = infoByPlayerId.get(inv.investigator_player_id);
      const target = infoByPlayerId.get(inv.target_player_id);
      const claimed = inv.claimed_affiliation as 'good' | 'evil';
      const targetActualTeam = target?.team ?? null;
      return {
        investigatorKnownId: investigator?.knownId ?? null,
        investigatorName: investigator?.name ?? '???',
        investigatorTeam: investigator?.team ?? null,
        claimed,
        targetActualTeam,
        truth: targetActualTeam ? claimed === targetActualTeam : null,
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
      ladyChecks,
    };
  });

  const roster = [...rosterMap.entries()]
    .map(([knownPlayerId, name]) => ({ knownPlayerId, name }))
    .sort((a, b) => a.name.localeCompare(b.name));

  return { roster, games: gameFacts };
}

// ── Widgets ──────────────────────────────────────────────────────────────────
export type WidgetSpec =
  | { kind: 'kpis'; items: { label: string; value: string }[] }
  | { kind: 'bars'; segments: { label: string; value: number; tone: 'good' | 'evil' | 'neutral' }[] }
  | { kind: 'leaderboard'; rows: { label: string; value: number; display: string }[] }
  | { kind: 'table'; columns: { key: string; label: string; align?: 'left' | 'right' }[]; rows: Record<string, string | number>[] }
  | { kind: 'heatcells'; cells: { label: string; value: number }[] }
  | { kind: 'timeseries'; dates: string[] };

export interface StatResult {
  view: WidgetSpec;
  note?: string;
}

// Global blocks answer whole-namespace questions; player blocks answer questions
// about one player. Add a block to the matching registry and it renders itself.
export interface GlobalBlock {
  id: string;
  title: string;
  compute: (facts: Facts) => StatResult;
}
export interface PlayerBlock {
  id: string;
  title: string;
  compute: (facts: Facts, knownPlayerId: string) => StatResult;
}

const pct = (n: number, d: number): string => (d ? `${Math.round((n / d) * 100)}%` : '—');
const MIN_GAMES = 3;

// Helper: this player's participation in each game they were in.
function playerGames(facts: Facts, pid: string): { g: GameFact; p: Participation }[] {
  const out: { g: GameFact; p: Participation }[] = [];
  for (const g of facts.games) {
    const p = g.participations.find((x) => x.knownPlayerId === pid);
    if (p) out.push({ g, p });
  }
  return out;
}

// ── Global blocks ──────────────────────────────────────────────────────────
const overview: GlobalBlock = {
  id: 'overview',
  title: 'Overview',
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
          { label: 'Games played', value: String(total) },
          { label: 'Good win rate', value: pct(good, decided.length) },
          { label: 'Evil win rate', value: pct(evil, decided.length) },
          { label: 'Avg quests / game', value: avgQuests ? avgQuests.toFixed(1) : '—' },
        ],
      },
      note: decided.length < total ? `${decided.length} of ${total} games have a decided result` : undefined,
    };
  },
};

const winRateLeaderboard: GlobalBlock = {
  id: 'win-rates',
  title: 'Win rate',
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

const snipeAccuracyLeaderboard: GlobalBlock = {
  id: 'snipe-accuracy',
  title: 'Assassination accuracy',
  compute: (f) => {
    const agg = new Map<string, { name: string; snipes: number; correct: number }>();
    for (const g of f.games) {
      for (const a of g.assassinations) {
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

// A game's date for time analysis (prefer when it finished).
const gameDate = (g: GameFact): string => g.finishedAt ?? g.createdAt;

const WEEKDAYS = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

export type Bucket = 'day' | 'week' | 'month';

/** Group ISO dates into ordered {label, count} buckets. Exported for the chart + tests. */
export function bucketDates(dates: string[], bucket: Bucket): { key: string; label: string; count: number }[] {
  const counts = new Map<string, number>();
  for (const iso of dates) {
    const d = new Date(iso);
    if (isNaN(d.getTime())) continue;
    let key: string;
    if (bucket === 'day') {
      key = d.toISOString().slice(0, 10); // YYYY-MM-DD
    } else if (bucket === 'month') {
      key = d.toISOString().slice(0, 7); // YYYY-MM
    } else {
      // ISO week start (Monday)
      const t = new Date(Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), d.getUTCDate()));
      const day = (t.getUTCDay() + 6) % 7; // 0 = Monday
      t.setUTCDate(t.getUTCDate() - day);
      key = t.toISOString().slice(0, 10);
    }
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  return [...counts.entries()]
    .sort((a, b) => a[0].localeCompare(b[0]))
    .map(([key, count]) => ({ key, label: bucket === 'month' ? key : key, count }));
}

const dayOfWeek: GlobalBlock = {
  id: 'day-of-week',
  title: 'When we play',
  compute: (f) => {
    const counts = new Array(7).fill(0);
    for (const g of f.games) {
      const d = new Date(gameDate(g));
      if (isNaN(d.getTime())) continue;
      counts[(d.getUTCDay() + 6) % 7]++; // Monday-first
    }
    return { view: { kind: 'heatcells', cells: WEEKDAYS.map((label, i) => ({ label, value: counts[i] })) } };
  },
};

const gamesOverTime: GlobalBlock = {
  id: 'games-over-time',
  title: 'Games over time',
  compute: (f) => ({ view: { kind: 'timeseries', dates: f.games.map(gameDate) } }),
};

const ladyTruth: GlobalBlock = {
  id: 'lady-truth',
  title: 'Lady of the Lake',
  compute: (f) => {
    const checks = f.games.flatMap((g) => g.ladyChecks).filter((c) => c.truth !== null);
    const truths = checks.filter((c) => c.truth).length;
    const evil = checks.filter((c) => c.investigatorTeam === 'evil');
    const evilLies = evil.filter((c) => !c.truth).length;
    return {
      view: {
        kind: 'kpis',
        items: [
          { label: 'Checks', value: String(checks.length) },
          { label: 'Truth rate', value: pct(truths, checks.length) },
          { label: 'Lie rate', value: pct(checks.length - truths, checks.length) },
          { label: 'Evil lie rate', value: pct(evilLies, evil.length) },
        ],
      },
    };
  },
};

const biggestLiars: GlobalBlock = {
  id: 'biggest-liars',
  title: 'Biggest liars (as Lady)',
  compute: (f) => {
    const agg = new Map<string, { name: string; lies: number; checks: number }>();
    for (const g of f.games) {
      for (const c of g.ladyChecks) {
        if (c.truth === null || !c.investigatorKnownId) continue;
        const e = agg.get(c.investigatorKnownId) ?? { name: c.investigatorName, lies: 0, checks: 0 };
        e.checks++;
        if (!c.truth) e.lies++;
        agg.set(c.investigatorKnownId, e);
      }
    }
    const rows = [...agg.values()]
      .filter((e) => e.lies > 0)
      .map((e) => ({ label: e.name, value: e.lies, display: `${e.lies} lie${e.lies === 1 ? '' : 's'} of ${e.checks}` }))
      .sort((a, b) => b.value - a.value);
    return { view: { kind: 'leaderboard', rows } };
  },
};

export const GLOBAL_BLOCKS: GlobalBlock[] = [overview, dayOfWeek, gamesOverTime, winRateLeaderboard, snipeAccuracyLeaderboard, ladyTruth, biggestLiars];

// ── Player blocks ──────────────────────────────────────────────────────────
const playerSummary: PlayerBlock = {
  id: 'summary',
  title: 'Summary',
  compute: (f, pid) => {
    const gs = playerGames(f, pid);
    const scored = gs.filter(({ p }) => p.won !== null);
    const wins = scored.filter(({ p }) => p.won).length;
    const good = gs.filter(({ p }) => p.team === 'good').length;
    const evil = gs.filter(({ p }) => p.team === 'evil').length;
    return {
      view: {
        kind: 'kpis',
        items: [
          { label: 'Games', value: String(gs.length) },
          { label: 'Win rate', value: pct(wins, scored.length) },
          { label: 'Times Good', value: String(good) },
          { label: 'Times Evil', value: String(evil) },
        ],
      },
      note: scored.length < gs.length ? `${scored.length} of ${gs.length} scored` : undefined,
    };
  },
};

const playerByTeam: PlayerBlock = {
  id: 'by-team',
  title: 'By team',
  compute: (f, pid) => {
    const gs = playerGames(f, pid);
    const row = (team: Team, label: string) => {
      const inTeam = gs.filter(({ p }) => p.team === team && p.won !== null);
      const wins = inTeam.filter(({ p }) => p.won).length;
      return { Team: label, Games: inTeam.length, Wins: wins, 'Win %': pct(wins, inTeam.length) };
    };
    return {
      view: {
        kind: 'table',
        columns: [
          { key: 'Team', label: 'Team' },
          { key: 'Games', label: 'Games', align: 'right' },
          { key: 'Wins', label: 'Wins', align: 'right' },
          { key: 'Win %', label: 'Win %', align: 'right' },
        ],
        rows: [row('good', 'Good'), row('evil', 'Evil')],
      },
    };
  },
};

const playerByRole: PlayerBlock = {
  id: 'by-role',
  title: 'By role',
  compute: (f, pid) => {
    const agg = new Map<Role, { times: number; wins: number }>();
    for (const { p } of playerGames(f, pid)) {
      if (!p.role) continue;
      const e = agg.get(p.role) ?? { times: 0, wins: 0 };
      e.times++;
      if (p.won) e.wins++;
      agg.set(p.role, e);
    }
    const rows = [...agg.entries()]
      .sort((a, b) => b[1].times - a[1].times)
      .map(([role, e]) => ({ Role: ROLE_DISPLAY_NAMES[role], Times: e.times, Wins: e.wins, 'Win %': pct(e.wins, e.times) }));
    return {
      view: {
        kind: 'table',
        columns: [
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

const SNIPE_ROLES: { role: Role; label: string; type: string }[] = [
  { role: 'merlin', label: 'Merlin', type: 'merlin' },
  { role: 'senior_messenger', label: 'Senior Messenger', type: 'messengers' },
  { role: 'junior_messenger', label: 'Junior Messenger', type: 'messengers' },
  { role: 'untrustworthy_servant', label: 'Untrustworthy Servant', type: 'untrustworthy_servant' },
];

const playerSniped: PlayerBlock = {
  id: 'sniped',
  title: 'How often you get caught',
  compute: (f, pid) => {
    const rows = [];
    for (const sr of SNIPE_ROLES) {
      const games = f.games.filter((g) => g.participations.some((p) => p.knownPlayerId === pid && p.role === sr.role));
      if (games.length === 0) continue;
      const caught = games.filter((g) =>
        g.assassinations.some((a) => a.correct && a.snipeType === sr.type && a.targetKnownIds.includes(pid))
      ).length;
      rows.push({ Role: sr.label, Times: games.length, Caught: caught, Rate: pct(caught, games.length) });
    }
    return {
      view: {
        kind: 'table',
        columns: [
          { key: 'Role', label: 'Role' },
          { key: 'Times', label: 'Times', align: 'right' },
          { key: 'Caught', label: 'Caught', align: 'right' },
          { key: 'Rate', label: 'Rate', align: 'right' },
        ],
        rows,
      },
      note: rows.length ? undefined : 'Never held a role that can be sniped',
    };
  },
};

const playerLady: PlayerBlock = {
  id: 'lady',
  title: 'As Lady of the Lake',
  compute: (f, pid) => {
    const checks = f.games.flatMap((g) => g.ladyChecks).filter((c) => c.investigatorKnownId === pid && c.truth !== null);
    const lies = checks.filter((c) => !c.truth).length;
    return {
      view: {
        kind: 'kpis',
        items: [
          { label: 'Times as Lady', value: String(checks.length) },
          { label: 'Truths', value: String(checks.length - lies) },
          { label: 'Lies', value: String(lies) },
          { label: 'Lie rate', value: pct(lies, checks.length) },
        ],
      },
    };
  },
};

export const PLAYER_BLOCKS: PlayerBlock[] = [playerSummary, playerByTeam, playerByRole, playerSniped, playerLady];
