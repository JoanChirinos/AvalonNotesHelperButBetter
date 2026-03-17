import type { Role, Team } from './types';

export const GOOD_ROLES: Role[] = [
  'merlin', 'percival', 'untrustworthy_servant', 'senior_messenger',
  'junior_messenger', 'good_sorcerer', 'troublemaker', 'cleric',
  'good_lancelot', 'loyal_servant',
];

export const EVIL_ROLES: Role[] = [
  'mordred', 'morgana', 'evil_messenger', 'evil_sorcerer',
  'trickster', 'evil_lancelot', 'oberon', 'lunatic', 'brute',
  'revealer', 'assassin', 'minion_of_mordred',
];

export function teamForRole(role: Role): Team {
  return GOOD_ROLES.includes(role) ? 'good' : 'evil';
}

export const ROLE_DISPLAY_NAMES: Record<Role, string> = {
  loyal_servant: 'Loyal Servant',
  merlin: 'Merlin',
  percival: 'Percival',
  cleric: 'Cleric',
  troublemaker: 'Troublemaker',
  untrustworthy_servant: 'Untrustworthy Servant',
  senior_messenger: 'Senior Messenger',
  junior_messenger: 'Junior Messenger',
  good_sorcerer: 'Good Sorcerer',
  good_lancelot: 'Good Lancelot',
  minion_of_mordred: 'Minion of Mordred',
  assassin: 'Assassin',
  morgana: 'Morgana',
  mordred: 'Mordred',
  oberon: 'Oberon',
  trickster: 'Trickster',
  brute: 'Brute',
  lunatic: 'Lunatic',
  revealer: 'Revealer',
  evil_messenger: 'Evil Messenger',
  evil_sorcerer: 'Evil Sorcerer',
  evil_lancelot: 'Evil Lancelot',
};

// Roles that must be added/removed as a bundle
export const ROLE_BUNDLES: Record<string, Role[]> = {
  messengers: ['senior_messenger', 'junior_messenger', 'evil_messenger'],
  sorcerers: ['good_sorcerer', 'evil_sorcerer'],
  lancelots: ['good_lancelot', 'evil_lancelot'],
};

// Quest sizes by player count (index 0 = quest 1)
// For counts > 10, we don't have official sizes — return null
export function questSize(playerCount: number, questNumber: number): number | null {
  const table: Record<number, number[]> = {
    5:  [2, 3, 2, 3, 3],
    6:  [2, 3, 4, 3, 4],
    7:  [2, 3, 3, 4, 4],
    8:  [3, 4, 4, 5, 5],
    9:  [3, 4, 4, 5, 5],
    10: [3, 4, 4, 5, 5],
  };
  return table[playerCount]?.[questNumber - 1] ?? null;
}

// Quest 4 with 7+ players requires 2 fails
export function failsRequired(playerCount: number, questNumber: number): number {
  return (questNumber === 4 && playerCount >= 7) ? 2 : 1;
}
