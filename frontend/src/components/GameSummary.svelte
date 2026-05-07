<script lang="ts">
  import { api } from '../api';
  import type { FullGameState, Role, SnipeType } from '../types';
  import { deriveQuestResult, playerNameById, teamForRole } from '../derived';
  import { ROLE_DISPLAY_NAMES, GOOD_ROLES, EVIL_ROLES } from '../constants';
  import QuestTrack from './QuestTrack.svelte';
  import QuestCard from './QuestCard.svelte';
  import { Crown, Frown } from 'lucide-svelte';

  interface Props {
    gameState: FullGameState;
    onNavigate: (path: string) => void;
  }

  let { gameState, onNavigate }: Props = $props();

  let players = $derived(gameState.players);
  let roles = $derived(gameState.roles);
  let sortedRoles = $derived(
    [...gameState.roles].sort((a, b) => {
      const order = [...GOOD_ROLES, ...EVIL_ROLES];
      return order.indexOf(a.role) - order.indexOf(b.role);
    })
  );
  let attempts = $derived(gameState.assassination_attempts);

  let failedQuests = $derived(
    gameState.quests.filter(q => deriveQuestResult(q.quest, players.length) === 'fail').length
  );
  let succeededQuests = $derived(
    gameState.quests.filter(q => deriveQuestResult(q.quest, players.length) === 'success').length
  );
  let assassinationNeeded = $derived(succeededQuests >= 3);

  // Derive game result
  let gameResult = $derived.by(() => {
    if (failedQuests >= 3) return 'evil';
    if (!assassinationNeeded) return null;
    const phase2 = attempts.find(a => a.phase === 2);
    if (!phase2) return null;
    return phase2.correct ? 'evil' : 'good';
  });

  // Check if untrustworthy servant is in the game
  let hasUntrustworthyServant = $derived(
    roles.some(r => r.role === 'untrustworthy_servant')
  );
  let hasMessengers = $derived(
    roles.some(r => r.role === 'senior_messenger')
  );

  // All quests with rounds for history
  let historyQuests = $derived(
    gameState.quests.filter(q => q.rounds.length > 0)
  );

  // Assassination state
  let phase1Sniper = $state('');
  let phase1Target = $state('');
  let phase1Correct = $state(false);

  let phase2Sniper = $state('');
  let phase2Type = $state<SnipeType>('merlin');
  let phase2Target1 = $state('');
  let phase2Target2 = $state(''); // for messengers
  let phase2Correct = $state(false);

  // Role assignment: role_id -> player_id
  let roleAssignments = $state<Map<string, string>>(new Map());
  let initialized = $state(false);

  // Init from existing data (once)
  $effect(() => {
    if (initialized) return;
    initialized = true;

    // Load existing assassination attempts
    const a1 = attempts.find(a => a.phase === 1);
    if (a1) {
      phase1Sniper = a1.sniper_player_id;
      const targets = JSON.parse(a1.target_player_ids) as string[];
      phase1Target = targets[0] ?? '';
      phase1Correct = a1.correct === 1;
    }
    const a2 = attempts.find(a => a.phase === 2);
    if (a2) {
      phase2Sniper = a2.sniper_player_id;
      phase2Type = a2.snipe_type as SnipeType;
      const targets = JSON.parse(a2.target_player_ids) as string[];
      phase2Target1 = targets[0] ?? '';
      phase2Target2 = targets[1] ?? '';
      phase2Correct = a2.correct === 1;
    }

    // Load existing role assignments
    const map = new Map<string, string>();
    for (const p of players) {
      if (p.role) {
        const roleEntry = roles.find(r => r.role === p.role && !map.has(r.id));
        if (roleEntry) map.set(roleEntry.id, p.id);
      }
    }
    roleAssignments = map;
  });

  async function saveAssassination(phase: number) {
    try {
      if (phase === 1) {
        await api.createAssassinationAttempt(gameState.game.id, {
          phase: 1,
          sniper_player_id: phase1Sniper,
          snipe_type: 'untrustworthy_servant',
          target_player_ids: [phase1Target],
          correct: phase1Correct,
        });
      } else {
        const targets = phase2Type === 'messengers'
          ? [phase2Target1, phase2Target2]
          : [phase2Target1];
        await api.createAssassinationAttempt(gameState.game.id, {
          phase: 2,
          sniper_player_id: phase2Sniper,
          snipe_type: phase2Type,
          target_player_ids: targets,
          correct: phase2Correct,
        });
      }
    } catch (e) {
      console.error(e);
    }
  }

  async function assignRole(roleId: string, playerId: string) {
    // Unassign previous player for this role
    const prevPlayerId = roleAssignments.get(roleId);
    if (prevPlayerId) {
      await api.updatePlayer(gameState.game.id, prevPlayerId, { clear_role: true });
    }

    roleAssignments.set(roleId, playerId);
    roleAssignments = new Map(roleAssignments);

    const roleEntry = roles.find(r => r.id === roleId);
    if (!roleEntry) return;

    try {
      await api.updatePlayer(gameState.game.id, playerId, { role: roleEntry.role });
    } catch (e) {
      console.error(e);
    }
  }

  async function unassignRole(roleId: string) {
    const prevPlayerId = roleAssignments.get(roleId);
    if (prevPlayerId) {
      try {
        await api.updatePlayer(gameState.game.id, prevPlayerId, { clear_role: true });
      } catch (e) {
        console.error(e);
      }
    }
    roleAssignments.delete(roleId);
    roleAssignments = new Map(roleAssignments);
  }

  // Players already assigned to a role
  let assignedPlayerIds = $derived(new Set(roleAssignments.values()));

  function availablePlayersForRole(roleId: string): typeof players {
    const currentAssignment = roleAssignments.get(roleId);
    return players.filter(p => p.id === currentAssignment || !assignedPlayerIds.has(p.id));
  }

  function playerWon(role: Role): boolean | null {
    const result = gameResult;
    if (!result) return null;
    const team = teamForRole(role);

    // Special case: sniped untrustworthy servant counts as evil
    if (role === 'untrustworthy_servant') {
      const a1 = attempts.find(a => a.phase === 1);
      if (a1 && a1.correct === 1) {
        return result === 'evil';
      }
    }

    return (team === 'good' && result === 'good') || (team === 'evil' && result === 'evil');
  }
