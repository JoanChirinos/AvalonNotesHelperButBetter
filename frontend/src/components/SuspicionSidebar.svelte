<script lang="ts">
  import type { FullGameState, Role } from '../types';
  import { playerNameById } from '../derived';
  import { ROLE_DISPLAY_NAMES } from '../constants';
  import { teamForRole } from '../constants';
  import { Wand, Sword, EyeOff, Mail, Sparkles, ShieldUser, ShieldQuestionMark, Circle, X } from 'lucide-svelte';

  interface Props {
    gameState: FullGameState;
  }

  let { gameState }: Props = $props();

  let expandedPlayer = $state<string | null>(null);

  let players = $derived(gameState.players);
  let gameId = $derived(gameState.game.id);

  // localStorage persistence
  type Suspicions = Record<string, Role[]>;

  function loadSuspicions(): Suspicions {
    try {
      const raw = localStorage.getItem(`anh_suspicions_${gameId}`);
      return raw ? JSON.parse(raw) : {};
    } catch { return {}; }
  }

  function saveSuspicions(s: Suspicions) {
    localStorage.setItem(`anh_suspicions_${gameId}`, JSON.stringify(s));
  }

  let suspicions = $state<Suspicions>(loadSuspicions());

  // Re-load when gameId changes
  $effect(() => {
    suspicions = loadSuspicions();
  });

  function toggleSuspicion(playerId: string, role: Role) {
    const current = suspicions[playerId] ?? [];
    if (current.includes(role)) {
      suspicions[playerId] = current.filter(r => r !== role);
      if (suspicions[playerId].length === 0) delete suspicions[playerId];
    } else {
      suspicions[playerId] = [...current, role];
    }
    suspicions = { ...suspicions };
    saveSuspicions(suspicions);
  }

  function clearPlayer(playerId: string) {
    delete suspicions[playerId];
    suspicions = { ...suspicions };
    saveSuspicions(suspicions);
  }

  // Good roles: special icons first, then others
  const GOOD_ICON_ROLES: Role[] = ['merlin', 'percival', 'loyal_servant', 'untrustworthy_servant', 'senior_messenger', 'junior_messenger', 'good_sorcerer'];
  const GOOD_OTHER_ROLES: Role[] = ['cleric', 'troublemaker', 'good_lancelot'];

  // Evil roles: special icons first, then others
  const EVIL_ICON_ROLES: Role[] = ['morgana', 'mordred', 'minion_of_mordred', 'evil_messenger', 'evil_sorcerer'];
  const EVIL_OTHER_ROLES: Role[] = ['assassin', 'oberon', 'trickster', 'brute', 'lunatic', 'revealer', 'evil_lancelot'];
</script>

