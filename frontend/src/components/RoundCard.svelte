<script lang="ts">
  import type { FullGameState, RoundState } from '../types';
  import { playerNameById } from '../derived';
  import { Check, X } from 'lucide-svelte';
  import { Shield, ShieldOff, Mail, Sparkles } from 'lucide-svelte';

  interface Props {
    gameState: FullGameState;
    roundState: RoundState;
    questNumber: number;
    showResult?: boolean;
    onEdit?: () => void;
  }

  let { gameState, roundState, questNumber, showResult = false, onEdit }: Props = $props();

  let round = $derived(roundState.round);
  let players = $derived(gameState.players);
  let teamIds = $derived(new Set(roundState.team.map(t => t.player_id)));
  let voteMap = $derived(new Map(roundState.votes.map(v => [v.player_id, v.vote])));

  let teamPlayers = $derived(
    players.filter(p => teamIds.has(p.id))
  );
  let nonTeamPlayers = $derived(
    players.filter(p => !teamIds.has(p.id))
  );

  let isApproved = $derived(round.status === 'approved');
  let isRejected = $derived(round.status === 'rejected');

  // Find the quest to show card counts
  let quest = $derived(gameState.quests.find(q => q.quest.quest_number === questNumber)?.quest);
</script>

<div
  class="rounded-lg border p-2 text-sm"
  class:border-base-300={!isApproved && !isRejected}
  class:border-success={isApproved}
  class:border-error={isRejected}
  class:bg-base-100={!isRejected}
  class:bg-base-200={isRejected}
  class:cursor-pointer={!!onEdit}
  class:hover:ring-2={!!onEdit}
  class:hover:ring-info={!!onEdit}
  onclick={onEdit}
  role={onEdit ? 'button' : undefined}
>
  <div class="text-xs text-base-content/50 mb-1">
    R{round.round_number}
    {#if isRejected}
      <span class="text-error"> rejected</span>
    {/if}
  </div>

  <!-- Team + Non-team: side by side on large, stacked on small -->
  <div class="flex flex-col sm:flex-row flex-wrap items-start gap-2">
    <!-- Team card -->
    {#if teamPlayers.length > 0}
      <div class="border-2 border-primary rounded px-2 py-1">
        <div class="flex flex-wrap gap-1 sm:gap-2">
          {#each teamPlayers as player}
            {@const vote = voteMap.get(player.id) ?? 'reject'}
            {@const isKing = player.id === round.leader_player_id}
            <div class="flex flex-col items-center gap-0.5">
              {#if isKing}
                <span class="bg-warning text-warning-content rounded-full px-1.5 py-0.5 text-xs font-medium">
                  {playerNameById(gameState, player.id)}
                </span>
              {:else}
                <span class="bg-base-200 rounded-full px-1.5 py-0.5 text-xs">
                  {playerNameById(gameState, player.id)}
                </span>
              {/if}
              {#if vote === 'approve'}
                <Check size={12} class="text-success" />
              {:else}
                <X size={12} class="text-error" />
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Non-team players -->
    {#if nonTeamPlayers.length > 0}
      <div class="rounded px-2 py-1">
        <div class="flex flex-wrap gap-1 sm:gap-2">
          {#each nonTeamPlayers as player}
            {@const vote = voteMap.get(player.id) ?? 'reject'}
            {@const isKing = player.id === round.leader_player_id}
            <div class="flex flex-col items-center gap-0.5">
              {#if isKing}
                <span class="bg-warning text-warning-content rounded-full px-1.5 py-0.5 text-xs font-medium">
                  {playerNameById(gameState, player.id)}
                </span>
              {:else}
                <span class="bg-base-200 rounded-full px-1.5 py-0.5 text-xs text-base-content/60">
                  {playerNameById(gameState, player.id)}
                </span>
              {/if}
              {#if vote === 'approve'}
                <Check size={12} class="text-success" />
              {:else}
                <X size={12} class="text-error" />
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  <!-- Card counts for approved rounds -->
  {#if isApproved && quest}
    <div class="mt-1 flex flex-wrap gap-0.5 items-center justify-end">
      {#each Array(quest.success_count ?? 0) as _}
        <Shield size={14} class="text-success" />
      {/each}
      {#each Array(quest.fail_count ?? 0) as _}
        <ShieldOff size={14} class="text-error" />
      {/each}
      {#each Array(quest.good_message_count ?? 0) as _}
        <Mail size={14} class="text-success" />
      {/each}
      {#each Array(quest.evil_message_count ?? 0) as _}
        <Mail size={14} class="text-error" />
      {/each}
      {#each Array(quest.magic_count ?? 0) as _}
        <Sparkles size={14} class="text-purple-500" />
      {/each}
    </div>
  {/if}
</div>
