import { describe, it, expect } from 'vitest';
import { questSize, failsRequired, GOOD_ROLES, EVIL_ROLES, ROLE_BUNDLES } from './constants';

describe('questSize', () => {
  it('returns correct sizes for 5 players', () => {
    expect(questSize(5, 1)).toBe(2);
    expect(questSize(5, 2)).toBe(3);
    expect(questSize(5, 3)).toBe(2);
    expect(questSize(5, 4)).toBe(3);
    expect(questSize(5, 5)).toBe(3);
  });

  it('returns correct sizes for 7 players', () => {
    expect(questSize(7, 1)).toBe(2);
    expect(questSize(7, 2)).toBe(3);
    expect(questSize(7, 3)).toBe(3);
    expect(questSize(7, 4)).toBe(4);
    expect(questSize(7, 5)).toBe(4);
  });

  it('returns correct sizes for 10 players', () => {
    expect(questSize(10, 1)).toBe(3);
    expect(questSize(10, 2)).toBe(4);
    expect(questSize(10, 3)).toBe(4);
    expect(questSize(10, 4)).toBe(5);
    expect(questSize(10, 5)).toBe(5);
  });

  it('returns null for out-of-range player counts', () => {
    expect(questSize(4, 1)).toBe(null);
    expect(questSize(11, 1)).toBe(null);
    expect(questSize(0, 1)).toBe(null);
  });

  it('returns null for out-of-range quest numbers', () => {
    expect(questSize(5, 0)).toBe(null);
    expect(questSize(5, 6)).toBe(null);
  });
});

describe('failsRequired', () => {
  it('returns 2 for quest 4 with 7+ players', () => {
    expect(failsRequired(7, 4)).toBe(2);
    expect(failsRequired(8, 4)).toBe(2);
    expect(failsRequired(9, 4)).toBe(2);
    expect(failsRequired(10, 4)).toBe(2);
  });

  it('returns 1 for quest 4 with fewer than 7 players', () => {
    expect(failsRequired(5, 4)).toBe(1);
    expect(failsRequired(6, 4)).toBe(1);
  });

  it('returns 1 for all other quests regardless of player count', () => {
    for (let q = 1; q <= 5; q++) {
      if (q === 4) continue;
      expect(failsRequired(10, q)).toBe(1);
      expect(failsRequired(5, q)).toBe(1);
    }
  });
});

describe('role lists', () => {
  it('good and evil roles have no overlap', () => {
    const overlap = GOOD_ROLES.filter(r => EVIL_ROLES.includes(r as any));
    expect(overlap).toEqual([]);
  });

  it('all bundle roles exist in role lists', () => {
    const allRoles = [...GOOD_ROLES, ...EVIL_ROLES];
    for (const bundle of Object.values(ROLE_BUNDLES)) {
      for (const role of bundle) {
        expect(allRoles).toContain(role);
      }
    }
  });

  it('messenger bundle has 3 roles', () => {
    expect(ROLE_BUNDLES.messengers).toHaveLength(3);
  });

  it('sorcerer bundle has 2 roles', () => {
    expect(ROLE_BUNDLES.sorcerers).toHaveLength(2);
  });

  it('lancelot bundle has 2 roles', () => {
    expect(ROLE_BUNDLES.lancelots).toHaveLength(2);
  });
});
