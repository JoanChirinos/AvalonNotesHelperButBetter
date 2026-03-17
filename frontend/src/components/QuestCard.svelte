<script lang="ts">
  import type { FullGameState, QuestState } from '../types';
  import { deriveQuestResult } from '../derived';
  import RoundCard from './RoundCard.svelte';

  interface Props {
    gameState: FullGameState;
    questState: QuestState;
  }

  let { gameState, questState }: Props = $props();

  let quest = $derived(questState.quest);
  let rounds = $derived(questState.rounds);
  let result = $derived(deriveQuestResult(quest, gameState.players.length));
  let isCurrent = $derived(quest.quest_number === gameState.game.current_quest);
</script>

{#if rounds.length > 0}
  <div
    class="card shadow-sm shrink-0"
    class:bg-base-100={!result}
    style:background-color={result === 'success' ? 'oklch(var(--su) / 0.1)' : result === 'fail' ? 'oklch(var(--er) / 0.1)' : ''}
  >
    <div
      class="px-3 py-1.5 rounded-t-2xl text-sm font-semibold"
      class:bg-success={result === 'success'}
      class:text-success-content={result === 'success'}
      class:bg-error={result === 'fail'}
      class:text-error-content={result === 'fail'}
      class:bg-base-300={!result}
    >
      Quest {quest.quest_number}
      {#if result}
        — {result === 'success' ? 'Pass' : 'Fail'}
      {:else if isCurrent}
        — Active
      {/if}
    </div>
    <div class="p-2 space-y-2">
      {#each rounds as roundState}
        <RoundCard
          {gameState}
          {roundState}
          questNumber={quest.quest_number}
          showResult={roundState.round.status === 'approved'}
        />
      {/each}
    </div>
  </div>
{/if}
