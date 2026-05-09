<script lang="ts">
  import type { FullGameState } from '../types';
  import { questSize, failsRequired, ROLE_DISPLAY_NAMES, GOOD_ROLES, EVIL_ROLES } from '../constants';
  import { deriveQuestResult, totalGoodMessages, totalEvilMessages, teamForRole } from '../derived';
  import { Mail, Star } from 'lucide-svelte';

  interface Props {
    gameState: FullGameState;
  }

  let { gameState }: Props = $props();

  let players = $derived(gameState.players);
  let quests = $derived(gameState.quests);
  let goodMsgs = $derived(totalGoodMessages(gameState));
  let evilMsgs = $derived(totalEvilMessages(gameState));
  let showMessages = $derived(gameState.roles.some(r =>
    r.role === 'senior_messenger' || r.role === 'evil_messenger'
  ));

  let sortedRoles = $derived(
    [...gameState.roles].sort((a, b) => {
      const order = [...GOOD_ROLES, ...EVIL_ROLES];
      return order.indexOf(a.role) - order.indexOf(b.role);
    })
  );
</script>

<div class="sticky top-0 z-10 bg-base-100/90 backdrop-blur-sm shadow-sm py-2 mb-4">
  <div class="flex items-center justify-between px-4 gap-4">
    <!-- Roles -->
    <div class="flex flex-wrap gap-0.5">
      {#each sortedRoles as r}
        <span
          class="rounded-full px-1.5 py-0.5 text-[10px] font-medium"
          class:bg-success={teamForRole(r.role) === 'good'}
          class:text-success-content={teamForRole(r.role) === 'good'}
          class:bg-error={teamForRole(r.role) === 'evil'}
          class:text-error-content={teamForRole(r.role) === 'evil'}
        >{ROLE_DISPLAY_NAMES[r.role]}</span>
      {/each}
    </div>

    <!-- Quests + Messages -->
    <div class="flex items-center gap-2 shrink-0">
      {#each quests as qs}
      {@const result = deriveQuestResult(qs.quest, players.length, qs.quest.quest_number === 5 ? { good: goodMsgs, evil: evilMsgs } : undefined)}
      {@const size = questSize(players.length, qs.quest.quest_number)}
      {@const needsTwo = failsRequired(players.length, qs.quest.quest_number) === 2}
      {@const isCurrent = qs.quest.quest_number === gameState.game.current_quest}
      <div
        class="flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium"
        class:bg-success={result === 'success'}
        class:text-success-content={result === 'success'}
        class:bg-error={result === 'fail'}
        class:text-error-content={result === 'fail'}
        class:bg-base-200={!result && !isCurrent}
        class:bg-primary={!result && isCurrent}
        class:text-primary-content={!result && isCurrent}
      >
        {size ?? '?'}{#if needsTwo}<Star size={10} class="inline fill-current" />{/if}
      </div>
    {/each}

    {#if showMessages && (goodMsgs > 0 || evilMsgs > 0)}
      <span class="ml-2 flex gap-0.5 items-center">
        {#each Array(goodMsgs) as _}
          <Mail size={14} class="text-success" />
        {/each}
        {#each Array(evilMsgs) as _}
          <Mail size={14} class="text-error" />
        {/each}
      </span>
    {/if}
    </div>
  </div>
</div>
