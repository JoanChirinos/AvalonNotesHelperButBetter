import type { Role, Module } from './types';
import { teamForRole } from './constants';

// ── Pause tiers ────────────────────────────────────────────────────────────
// One base value; medium/long are multiples so tuning `short` scales everything.
// Used AFTER a line is spoken, to give players time to act. (Later: expose in UI.)
export const PAUSE_SHORT_MS = 2000;
export const PAUSE_MULTIPLIER = { short: 1, medium: 1.5, long: 2 } as const;
export type PauseTier = keyof typeof PAUSE_MULTIPLIER;

export function pauseMs(tier: PauseTier): number {
  return PAUSE_SHORT_MS * PAUSE_MULTIPLIER[tier];
}

export interface RevealLine {
  text: string;
  /** Pause AFTER speaking. `long` after "open your eyes and look" (players need time);
   *  `short` after passive lines ("close your eyes", "extend your thumb"). */
  pause: PauseTier;
}

// ── Context handed to each part ──────────────────────────────────────────────
interface RevealContext {
  has: (role: Role) => boolean;
  hasModule: (module: Module) => boolean;
  hasAnyEvil: boolean;
}

interface RevealPart {
  id: string;
  applies: (ctx: RevealContext) => boolean;
  lines: (ctx: RevealContext) => RevealLine[];
}

// Join a base subject with optional inclusion/exclusion clauses, e.g.
//   "Minions of Mordred" + [", except Mordred", ", and the Untrustworthy Servant"]
const subject = (base: string, ...clauses: (string | false)[]): string =>
  base + clauses.filter(Boolean).join('');

// Blanket reset between steps. Only the ACTION lines name specific roles (who
// extends a thumb / opens their eyes); the reset is generic so we never tell a
// role to undo something it didn't do (e.g. Mordred/Oberon who never opened).
const RESET: RevealLine = {
  text: 'Everyone, close your eyes and re-form your hand into a fist.',
  pause: 'short',
};

// ── The script, as an ordered list of parts ──────────────────────────────────
// To add a new role's reveal step in the future: add a RevealPart here. Each part
// is self-contained (its own `applies` predicate and `lines`); order in this array
// is the order narrated. Convention: name roles only on action lines, end with RESET.
const PARTS: RevealPart[] = [
  {
    id: 'open',
    applies: () => true,
    lines: () => [
      { text: 'Everyone, close your eyes and extend your hand into a fist in front of you.', pause: 'long' },
    ],
  },
  {
    id: 'cleric',
    applies: (c) => c.has('cleric'),
    lines: () => [
      { text: 'Leader, extend your thumb if you are Evil.', pause: 'short' },
      { text: 'Cleric, open your eyes and see whether your Leader is Good or Evil.', pause: 'long' },
      RESET,
    ],
  },
  {
    id: 'evil-sees-evil',
    applies: (c) => c.hasAnyEvil,
    lines: (c) => [
      {
        text: `${subject('Minions of Mordred', c.has('oberon') && ', except Oberon')}, open your eyes and look around so you know all agents of Evil.`,
        pause: 'long',
      },
      RESET,
    ],
  },
  {
    id: 'merlin',
    applies: (c) => c.has('merlin'),
    lines: (c) => {
      const extendSubject = subject(
        'Minions of Mordred',
        c.has('mordred') && ', except Mordred',
        c.has('untrustworthy_servant') && ', and the Untrustworthy Servant',
      );
      return [
        { text: `${extendSubject}, extend your thumb so Merlin will know of you.`, pause: 'short' },
        { text: 'Merlin, open your eyes and see the agents of Evil.', pause: 'long' },
        RESET,
      ];
    },
  },
  {
    id: 'percival',
    applies: (c) => c.has('percival'),
    lines: (c) => {
      const subj = subject('Merlin', c.has('morgana') && ' and Morgana');
      return [
        { text: `${subj}, extend your thumb so Percival may know of you.`, pause: 'short' },
        { text: `Percival, open your eyes and see ${subj}.`, pause: 'long' },
        RESET,
      ];
    },
  },
  {
    id: 'messengers',
    // Senior always knows Junior (optional rule always on for us).
    applies: (c) => c.has('senior_messenger') && c.has('junior_messenger'),
    lines: () => [
      { text: 'Junior Messenger, extend your thumb so the Senior Messenger may know you.', pause: 'short' },
      { text: 'Senior Messenger, open your eyes and see the Junior Messenger.', pause: 'long' },
      RESET,
    ],
  },
  {
    id: 'close',
    applies: () => true,
    lines: () => [{ text: 'Everyone, open your eyes.', pause: 'short' }],
  },
];

/** Build the ordered reveal narration for the roles (and modules) present. */
export function buildRevealScript(roles: Role[], modules: Module[] = []): RevealLine[] {
  const roleSet = new Set(roles);
  const moduleSet = new Set(modules);
  const ctx: RevealContext = {
    has: (r) => roleSet.has(r),
    hasModule: (m) => moduleSet.has(m),
    hasAnyEvil: roles.some((r) => teamForRole(r) === 'evil'),
  };
  return PARTS.filter((p) => p.applies(ctx)).flatMap((p) => p.lines(ctx));
}