</script>

<QuestTrack {gameState} />

<div class="space-y-6">
  <!-- Quest/Round History (read-only) -->
  {#if historyQuests.length > 0}
    <div class="flex flex-wrap justify-center gap-3 px-4">
      {#each historyQuests as questState}
        <QuestCard {gameState} {questState} />
      {/each}
    </div>
  {/if}

  <div class="max-w-2xl mx-auto space-y-6">
    <!-- Assassination -->
    {#if assassinationNeeded}
      <div class="card bg-base-100 shadow-sm">
        <div class="card-body">
          <h3 class="card-title">Assassination</h3>

          <!-- Phase 1: Untrustworthy Servant -->
          {#if hasUntrustworthyServant}
            <div class="bg-base-200 rounded-lg p-3 space-y-2">
              <div class="flex flex-wrap items-center gap-2 text-sm">
                <select class="select select-bordered select-sm" bind:value={phase1Sniper}>
                  <option value="" disabled>Sniper</option>
                  {#each players as p}
                    <option value={p.id}>{playerNameById(gameState, p.id)}</option>
                  {/each}
                </select>
                <span>guessed</span>
                <select class="select select-bordered select-sm" bind:value={phase1Target}>
                  <option value="" disabled>Target</option>
                  {#each players as p}
                    <option value={p.id}>{playerNameById(gameState, p.id)}</option>
                  {/each}
                </select>
                <span>as Untrustworthy Servant.</span>
                <label class="label cursor-pointer gap-1">
                  <span>Correct?</span>
                  <input type="checkbox" class="checkbox checkbox-sm" bind:checked={phase1Correct} />
                </label>
                <button class="btn btn-xs btn-primary" onclick={() => saveAssassination(1)}>Save</button>
              </div>
            </div>
          {/if}

          <!-- Phase 2: Merlin or Messengers -->
          <div class="bg-base-200 rounded-lg p-3 space-y-2">
            <div class="flex gap-2 mb-2">
              <label class="label cursor-pointer gap-1">
                <input type="radio" class="radio radio-sm" value="merlin" bind:group={phase2Type} />
                <span class="text-sm">Merlin</span>
              </label>
              {#if hasMessengers}
                <label class="label cursor-pointer gap-1">
                  <input type="radio" class="radio radio-sm" value="messengers" bind:group={phase2Type} />
                  <span class="text-sm">Messengers</span>
                </label>
              {/if}
            </div>
            <div class="flex flex-wrap items-center gap-2 text-sm">
              <select class="select select-bordered select-sm" bind:value={phase2Sniper}>
                <option value="" disabled>Sniper</option>
                {#each players as p}
                  <option value={p.id}>{playerNameById(gameState, p.id)}</option>
                {/each}
              </select>
              <span>guessed</span>
              <select class="select select-bordered select-sm" bind:value={phase2Target1}>
                <option value="" disabled>Target</option>
                {#each players as p}
                  <option value={p.id}>{playerNameById(gameState, p.id)}</option>
                {/each}
              </select>
              {#if phase2Type === 'messengers'}
                <span>and</span>
                <select class="select select-bordered select-sm" bind:value={phase2Target2}>
                  <option value="" disabled>Target</option>
                  {#each players as p}
                    <option value={p.id}>{playerNameById(gameState, p.id)}</option>
                  {/each}
                </select>
              {/if}
              <span>as {phase2Type === 'messengers' ? 'Messengers' : 'Merlin'}.</span>
              <label class="label cursor-pointer gap-1">
                <span>Correct?</span>
                <input type="checkbox" class="checkbox checkbox-sm" bind:checked={phase2Correct} />
              </label>
              <button class="btn btn-xs btn-primary" onclick={() => saveAssassination(2)}>Save</button>
            </div>
          </div>
        </div>
      </div>
    {/if}

    <!-- Role Assignment -->
    <div class="card bg-base-100 shadow-sm">
      <div class="card-body">
        <h3 class="card-title">Role Assignment</h3>
        <div class="space-y-2">
          {#each sortedRoles as role}
            {@const assignedPlayerId = roleAssignments.get(role.id)}
            {@const won = role.role && assignedPlayerId ? playerWon(role.role) : null}
            <div class="flex items-center gap-3">
              <span class="text-sm font-medium w-40 badge" class:badge-success={teamForRole(role.role) === 'good'} class:badge-error={teamForRole(role.role) === 'evil'}>
                {ROLE_DISPLAY_NAMES[role.role]}
              </span>
              <select
                class="select select-bordered select-sm flex-1"
                value={assignedPlayerId ?? ''}
                onchange={(e) => {
                  const target = e.target as HTMLSelectElement;
                  if (target.value) {
                    assignRole(role.id, target.value);
                  } else {
                    unassignRole(role.id);
                  }
                }}
              >
                <option value="">—</option>
                {#each availablePlayersForRole(role.id) as p}
                  <option value={p.id}>{playerNameById(gameState, p.id)}</option>
                {/each}
              </select>
              {#if won === true}
                <Crown size={18} class="text-warning" />
              {:else if won === false}
                <Frown size={18} class="text-base-content/30" />
              {/if}
            </div>
          {/each}
        </div>
      </div>
    </div>

    <!-- Game Result -->
    {#if gameResult}
      <div class="card shadow-sm" class:bg-success={gameResult === 'good'} class:bg-error={gameResult === 'evil'}>
        <div class="card-body items-center text-center py-4">
          <h3 class="text-xl font-bold" class:text-success-content={gameResult === 'good'} class:text-error-content={gameResult === 'evil'}>
            {gameResult === 'good' ? 'Good Wins!' : 'Evil Wins!'}
          </h3>
        </div>
      </div>
    {/if}
  </div>
</div>
