<script lang="ts">
  import { api } from '../api';
  import type { FullGameState, RoundState, Vote } from '../types';
  import { questSize } from '../constants';
  import { playerNameById, hasSorcerers, hasMessengers, resultFromCounts } from '../derived';
  import { Check, X, Pencil } from 'lucide-svelte';

  interface Props {
    gameState: FullGameState;
    roundState: RoundState;
    questNumber: number;
    questId: string;
    onCancel: () => void;
  }

  let { gameState, roundState, questNumber, questId, onCancel }: Props = $props();

  let error = $state('');

  let players = $derived(gameState.players);
  let round = $derived(roundState.round);
  let teamIds = $derived(new Set(roundState.team.map(t => t.player_id)));
  let voteMap = $derived(new Map(roundState.votes.map(v => [v.player_id, v.vote])));
  let isApproved = $derived(round.status === 'approved');

  let qSize = $derived(questSize(players.length, questNumber));
  let showSorcerers = $derived(hasSorcerers(gameState));
  let showMessengers = $derived(hasMessengers(gameState));

  let quest = $derived(gameState.quests.find(q => q.quest.quest_number === questNumber)?.quest);

  // Card counts — initialized from quest data
  let successCount = $state(0);
  let failCount = $state(0);
  let magicCount = $state(0);
  let goodMsgCount = $state(0);
  let evilMsgCount = $state(0);

  // Sync card counts from server state
  $effect(() => {
    if (quest) {
      successCount = quest.success_count ?? 0;
      failCount = quest.fail_count ?? 0;
      magicCount = quest.magic_count ?? 0;
      goodMsgCount = quest.good_message_count ?? 0;
      evilMsgCount = quest.evil_message_count ?? 0;
    }
  });

  // Preview result (shares deriveQuestResult via resultFromCounts, incl. quest-5 message backup)
  let previewResult = $derived.by(() => {
    if (!isApproved) return null;
    return resultFromCounts(gameState, questNumber, {
      success: successCount, fail: failCount, magic: magicCount, good: goodMsgCount, evil: evilMsgCount,
    });
  });

  async function setKing(playerId: string) {
    try {
      await api.updateRound(gameState.game.id, round.id, { leader_player_id: playerId });
    } catch (e) { error = String(e); }
  }

  async function toggleTeam(playerId: string) {
    try {
      const newTeam = teamIds.has(playerId)
        ? [...teamIds].filter(id => id !== playerId)
        : [...teamIds, playerId];
      await api.updateRound(gameState.game.id, round.id, { team_player_ids: newTeam });
    } catch (e) { error = String(e); }
  }

  async function toggleVote(playerId: string) {
    try {
      const currentVote = voteMap.get(playerId);
      const newVote: Vote = currentVote === 'approve' ? 'reject' : 'approve';
      const newVotes = players.map(p => ({
        player_id: p.id,
        vote: (p.id === playerId ? newVote : (voteMap.get(p.id) ?? 'reject')) as Vote,
      }));
      await api.recordVotes(gameState.game.id, round.id, { votes: newVotes });
    } catch (e) { error = String(e); }
  }

  async function updateCardCounts() {
    try {
      const result = resultFromCounts(gameState, questNumber, {
        success: successCount, fail: failCount, magic: magicCount, good: goodMsgCount, evil: evilMsgCount,
      });

      await api.updateQuest(gameState.game.id, questId, {
        result: result ?? undefined,
        success_count: successCount,
        fail_count: failCount,
        magic_count: showSorcerers ? magicCount : undefined,
        good_message_count: showMessengers ? goodMsgCount : undefined,
        evil_message_count: showMessengers ? evilMsgCount : undefined,
      });
    } catch (e) { error = String(e); }
  }
</script>

