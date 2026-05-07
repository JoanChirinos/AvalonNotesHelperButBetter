<script lang="ts">
  import { api } from '../api';
  import type { FullGameState, RoundState, Vote } from '../types';
  import { questSize, failsRequired } from '../constants';
  import { playerNameById, hasSorcerers, hasMessengers, deriveQuestResult } from '../derived';
  import { Check, X, Minus, Plus } from 'lucide-svelte';
  import ToolsModal from './ToolsModal.svelte';

  interface Props {
    gameState: FullGameState;
  }

  let { gameState }: Props = $props();

  let error = $state('');

  let game = $derived(gameState.game);
  let players = $derived(gameState.players);
  let currentQuest = $derived(gameState.quests.find(q => q.quest.quest_number === game.current_quest)!);
  let currentRound = $derived(currentQuest.rounds[currentQuest.rounds.length - 1]);
  let round = $derived(currentRound.round);
  let teamIds = $derived(new Set(currentRound.team.map(t => t.player_id)));
  let voteMap = $derived(new Map(currentRound.votes.map(v => [v.player_id, v.vote])));

  let qSize = $derived(questSize(players.length, currentQuest.quest.quest_number));
  let showSorcerers = $derived(hasSorcerers(gameState));
  let showMessengers = $derived(hasMessengers(gameState));

  // Vote counts
  let approveCount = $derived(currentRound.votes.filter(v => v.vote === 'approve').length);
  let rejectCount = $derived(players.length - approveCount);
  let majorityApproved = $derived(approveCount > players.length / 2);
  let allVoted = $derived(true); // All players always have a vote state (default reject)

  // Card counts (local state for editing before submit)
  let successCount = $state(0);
  let failCount = $state(0);
  let magicCount = $state(0);
  let goodMsgCount = $state(0);
  let evilMsgCount = $state(0);

  // Reset card counts when the round changes
  let lastRoundId = $state('');
  $effect(() => {
    if (round.id !== lastRoundId) {
      lastRoundId = round.id;
      successCount = 0; failCount = 0; magicCount = 0; goodMsgCount = 0; evilMsgCount = 0;
    }
  });

  // Result preview
  let previewResult = $derived.by(() => {
    if (!majorityApproved) return null;
    const fails = failCount + evilMsgCount;
    const threshold = failsRequired(players.length, currentQuest.quest.quest_number);
    let result: 'success' | 'fail' = fails >= threshold ? 'fail' : 'success';
    if (magicCount % 2 === 1) result = result === 'success' ? 'fail' : 'success';
    return result;
  });

  async function setKing(playerId: string) {
    try {
      await api.updateRound(game.id, round.id, { leader_player_id: playerId });
    } catch (e) { error = String(e); }
  }

  async function toggleTeam(playerId: string) {
    try {
      const newTeam = teamIds.has(playerId)
        ? [...teamIds].filter(id => id !== playerId)
        : [...teamIds, playerId];
      await api.updateRound(game.id, round.id, { team_player_ids: newTeam });
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
      await api.recordVotes(game.id, round.id, { votes: newVotes });
    } catch (e) { error = String(e); }
  }

  let submitting = $state(false);

  async function submit() {
    if (submitting) return;
    submitting = true;
    try {
      if (majorityApproved) {
        // Approve round
        await api.updateRound(game.id, round.id, { status: 'approved' });
        // Save card counts + result
        const result = previewResult;
        await api.updateQuest(game.id, currentQuest.quest.id, {
          result: result ?? undefined,
          success_count: successCount,
          fail_count: failCount,
          magic_count: showSorcerers ? magicCount : undefined,
          good_message_count: showMessengers ? goodMsgCount : undefined,
          evil_message_count: showMessengers ? evilMsgCount : undefined,
        });

        // Check if game is over (count existing results + this one)
        const prevSuccesses = gameState.quests.filter(q =>
          q.quest.id !== currentQuest.quest.id && q.quest.result === 'success'
        ).length;
        const prevFails = gameState.quests.filter(q =>
          q.quest.id !== currentQuest.quest.id && q.quest.result === 'fail'
        ).length;
        const totalSuccesses = prevSuccesses + (result === 'success' ? 1 : 0);
        const totalFails = prevFails + (result === 'fail' ? 1 : 0);
        const isGameOver = totalSuccesses >= 3 || totalFails >= 3;

        // Advance to next quest only if game isn't over
        if (!isGameOver && game.current_quest < 5) {
          const nextQuest = gameState.quests.find(q => q.quest.quest_number === game.current_quest + 1);
          if (nextQuest) {
            const nextLeaderIdx = (players.findIndex(p => p.id === round.leader_player_id) + 1) % players.length;
            await api.createRound(game.id, nextQuest.quest.id, {
              leader_player_id: players[nextLeaderIdx].id,
              team_player_ids: [],
            });
          }
          await api.updateGame(game.id, { current_quest: game.current_quest + 1 });
        }
      } else {
        // Reject round
        await api.updateRound(game.id, round.id, { status: 'rejected' });
        // Check if 5th rejection
        if (currentQuest.rounds.length >= 5) {
          await api.updateQuest(game.id, currentQuest.quest.id, { result: 'fail' });

          // Check if game is over
          const prevFails = gameState.quests.filter(q =>
            q.quest.id !== currentQuest.quest.id && q.quest.result === 'fail'
          ).length;
          const isGameOver = (prevFails + 1) >= 3;

          if (!isGameOver && game.current_quest < 5) {
            const nextQuest = gameState.quests.find(q => q.quest.quest_number === game.current_quest + 1);
            if (nextQuest) {
              const nextLeaderIdx = (players.findIndex(p => p.id === round.leader_player_id) + 1) % players.length;
              await api.createRound(game.id, nextQuest.quest.id, {
                leader_player_id: players[nextLeaderIdx].id,
                team_player_ids: [],
              });
            }
            await api.updateGame(game.id, { current_quest: game.current_quest + 1 });
          }
        } else {
          // Next round, next leader
          const nextLeaderIdx = (players.findIndex(p => p.id === round.leader_player_id) + 1) % players.length;
          await api.createRound(game.id, currentQuest.quest.id, {
            leader_player_id: players[nextLeaderIdx].id,
            team_player_ids: [],
          });
        }
      }
      // Reset card counts
      successCount = 0; failCount = 0; magicCount = 0; goodMsgCount = 0; evilMsgCount = 0;
    } catch (e) { error = String(e); } finally { submitting = false; }
  }
