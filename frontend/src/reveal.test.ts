import { describe, it, expect } from 'vitest';
import { buildRevealScript, pauseMs, PAUSE_SHORT_MS } from './reveal';
import type { Role } from './types';

// Join the script's text lines for easy substring assertions.
const textOf = (roles: Role[]) => buildRevealScript(roles).map((l) => l.text);
const joined = (roles: Role[]) => textOf(roles).join('\n');

describe('buildRevealScript', () => {
  it('always brackets the script with the open/close lines', () => {
    const lines = textOf(['merlin', 'minion_of_mordred']);
    expect(lines[0]).toMatch(/close your eyes and extend/i);
    expect(lines[lines.length - 1]).toBe('Everyone, open your eyes.');
  });

  it('base Merlin + Minions: evil see each other, Merlin sees evil, no extra parts', () => {
    const s = joined(['merlin', 'loyal_servant', 'loyal_servant', 'minion_of_mordred', 'minion_of_mordred']);
    expect(s).toMatch(/Minions of Mordred, open your eyes and look around/);
    expect(s).toMatch(/Merlin, open your eyes and see the agents of Evil/);
    expect(s).not.toMatch(/Percival/);
    expect(s).not.toMatch(/Cleric/);
    expect(s).not.toMatch(/Messenger/);
  });

  it('Mordred is excluded from the thumb-extend for Merlin', () => {
    const s = joined(['merlin', 'mordred', 'minion_of_mordred']);
    expect(s).toMatch(/Minions of Mordred, except Mordred, extend your thumb so Merlin/);
  });

  it('Oberon is excluded from evil-sees-evil but NOT from Merlin', () => {
    const s = joined(['merlin', 'oberon', 'minion_of_mordred']);
    expect(s).toMatch(/Minions of Mordred, except Oberon, open your eyes/);
    // Merlin's extend line has no Oberon exclusion
    expect(s).toMatch(/Minions of Mordred, extend your thumb so Merlin/);
  });

  it('Percival sees Merlin, and Morgana too when present', () => {
    const withMorgana = joined(['merlin', 'percival', 'morgana', 'minion_of_mordred']);
    expect(withMorgana).toMatch(/Merlin and Morgana, extend your thumb so Percival/);
    expect(withMorgana).toMatch(/Percival, open your eyes and see Merlin and Morgana/);

    const noMorgana = joined(['merlin', 'percival', 'minion_of_mordred']);
    expect(noMorgana).toMatch(/Merlin, extend your thumb so Percival/);
    expect(noMorgana).not.toMatch(/Morgana/);
  });

  it('Untrustworthy Servant extends with the minions for Merlin', () => {
    const s = joined(['merlin', 'untrustworthy_servant', 'minion_of_mordred']);
    expect(s).toMatch(/and the Untrustworthy Servant, extend your thumb so Merlin/);
    expect(s).toMatch(/and the Untrustworthy Servant, re-form your hand/);
  });

  it('Cleric block appears only with Cleric', () => {
    expect(joined(['merlin', 'cleric', 'minion_of_mordred'])).toMatch(/Cleric, open your eyes/);
    expect(joined(['merlin', 'minion_of_mordred'])).not.toMatch(/Cleric/);
  });

  it('Messenger block requires both Senior and Junior', () => {
    expect(joined(['senior_messenger', 'junior_messenger', 'minion_of_mordred'])).toMatch(/Senior Messenger, open your eyes/);
    expect(joined(['senior_messenger', 'minion_of_mordred'])).not.toMatch(/Messenger, open/);
  });

  it('omits Merlin block when Merlin is absent', () => {
    const s = joined(['loyal_servant', 'minion_of_mordred']);
    expect(s).not.toMatch(/Merlin/);
    expect(s).toMatch(/Minions of Mordred, open your eyes/);
  });

  it('never mentions the Assassin (dropped from reveal)', () => {
    const s = joined(['merlin', 'assassin', 'untrustworthy_servant', 'minion_of_mordred']);
    expect(s).not.toMatch(/Assassin/);
  });

  it('assigns long pauses to active lines and short to passive', () => {
    const script = buildRevealScript(['merlin', 'minion_of_mordred']);
    const look = script.find((l) => /open your eyes and look around/.test(l.text));
    const close = script.find((l) => l.text === 'Minions of Mordred, close your eyes.');
    expect(look?.pause).toBe('long');
    expect(close?.pause).toBe('short');
  });
});

describe('pauseMs', () => {
  it('scales from the short base (short=2s, medium=1.5x, long=2x)', () => {
    expect(pauseMs('short')).toBe(PAUSE_SHORT_MS);
    expect(pauseMs('medium')).toBe(PAUSE_SHORT_MS * 1.5);
    expect(pauseMs('long')).toBe(PAUSE_SHORT_MS * 2);
  });
});