<div class="bg-base-100 border-r border-base-300 p-3 w-56 overflow-y-auto h-full">
  <div class="mb-3">
    <span class="text-xs font-semibold uppercase text-base-content/60">Suspicions</span>
  </div>

  <div class="space-y-2">
    {#each players as player}
      {@const playerSuspicions = suspicions[player.id] ?? []}
      <div class="rounded-lg bg-base-200 px-2 py-1.5">
        <div
          class="flex items-center justify-between cursor-pointer"
          onclick={() => expandedPlayer = expandedPlayer === player.id ? null : player.id}
          role="button"
          tabindex="0"
          onkeydown={(e) => { if (e.key === 'Enter') expandedPlayer = expandedPlayer === player.id ? null : player.id; }}
        >
          <span class="text-xs font-medium">{playerNameById(gameState, player.id)}</span>
          {#if playerSuspicions.length > 0}
            <button class="btn btn-ghost btn-xs opacity-0 group-hover:opacity-100" onclick={(e) => { e.stopPropagation(); clearPlayer(player.id); }}>
              <X size={10} />
            </button>
          {/if}
        </div>

        <!-- Current suspicion icons (compact, no labels) -->
        {#if playerSuspicions.length > 0}
          <div class="flex flex-wrap gap-1 mt-1">
            {#each playerSuspicions as role}
              {@const color = teamForRole(role) === 'good' ? 'text-success' : 'text-error'}
              <span class="{color}" title={ROLE_DISPLAY_NAMES[role]}>
                {#if role === 'merlin' || role === 'morgana'}
                  <Wand size={12} />
                {:else if role === 'percival'}
                  <Sword size={12} />
                {:else if role === 'mordred'}
                  <EyeOff size={12} />
                {:else if role === 'senior_messenger' || role === 'junior_messenger' || role === 'evil_messenger'}
                  <Mail size={12} />
                {:else if role === 'good_sorcerer' || role === 'evil_sorcerer'}
                  <Sparkles size={12} />
                {:else if role === 'loyal_servant' || role === 'minion_of_mordred'}
                  <ShieldUser size={12} />
                {:else if role === 'untrustworthy_servant'}
                  <ShieldQuestionMark size={12} />
                {:else}
                  <Circle size={8} class="fill-current" />
                {/if}
              </span>
            {/each}
          </div>
        {/if}

        <!-- Expanded role picker -->
        {#if expandedPlayer === player.id}
          <div class="mt-2 space-y-2">
            <!-- Good roles -->
            <div class="flex flex-wrap gap-1">
              {#each GOOD_ICON_ROLES as role}
                {@const active = playerSuspicions.includes(role)}
                <button
                  class="btn btn-xs btn-ghost text-success"
                  class:bg-base-300={active}
                  class:ring-1={active}
                  class:ring-success={active}
                  onclick={() => toggleSuspicion(player.id, role)}
                >
                  {#if role === 'merlin'}
                    <Wand size={12} />
                  {:else if role === 'percival'}
                    <Sword size={12} />
                  {:else if role === 'senior_messenger' || role === 'junior_messenger'}
                    <Mail size={12} />
                  {:else if role === 'good_sorcerer'}
                    <Sparkles size={12} />
                  {:else if role === 'loyal_servant'}
                    <ShieldUser size={12} />
                  {:else if role === 'untrustworthy_servant'}
                    <ShieldQuestionMark size={12} />
                  {/if}
                  <span class="text-[10px]">{ROLE_DISPLAY_NAMES[role]}</span>
                </button>
              {/each}
              {#each GOOD_OTHER_ROLES as role}
                {@const active = playerSuspicions.includes(role)}
                <button
                  class="btn btn-xs btn-ghost text-success"
                  class:bg-base-300={active}
                  class:ring-1={active}
                  class:ring-success={active}
                  onclick={() => toggleSuspicion(player.id, role)}
                >
                  <Circle size={6} class="fill-current" />
                  <span class="text-[10px]">{ROLE_DISPLAY_NAMES[role]}</span>
                </button>
              {/each}
            </div>
            <!-- Evil roles -->
            <div class="flex flex-wrap gap-1">
              {#each EVIL_ICON_ROLES as role}
                {@const active = playerSuspicions.includes(role)}
                <button
                  class="btn btn-xs btn-ghost text-error"
                  class:bg-base-300={active}
                  class:ring-1={active}
                  class:ring-error={active}
                  onclick={() => toggleSuspicion(player.id, role)}
                >
                  {#if role === 'morgana'}
                    <Wand size={12} />
                  {:else if role === 'mordred'}
                    <EyeOff size={12} />
                  {:else if role === 'evil_messenger'}
                    <Mail size={12} />
                  {:else if role === 'evil_sorcerer'}
                    <Sparkles size={12} />
                  {:else if role === 'minion_of_mordred'}
                    <ShieldUser size={12} />
                  {/if}
                  <span class="text-[10px]">{ROLE_DISPLAY_NAMES[role]}</span>
                </button>
              {/each}
              {#each EVIL_OTHER_ROLES as role}
                {@const active = playerSuspicions.includes(role)}
                <button
                  class="btn btn-xs btn-ghost text-error"
                  class:bg-base-300={active}
                  class:ring-1={active}
                  class:ring-error={active}
                  onclick={() => toggleSuspicion(player.id, role)}
                >
                  <Circle size={6} class="fill-current" />
                  <span class="text-[10px]">{ROLE_DISPLAY_NAMES[role]}</span>
                </button>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>
