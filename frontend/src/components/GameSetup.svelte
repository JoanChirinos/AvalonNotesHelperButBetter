<script lang="ts">
  import { onDestroy } from 'svelte';
  import { api } from '../api';
  import type { FullGameState, KnownPlayer, Role, Module } from '../types';
  import { GOOD_ROLES, EVIL_ROLES, ROLE_DISPLAY_NAMES, ROLE_BUNDLES } from '../constants';
  import { X, Minus, Plus, Swords, GripVertical } from 'lucide-svelte';
  import Sortable from 'sortablejs';

  interface Props {
    gameState: FullGameState;
    onNavigate: (path: string) => void;
  }

  let { gameState, onNavigate }: Props = $props();

  let newPlayerName = $state('');
  let error = $state('');
  let allKnownPlayers = $state<KnownPlayer[]>([]);

  let gameId = $derived(gameState.game.id);
  let players = $derived(gameState.players);
  let knownPlayers = $derived(gameState.known_players);
  let roles = $derived(gameState.roles);
  let modules = $derived(gameState.modules);

  let roleCount = $derived(roles.length);
  let playerCount = $derived(players.length);
  let canStart = $derived(playerCount > 0 && roleCount === playerCount);

  let enabledModules = $derived(new Set(modules.map(m => m.module)));

  // Known players not already in this game
  let availablePlayers = $derived(
    allKnownPlayers.filter(kp => !players.some(p => p.known_player_id === kp.id))
  );

  // Map known_player_id to name for display
  function playerName(knownPlayerId: string): string {
    return knownPlayers.find(kp => kp.id === knownPlayerId)?.name ?? '???';
  }

  async function loadKnownPlayers() {
    try {
      allKnownPlayers = await api.listKnownPlayers();
    } catch (e) {
      error = String(e);
    }
  }

  async function addNewPlayer() {
    const name = newPlayerName.trim();
    if (!name) return;
    try {
      await api.addPlayer(gameId, { name });
      newPlayerName = '';
      await loadKnownPlayers();
    } catch (e) {
      const msg = String(e);
      if (msg.includes('UNIQUE')) {
        error = `Player "${name}" already exists. Use the dropdown to add them.`;
      } else {
        error = msg;
      }
    }
  }

  async function addExistingPlayer(kp: KnownPlayer) {
    try {
      await api.addPlayer(gameId, { known_player_id: kp.id });
    } catch (e) {
      error = String(e);
    }
  }

  async function removePlayer(playerId: string) {
    try {
      await api.deletePlayer(gameId, playerId);
    } catch (e) {
      error = String(e);
    }
  }

  async function addSingleRole(role: Role) {
    try {
      await api.addRole(gameId, role);
    } catch (e) {
      error = String(e);
    }
  }

  async function addRole(role: Role) {
    try {
      for (const [, bundle] of Object.entries(ROLE_BUNDLES)) {
        if (bundle.includes(role)) {
          for (const r of bundle) await api.addRole(gameId, r);
          return;
        }
      }
      await api.addRole(gameId, role);
    } catch (e) {
      error = String(e);
    }
  }

  async function removeRole(role: Role) {
    try {
      for (const [, bundle] of Object.entries(ROLE_BUNDLES)) {
        if (bundle.includes(role)) {
          for (const r of bundle) {
            const entry = roles.find(gr => gr.role === r);
            if (entry) await api.deleteRole(gameId, entry.id);
          }
          return;
        }
      }
      const entry = roles.find(r => r.role === role);
      if (entry) await api.deleteRole(gameId, entry.id);
    } catch (e) {
      error = String(e);
    }
  }

  async function toggleModule(mod: Module) {
    try {
      if (enabledModules.has(mod)) {
        const entry = modules.find(m => m.module === mod);
        if (entry) await api.deleteModule(gameId, entry.id);
      } else {
        await api.addModule(gameId, mod);
      }
    } catch (e) {
      error = String(e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') addNewPlayer();
  }

  function getRoleCount(role: Role): number {
    return roles.filter(r => r.role === role).length;
  }

  async function startGame() {
    try {
      // Create first round of Quest 1 with seat 1 as leader, empty team
      const quest1 = gameState.quests[0];
      const firstLeader = players[Math.floor(Math.random() * players.length)];
      await api.createRound(gameId, quest1.quest.id, {
        leader_player_id: firstLeader.id,
        team_player_ids: [],
      });
    } catch (e) {
      error = String(e);
    }
  }

  let playerListEl: HTMLDivElement;
  let sortableInstance: any = null;

  onDestroy(() => { sortableInstance?.destroy(); });

  async function reorderPlayers(oldIndex: number, newIndex: number) {
    if (oldIndex === newIndex) return;
    try {
      const reordered = [...players];
      const [moved] = reordered.splice(oldIndex, 1);
      reordered.splice(newIndex, 0, moved);
      await api.reorderPlayers(gameId, reordered.map(p => p.id));
    } catch (e) {
      error = String(e);
    }
  }

  $effect(() => {
    if (playerListEl && !sortableInstance) {
      sortableInstance = Sortable.create(playerListEl, {
        animation: 150,
        handle: '.drag-handle',
        onEnd: (evt: any) => {
          // Revert DOM change — let Svelte handle rendering from state
          const parent = evt.from;
          if (evt.oldIndex < evt.newIndex) {
            parent.insertBefore(evt.item, parent.children[evt.oldIndex]);
          } else {
            parent.insertBefore(evt.item, parent.children[evt.oldIndex + 1]);
          }
          if (evt.oldIndex !== undefined && evt.newIndex !== undefined) {
            reorderPlayers(evt.oldIndex, evt.newIndex);
          }
        },
      });
    }
  });

  $effect(() => {
    loadKnownPlayers();
  });
</script>

<div class="max-w-4xl mx-auto">
  <h2 class="text-2xl font-bold mb-6">Game Setup</h2>

  {#if error}
    <div class="alert alert-error mb-4">
      <span>{error}</span>
      <button class="btn btn-ghost btn-xs" onclick={() => error = ''}><X size={14} /></button>
    </div>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
    <!-- Players -->
    <div class="card bg-base-100 shadow-sm">
      <div class="card-body">
        <h3 class="card-title">
          Players
          <span class="badge badge-sm" class:badge-success={canStart} class:badge-error={!canStart}>
            {playerCount}
          </span>
        </h3>

        <!-- Add new player -->
        <div class="flex gap-2 mb-2">
          <input
            type="text"
            class="input input-bordered input-sm flex-1"
            placeholder="New player name"
            bind:value={newPlayerName}
            onkeydown={handleKeydown}
          />
          <button class="btn btn-sm btn-primary" onclick={addNewPlayer}>Add</button>
        </div>

        <!-- Select existing player -->
        {#if availablePlayers.length > 0}
          <div class="mb-3">
            <select
              class="select select-bordered select-sm w-full"
              onchange={(e) => {
                const target = e.target as HTMLSelectElement;
                const kp = availablePlayers.find(p => p.id === target.value);
                if (kp) { addExistingPlayer(kp); target.value = ''; }
              }}
            >
              <option value="" disabled selected>Add existing player...</option>
              {#each availablePlayers as kp}
                <option value={kp.id}>{kp.name}</option>
              {/each}
            </select>
          </div>
        {/if}

        <!-- Current players -->
        <div class="space-y-1" bind:this={playerListEl}>
          {#each players as player, i}
            <div class="flex items-center justify-between bg-base-200 rounded-lg px-3 py-2" data-id={player.id}>
              <div class="flex items-center gap-2">
                <span class="drag-handle cursor-grab active:cursor-grabbing text-base-content/30 hover:text-base-content/60">
                  <GripVertical size={14} />
                </span>
                <span class="text-sm">
                  {playerName(player.known_player_id)}
                </span>
              </div>
              <button
                class="btn btn-ghost btn-xs text-error"
                onclick={() => removePlayer(player.id)}
              ><X size={14} /></button>
            </div>
          {/each}
        </div>
      </div>
    </div>

    <!-- Roles -->
    <div class="card bg-base-100 shadow-sm">
      <div class="card-body">
        <h3 class="card-title">
          Roles
          <span class="badge badge-sm" class:badge-success={canStart} class:badge-error={!canStart}>
            {roleCount} / {playerCount}
          </span>
        </h3>

        <!-- Good Roles (excluding Loyal Servant) -->
        <div class="mb-3">
          <p class="text-xs font-semibold text-success mb-1 uppercase">Good</p>
          <div class="flex flex-wrap gap-1">
            {#each GOOD_ROLES.filter(r => r !== 'loyal_servant') as role}
              {@const count = getRoleCount(role)}
              <button
                class="btn btn-xs"
                class:btn-success={count > 0}
                class:btn-outline={count === 0}
                onclick={() => count > 0 ? removeRole(role) : addRole(role)}
              >
                {ROLE_DISPLAY_NAMES[role]}
              </button>
            {/each}
          </div>
        </div>

        <!-- Evil Roles (excluding Minion of Mordred) -->
        <div class="mb-3">
          <p class="text-xs font-semibold text-error mb-1 uppercase">Evil</p>
          <div class="flex flex-wrap gap-1">
            {#each EVIL_ROLES.filter(r => r !== 'minion_of_mordred') as role}
              {@const count = getRoleCount(role)}
              <button
                class="btn btn-xs"
                class:btn-error={count > 0}
                class:btn-outline={count === 0}
                onclick={() => count > 0 ? removeRole(role) : addRole(role)}
              >
                {ROLE_DISPLAY_NAMES[role]}
              </button>
            {/each}
          </div>
        </div>

        <!-- Counter roles (Loyal Servant / Minion of Mordred) -->
        <div class="flex flex-wrap gap-4 mb-3">
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium text-success">Loyal Servant</span>
            <div class="flex items-center gap-1">
              <button class="btn btn-xs btn-outline" onclick={() => removeRole('loyal_servant')} disabled={getRoleCount('loyal_servant') === 0}><Minus size={14} /></button>
              <span class="badge badge-sm w-8 justify-center">{getRoleCount('loyal_servant')}</span>
              <button class="btn btn-xs btn-outline" onclick={() => addSingleRole('loyal_servant')}><Plus size={14} /></button>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium text-error">Minion of Mordred</span>
            <div class="flex items-center gap-1">
              <button class="btn btn-xs btn-outline" onclick={() => removeRole('minion_of_mordred')} disabled={getRoleCount('minion_of_mordred') === 0}><Minus size={14} /></button>
              <span class="badge badge-sm w-8 justify-center">{getRoleCount('minion_of_mordred')}</span>
              <button class="btn btn-xs btn-outline" onclick={() => addSingleRole('minion_of_mordred')}><Plus size={14} /></button>
            </div>
          </div>
        </div>

        <div>
          <p class="text-xs font-semibold text-info mb-1 uppercase">Modules</p>
          <div class="flex flex-wrap gap-2">
            <label class="label cursor-pointer gap-2">
              <input
                type="checkbox"
                class="checkbox checkbox-sm checkbox-info"
                checked={enabledModules.has('lady_of_the_lake')}
                onchange={() => toggleModule('lady_of_the_lake')}
              />
              <span class="text-sm">Lady of the Lake</span>
            </label>
            <label class="label cursor-pointer gap-2">
              <input
                type="checkbox"
                class="checkbox checkbox-sm checkbox-info"
                checked={enabledModules.has('lancelot_switching')}
                onchange={() => toggleModule('lancelot_switching')}
              />
              <span class="text-sm">Lancelot Switching</span>
            </label>
            <label class="label cursor-pointer gap-2">
              <input
                type="checkbox"
                class="checkbox checkbox-sm checkbox-info"
                checked={enabledModules.has('plot_cards')}
                onchange={() => toggleModule('plot_cards')}
              />
              <span class="text-sm">Plot Cards</span>
            </label>
          </div>
        </div>
      </div>
    </div>
  </div>

  <div class="flex justify-center mt-6">
    <button
      class="btn btn-lg btn-primary"
      disabled={!canStart}
      onclick={startGame}
    >
      <Swords size={18} /> Start Game
      {#if !canStart && playerCount === 0}
        <span class="text-xs opacity-70">(Add players)</span>
      {:else if !canStart}
        <span class="text-xs opacity-70">
          (Need {playerCount - roleCount > 0 ? `${playerCount - roleCount} more role${playerCount - roleCount !== 1 ? 's' : ''}` : `${roleCount - playerCount} fewer role${roleCount - playerCount !== 1 ? 's' : ''}`})
        </span>
      {/if}
    </button>
  </div>
</div>
