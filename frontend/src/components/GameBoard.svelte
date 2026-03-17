<script lang="ts">
  import { api } from '../api';
  import type { FullGameState } from '../types';
  import { deriveQuestResult } from '../derived';
  import QuestTrack from './QuestTrack.svelte';
  import QuestCard from './QuestCard.svelte';
  import CurrentRound from './CurrentRound.svelte';
  import { Flag } from 'lucide-svelte';

  interface Props {
    gameState: FullGameState;
    onNavigate: (path: string) => void;
  }

  let { gameState, onNavigate }: Props = $props();

  let failedQuests = $derived(
    gameState.quests.filter(q => deriveQuestResult(q.quest, gameState.players.length) === 'fail').length
  );
  let succeededQuests = $derived(
    gameState.quests.filter(q => deriveQuestResult(q.quest, gameState.players.length) === 'success').length
  );
  let gameOver = $derived(failedQuests >= 3 || succeededQuests >= 3);

  // All quests that have rounds, but for the current quest exclude the latest round
  let historyQuests = $derived(
    gameState.quests
      .filter(q => q.rounds.length > 0)
      .map(q => {
        if (!gameOver && q.quest.quest_number === gameState.game.current_quest) {
          return { ...q, rounds: q.rounds.slice(0, -1) };
        }
        return q;
      })
      .filter(q => q.rounds.length > 0)
  );

  async function endGame() {
    try {
      await api.updateGame(gameState.game.id, {
        finished_at: new Date().toISOString(),
      });
    } catch (e) {
      console.error(e);
    }
  }
</script>

<QuestTrack {gameState} />

<div class="space-y-4">
  <!-- Quest/Round History -->
  {#if historyQuests.length > 0}
    <div class="flex flex-wrap justify-center gap-3 px-4">
      {#each historyQuests as questState}
        <QuestCard {gameState} {questState} />
      {/each}
    </div>
  {/if}

  {#if gameOver}
    <!-- End Game prompt -->
    <div class="max-w-4xl mx-auto">
      <div class="card bg-base-100 shadow-sm">
        <div class="card-body items-center text-center py-6">
          <h3 class="text-xl font-bold">
            {succeededQuests >= 3 ? 'Good wins the quests!' : 'Evil wins by quests!'}
          </h3>
          {#if succeededQuests >= 3}
            <p class="text-sm text-base-content/60">Proceed to assassination phase and role assignment.</p>
          {/if}
          <button class="btn btn-primary mt-2 gap-2" onclick={endGame}>
            <Flag size={16} /> End Game
          </button>
        </div>
      </div>
    </div>
  {:else}
    <!-- Current Round -->
    <div class="max-w-4xl mx-auto">
      <CurrentRound {gameState} />
    </div>
  {/if}
</div>
