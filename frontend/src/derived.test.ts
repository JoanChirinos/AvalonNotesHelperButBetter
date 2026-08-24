import { describe, it, expect } from 'vitest';
import { deriveQuestResult, teamForRole, totalGoodMessages, totalEvilMessages, hasSorcerers, hasMessengers } from './derived';
import type { Quest, FullGameState, Role } from './types';

function makeQuest(overrides: Partial<Quest> = {}): Quest {
  return {
    id: 'q1',
    game_id: 'g1',
    quest_number: 1,
    result: null,
    success_count: null,
    fail_count: null,
    magic_count: null,
    good_message_count: null,
    evil_message_count: null,
    ...overrides,
  };
}

describe('deriveQuestResult', () => {
  it('returns null when no cards played and no stored result', () => {
    expect(deriveQuestResult(makeQuest(), 5)).toBe(null);
  });

  it('returns stored result when no cards played (5-rejection case)', () => {
    expect(deriveQuestResult(makeQuest({ result: 'fail' }), 5)).toBe('fail');
  });

  it('returns success when no fails', () => {
    expect(deriveQuestResult(makeQuest({ success_count: 3, fail_count: 0 }), 5)).toBe('success');
  });

  it('returns fail when fails meet threshold', () => {
    expect(deriveQuestResult(makeQuest({ success_count: 2, fail_count: 1 }), 5)).toBe('fail');
  });

  it('counts evil messages as fails', () => {
    expect(deriveQuestResult(makeQuest({ success_count: 2, fail_count: 0, evil_message_count: 1 }), 5)).toBe('fail');
  });

  it('magic card flips result (odd count)', () => {
    const quest = makeQuest({ success_count: 2, fail_count: 1, magic_count: 1 });
    expect(deriveQuestResult(quest, 5)).toBe('success');
  });

  it('even magic count does not flip', () => {
    const quest = makeQuest({ success_count: 2, fail_count: 1, magic_count: 2 });
    expect(deriveQuestResult(quest, 5)).toBe('fail');
  });

  it('quest 4 with 7+ players requires 2 fails', () => {
    const quest = makeQuest({ quest_number: 4, success_count: 3, fail_count: 1 });
    expect(deriveQuestResult(quest, 7)).toBe('success');
    expect(deriveQuestResult(makeQuest({ quest_number: 4, success_count: 2, fail_count: 2 }), 7)).toBe('fail');
  });

  it('quest 4 with 6 players requires only 1 fail', () => {
    const quest = makeQuest({ quest_number: 4, success_count: 2, fail_count: 1 });
    expect(deriveQuestResult(quest, 6)).toBe('fail');
  });

  it('zero fails with magic flip produces fail', () => {
    const quest = makeQuest({ success_count: 3, fail_count: 0, magic_count: 1 });
    expect(deriveQuestResult(quest, 5)).toBe('fail');
  });
});

describe('teamForRole', () => {
  const goodRoles: Role[] = [
    'loyal_servant', 'merlin', 'percival', 'cleric', 'troublemaker',
    'untrustworthy_servant', 'senior_messenger', 'junior_messenger',
    'good_sorcerer', 'good_lancelot',
  ];

  const evilRoles: Role[] = [
    'minion_of_mordred', 'assassin', 'morgana', 'mordred', 'oberon',
    'trickster', 'brute', 'lunatic', 'revealer',
    'evil_messenger', 'evil_sorcerer', 'evil_lancelot',
  ];

  it('returns good for all good roles', () => {
    for (const role of goodRoles) {
      expect(teamForRole(role)).toBe('good');
    }
  });

  it('returns evil for all evil roles', () => {
    for (const role of evilRoles) {
      expect(teamForRole(role)).toBe('evil');
    }
  });
});

describe('totalGoodMessages / totalEvilMessages', () => {
  function makeState(quests: Partial<Quest>[]): FullGameState {
    return {
      game: { id: 'g1', created_at: '', finished_at: null, current_quest: 1, namespace: 'SGW' },
      players: [],
      known_players: [],
      roles: [],
      modules: [],
      quests: quests.map(q => ({ quest: makeQuest(q), rounds: [] })),
      lady_holders: [],
      lady_investigations: [],
      lancelot_switches: [],
      plot_cards: [],
      assassination_attempts: [],
      notes: [],
    };
  }

  it('sums good messages across quests', () => {
    const state = makeState([
      { good_message_count: 1 },
      { good_message_count: 2 },
      { good_message_count: null },
    ]);
    expect(totalGoodMessages(state)).toBe(3);
  });

  it('sums evil messages across quests', () => {
    const state = makeState([
      { evil_message_count: 1 },
      { evil_message_count: 1 },
    ]);
    expect(totalEvilMessages(state)).toBe(2);
  });
});

describe('hasSorcerers / hasMessengers', () => {
  function makeState(roles: string[]): FullGameState {
    return {
      game: { id: 'g1', created_at: '', finished_at: null, current_quest: 1, namespace: 'SGW' },
      players: [],
      known_players: [],
      roles: roles.map(r => ({ id: r, game_id: 'g1', role: r as Role })),
      modules: [],
      quests: [],
      lady_holders: [],
      lady_investigations: [],
      lancelot_switches: [],
      plot_cards: [],
      assassination_attempts: [],
      notes: [],
    };
  }

  it('detects sorcerers', () => {
    expect(hasSorcerers(makeState(['good_sorcerer']))).toBe(true);
    expect(hasSorcerers(makeState(['evil_sorcerer']))).toBe(true);
    expect(hasSorcerers(makeState(['merlin']))).toBe(false);
  });

  it('detects messengers', () => {
    expect(hasMessengers(makeState(['senior_messenger']))).toBe(true);
    expect(hasMessengers(makeState(['evil_messenger']))).toBe(true);
    expect(hasMessengers(makeState(['merlin']))).toBe(false);
  });
});