</script>

<div class="card bg-base-100 shadow-sm">
  <div class="card-body">
    <div class="flex items-center justify-between">
      <h3 class="card-title">
        Quest {currentQuest.quest.quest_number}, Round {round.round_number}
        {#if qSize}
          <span class="badge badge-sm badge-outline">Team of {qSize}</span>
        {/if}
      </h3>
      <ToolsModal {gameState} />
    </div>

    {#if error}
      <div class="alert alert-error alert-sm mt-2">
        <span>{error}</span>
        <button class="btn btn-ghost btn-xs" onclick={() => error = ''}><X size={14} /></button>
      </div>
    {/if}

    <!-- King Selection -->
    <div class="mt-3">
      <p class="text-xs font-semibold text-base-content/60 mb-1 uppercase">King</p>
      <div class="flex flex-wrap gap-1">
        {#each players as player}
          <button
            class="btn btn-sm"
            class:btn-warning={round.leader_player_id === player.id}
            class:btn-outline={round.leader_player_id !== player.id}
            onclick={() => setKing(player.id)}
          >
            {playerNameById(gameState, player.id)}
          </button>
        {/each}
      </div>
    </div>

    <!-- Team Selection -->
    <div class="mt-3">
      <p class="text-xs font-semibold text-base-content/60 mb-1 uppercase">
        Team
        <span class="badge badge-xs" class:badge-success={teamIds.size === qSize} class:badge-error={teamIds.size !== qSize}>
          {teamIds.size}{qSize ? ` / ${qSize}` : ''}
        </span>
      </p>
      <div class="flex flex-wrap gap-1">
        {#each players as player}
          <button
            class="btn btn-sm"
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
    <div class="mt-3">
      <p class="text-xs font-semibold text-base-content/60 mb-1 uppercase">
        Votes
        <span class="badge badge-xs" class:badge-success={majorityApproved} class:badge-error={!majorityApproved}>
          {approveCount}<Check size={12} class="inline" /> {rejectCount}<X size={12} class="inline" /> → {majorityApproved ? 'Approved' : 'Rejected'}
        </span>
      </p>
      <div class="flex flex-wrap gap-1">
        {#each players as player}
          {@const vote = voteMap.get(player.id) ?? 'reject'}
          <button
            class="btn btn-sm"
            class:btn-success={vote === 'approve'}
            class:btn-error={vote === 'reject'}
            onclick={() => toggleVote(player.id)}
          >
            {playerNameById(gameState, player.id)}
            {#if vote === 'approve'}<Check size={14} class="inline" />{:else}<X size={14} class="inline" />{/if}
          </button>
        {/each}
      </div>
    </div>

    <!-- Results (only if majority approved) -->
    {#if majorityApproved}
      <div class="mt-3">
        <p class="text-xs font-semibold text-base-content/60 mb-1 uppercase">
          Cards Played
          {#if qSize}
            {@const totalCards = successCount + failCount + magicCount + goodMsgCount + evilMsgCount}
            <span class="badge badge-xs" class:badge-success={totalCards === qSize} class:badge-warning={totalCards !== qSize}>
              {totalCards} / {qSize}
            </span>
          {/if}
        </p>
        <div class="grid grid-cols-2 gap-2 max-w-sm">
          <div class="flex items-center justify-between bg-base-200 rounded-lg px-3 py-1">
            <span class="text-sm">Success</span>
            <div class="flex items-center gap-1">
              <button class="btn btn-xs btn-outline" onclick={() => successCount = Math.max(0, successCount - 1)}>−</button>
              <span class="badge badge-sm w-8 justify-center">{successCount}</span>
              <button class="btn btn-xs btn-outline" onclick={() => successCount++}>+</button>
            </div>
          </div>
          <div class="flex items-center justify-between bg-base-200 rounded-lg px-3 py-1">
            <span class="text-sm">Fail</span>
            <div class="flex items-center gap-1">
              <button class="btn btn-xs btn-outline" onclick={() => failCount = Math.max(0, failCount - 1)}>−</button>
              <span class="badge badge-sm w-8 justify-center">{failCount}</span>
              <button class="btn btn-xs btn-outline" onclick={() => failCount++}>+</button>
            </div>
          </div>
          {#if showMessengers}
            <div class="flex items-center justify-between bg-base-200 rounded-lg px-3 py-1">
              <span class="text-sm">Good Message</span>
              <div class="flex items-center gap-1">
                <button class="btn btn-xs btn-outline" onclick={() => goodMsgCount = Math.max(0, goodMsgCount - 1)}>−</button>
                <span class="badge badge-sm w-8 justify-center">{goodMsgCount}</span>
                <button class="btn btn-xs btn-outline" onclick={() => goodMsgCount++}>+</button>
              </div>
            </div>
            <div class="flex items-center justify-between bg-base-200 rounded-lg px-3 py-1">
              <span class="text-sm">Evil Message</span>
              <div class="flex items-center gap-1">
                <button class="btn btn-xs btn-outline" onclick={() => evilMsgCount = Math.max(0, evilMsgCount - 1)}>−</button>
                <span class="badge badge-sm w-8 justify-center">{evilMsgCount}</span>
                <button class="btn btn-xs btn-outline" onclick={() => evilMsgCount++}>+</button>
              </div>
            </div>
          {/if}
          {#if showSorcerers}
            <div class="flex items-center justify-between bg-base-200 rounded-lg px-3 py-1">
              <span class="text-sm">Magic</span>
              <div class="flex items-center gap-1">
                <button class="btn btn-xs btn-outline" onclick={() => magicCount = Math.max(0, magicCount - 1)}>−</button>
                <span class="badge badge-sm w-8 justify-center">{magicCount}</span>
                <button class="btn btn-xs btn-outline" onclick={() => magicCount++}>+</button>
              </div>
            </div>
          {/if}
        </div>

        <!-- Result Preview -->
        {#if previewResult}
          <div class="mt-2">
            <span class="badge badge-lg" class:badge-success={previewResult === 'success'} class:badge-error={previewResult === 'fail'}>
              Quest {previewResult === 'success' ? 'PASSES' : 'FAILS'}
              {#if previewResult === 'success'}<Check size={14} class="inline" />{:else}<X size={14} class="inline" />{/if}
            </span>
          </div>
        {/if}
      </div>
    {/if}

    <!-- Submit -->
    <div class="mt-4 flex justify-end">
      <button class="btn btn-primary" onclick={submit} disabled={submitting}>
        {#if majorityApproved}
          <Check size={16} class="inline" /> Complete Quest
        {:else}
          <X size={16} class="inline" /> Next Round
        {/if}
      </button>
    </div>
  </div>
</div>