<div class="rounded-lg border-2 border-info p-3 bg-base-100 space-y-3">
  <div class="flex items-center justify-between">
    <span class="text-xs font-semibold text-info uppercase flex items-center gap-1">
      <Pencil size={12} /> Editing R{round.round_number}
    </span>
    <button class="btn btn-ghost btn-xs" onclick={onCancel}><X size={14} /></button>
  </div>

  {#if error}
    <div class="alert alert-error alert-sm">
      <span class="text-xs">{error}</span>
      <button class="btn btn-ghost btn-xs" onclick={() => error = ''}><X size={12} /></button>
    </div>
  {/if}

  <!-- King -->
  <div>
    <p class="text-xs font-semibold text-base-content/60 mb-1 uppercase">King</p>
    <div class="flex flex-wrap gap-1">
      {#each players as player}
        <button
          class="btn btn-xs"
          class:btn-warning={round.leader_player_id === player.id}
          class:btn-outline={round.leader_player_id !== player.id}
          onclick={() => setKing(player.id)}
        >
          {playerNameById(gameState, player.id)}
        </button>
      {/each}
    </div>
  </div>

  <!-- Team -->
  <div>
    <p class="text-xs font-semibold text-base-content/60 mb-1 uppercase">
      Team
      {#if qSize}
        <span class="badge badge-xs" class:badge-success={teamIds.size === qSize} class:badge-error={teamIds.size !== qSize}>
          {teamIds.size} / {qSize}
        </span>
      {/if}
    </p>
    <div class="flex flex-wrap gap-1">
      {#each players as player}
        <button
          class="btn btn-xs"
          class:btn-primary={teamIds.has(player.id)}
          class:btn-outline={!teamIds.has(player.id)}
          onclick={() => toggleTeam(player.id)}
        >
          {playerNameById(gameState, player.id)}
        </button>
      {/each}
    </div>
  </div>

  <!-- Votes -->
  <div>
    <p class="text-xs font-semibold text-base-content/60 mb-1 uppercase">Votes</p>
    <div class="flex flex-wrap gap-1">
      {#each players as player}
        {@const vote = voteMap.get(player.id) ?? 'reject'}
        <button
          class="btn btn-xs"
          class:btn-success={vote === 'approve'}
          class:btn-error={vote === 'reject'}
          onclick={() => toggleVote(player.id)}
        >
          {playerNameById(gameState, player.id)}
          {#if vote === 'approve'}<Check size={12} class="inline" />{:else}<X size={12} class="inline" />{/if}
        </button>
      {/each}
    </div>
  </div>

  <!-- Card counts (approved rounds only) -->
  {#if isApproved}
    <div>
      <p class="text-xs font-semibold text-base-content/60 mb-1 uppercase">Cards Played</p>
      <div class="grid grid-cols-2 gap-2 max-w-sm">
        <div class="flex items-center justify-between bg-base-200 rounded-lg px-2 py-1">
          <span class="text-xs">Success</span>
          <div class="flex items-center gap-1">
            <button class="btn btn-xs btn-outline" onclick={() => successCount = Math.max(0, successCount - 1)}>−</button>
            <span class="badge badge-sm w-6 justify-center">{successCount}</span>
            <button class="btn btn-xs btn-outline" onclick={() => successCount++}>+</button>
          </div>
        </div>
        <div class="flex items-center justify-between bg-base-200 rounded-lg px-2 py-1">
          <span class="text-xs">Fail</span>
          <div class="flex items-center gap-1">
            <button class="btn btn-xs btn-outline" onclick={() => failCount = Math.max(0, failCount - 1)}>−</button>
            <span class="badge badge-sm w-6 justify-center">{failCount}</span>
            <button class="btn btn-xs btn-outline" onclick={() => failCount++}>+</button>
          </div>
        </div>
        {#if showMessengers}
          <div class="flex items-center justify-between bg-base-200 rounded-lg px-2 py-1">
            <span class="text-xs">Good Msg</span>
            <div class="flex items-center gap-1">
              <button class="btn btn-xs btn-outline" onclick={() => goodMsgCount = Math.max(0, goodMsgCount - 1)}>−</button>
              <span class="badge badge-sm w-6 justify-center">{goodMsgCount}</span>
              <button class="btn btn-xs btn-outline" onclick={() => goodMsgCount++}>+</button>
            </div>
          </div>
          <div class="flex items-center justify-between bg-base-200 rounded-lg px-2 py-1">
            <span class="text-xs">Evil Msg</span>
            <div class="flex items-center gap-1">
              <button class="btn btn-xs btn-outline" onclick={() => evilMsgCount = Math.max(0, evilMsgCount - 1)}>−</button>
              <span class="badge badge-sm w-6 justify-center">{evilMsgCount}</span>
              <button class="btn btn-xs btn-outline" onclick={() => evilMsgCount++}>+</button>
            </div>
          </div>
        {/if}
        {#if showSorcerers}
          <div class="flex items-center justify-between bg-base-200 rounded-lg px-2 py-1">
            <span class="text-xs">Magic</span>
            <div class="flex items-center gap-1">
              <button class="btn btn-xs btn-outline" onclick={() => magicCount = Math.max(0, magicCount - 1)}>−</button>
              <span class="badge badge-sm w-6 justify-center">{magicCount}</span>
              <button class="btn btn-xs btn-outline" onclick={() => magicCount++}>+</button>
            </div>
          </div>
        {/if}
      </div>

      {#if previewResult}
        <div class="mt-2 flex items-center gap-2">
          <span class="badge badge-sm" class:badge-success={previewResult === 'success'} class:badge-error={previewResult === 'fail'}>
            {previewResult === 'success' ? 'Pass' : 'Fail'}
          </span>
          <button class="btn btn-xs btn-primary" onclick={updateCardCounts}>Update</button>
        </div>
      {/if}
    </div>
  {/if}

  <div class="flex justify-end">
    <button class="btn btn-xs btn-ghost" onclick={onCancel}>Done</button>
  </div>
</div>
