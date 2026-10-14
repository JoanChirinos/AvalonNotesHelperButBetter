<script lang="ts">
  import { api } from '../api';
  import type { FullGameState, Vote } from '../types';
  import { questSize } from '../constants';
  import { playerNameById } from '../derived';
  import { Dices, Users, ThumbsUp, X, Wrench, ArrowLeft, ArrowRight, Columns3, ToggleLeft, ToggleRight, Trophy, Repeat } from 'lucide-svelte';

  interface Props {
    gameState: FullGameState;
  }

  let { gameState }: Props = $props();

  let open = $state(false);
  let teamSizeInput = $state('');
  let alternating = $state(false);
  let error = $state('');

  let game = $derived(gameState.game);
  let players = $derived(gameState.players);
  let currentQuest = $derived(gameState.quests.find(q => q.quest.quest_number === game.current_quest)!);
  let currentRound = $derived(currentQuest.rounds[currentQuest.rounds.length - 1]);
  let round = $derived(currentRound.round);

  let defaultTeamSize = $derived(questSize(players.length, currentQuest.quest.quest_number) ?? 2);
  let teamSize = $derived(parseInt(teamSizeInput) || defaultTeamSize);

  function kingIndex(): number {
    return players.findIndex(p => p.id === round.leader_player_id);
  }

  // Pick players going in a direction from king, optionally skipping every other
  function pickInDirection(direction: -1 | 1, count: number): string[] {
    const ki = kingIndex();
    const step = alternating ? 2 : 1;
    const result: string[] = [];
    const seen = new Set<number>();
    let i = 0;
    while (result.length < count && seen.size < players.length - 1) {
      i++;
      const offset = i * step * direction;
      const idx = ((ki + offset) % players.length + players.length) % players.length;
      if (idx === ki || seen.has(idx)) continue;
      seen.add(idx);
      result.push(players[idx].id);
    }
    return result;
  }

  async function setTeam(ids: string[]) {
    try {
      await api.updateRound(game.id, round.id, { team_player_ids: ids });
      open = false;
    } catch (e) { error = String(e); }
  }

  async function windowBackward() {
    const king = players[kingIndex()].id;
    const others = pickInDirection(-1, teamSize - 1);
    await setTeam([king, ...others]);
  }

  async function windowForward() {
    const king = players[kingIndex()].id;
    const others = pickInDirection(1, teamSize - 1);
    await setTeam([king, ...others]);
  }

  async function windowMiddle() {
    const king = players[kingIndex()].id;
    const needed = teamSize - 1;
    let bc = Math.floor(needed / 2);
    let fc = needed - bc;

    // Odd splits must lean one direction — randomize which
    if (needed % 2 !== 0 && Math.random() < 0.5) {
      [bc, fc] = [fc, bc];
    }

    const back = pickInDirection(-1, bc);
    const forward = pickInDirection(1, fc);
    await setTeam([king, ...back, ...forward]);
  }

  async function randomKing() {
    try {
      const randomPlayer = players[Math.floor(Math.random() * players.length)];
      await api.updateRound(game.id, round.id, { leader_player_id: randomPlayer.id });
      open = false;
    } catch (e) { error = String(e); }
  }

  async function randomTeam() {
    try {
      const size = teamSize;
      const kingId = round.leader_player_id;
      const others = players.filter(p => p.id !== kingId);
      for (let i = others.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [others[i], others[j]] = [others[j], others[i]];
      }
      const team = [kingId, ...others.slice(0, size - 1).map(p => p.id)];
      await api.updateRound(game.id, round.id, { team_player_ids: team });
      open = false;
    } catch (e) { error = String(e); }
  }

  async function recordVotes(approve: (playerId: string) => boolean) {
    try {
      const votes = players.map(p => ({
        player_id: p.id,
        vote: (approve(p.id) ? 'approve' : 'reject') as Vote,
      }));
      await api.recordVotes(game.id, round.id, { votes });
      open = false;
    } catch (e) { error = String(e); }
  }

  function ezApproveTeam() {
    const teamIds = new Set(currentRound.team.map(t => t.player_id));
    return recordVotes(id => teamIds.has(id));
  }

  const ezApproveEveryone = () => recordVotes(() => true);

  let lastPassedTeam = $derived.by(() => {
    const passed = gameState.quests
      .filter(q => q.quest.result === 'success')
      .sort((a, b) => b.quest.quest_number - a.quest.quest_number)[0];
    const approved = passed?.rounds.find(r => r.round.status === 'approved');
    return approved ? approved.team.map(t => t.player_id) : [];
  });

  let lastProposedTeam = $derived.by(() => {
    const earlier = gameState.quests
      .slice()
      .sort((a, b) => a.quest.quest_number - b.quest.quest_number)
      .flatMap(q => q.rounds)
      .filter(r => r.round.id !== round.id && r.team.length > 0);
    return earlier.at(-1)?.team.map(t => t.player_id) ?? [];
  });
</script>

<button class="btn btn-sm btn-ghost" onclick={() => open = true}>
  <Wrench size={16} />
</button>

{#if open}
  <div class="fixed inset-0 bg-black/50 z-40" onclick={() => open = false}></div>

  <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
    <div class="card bg-base-100 shadow-xl w-full max-w-sm">
      <div class="card-body p-4">
        <div class="flex items-center justify-between mb-2">
          <h3 class="card-title text-base">Tools</h3>
          <button class="btn btn-ghost btn-sm" onclick={() => open = false}><X size={16} /></button>
        </div>

        {#if error}
          <div class="alert alert-error alert-sm mb-2">
            <span class="text-xs">{error}</span>
          </div>
        {/if}

        <!-- Team size input -->
        <div class="flex items-center gap-2 mb-2">
          <span class="text-sm">Team size:</span>
          <input
            type="number"
            class="input input-bordered input-sm w-16 text-center"
            placeholder={String(defaultTeamSize)}
            bind:value={teamSizeInput}
            min="1"
            max={players.length}
          />
        </div>

        <div class="space-y-2">
          <!-- Random -->
          <button class="btn btn-sm btn-outline w-full justify-start gap-2" onclick={randomKing}>
            <Dices size={16} /> Random King
          </button>
          <button class="btn btn-sm btn-outline w-full justify-start gap-2" onclick={randomTeam}>
            <Users size={16} /> Random Team
          </button>

          <!-- Windows -->
          <div class="divider text-xs my-1">Windows</div>

          <label class="flex items-center gap-2 cursor-pointer mb-1">
            {#if alternating}
              <ToggleRight size={20} class="text-primary" />
            {:else}
              <ToggleLeft size={20} class="text-base-content/40" />
            {/if}
            <input type="checkbox" class="hidden" bind:checked={alternating} />
            <span class="text-sm" class:text-primary={alternating}>Alternating</span>
          </label>

          <div class="flex gap-2">
            <button class="btn btn-sm btn-outline flex-1 gap-1" onclick={windowBackward}>
              <ArrowLeft size={14} /> Back
            </button>
            <button class="btn btn-sm btn-outline flex-1 gap-1" onclick={windowMiddle}>
              <Columns3 size={14} /> Middle
            </button>
            <button class="btn btn-sm btn-outline flex-1 gap-1" onclick={windowForward}>
              Fwd <ArrowRight size={14} />
            </button>
          </div>

          <!-- Ez-Teams -->
          {#if lastPassedTeam.length || lastProposedTeam.length}
            <div class="divider text-xs my-1">Ez-Teams</div>
            {#if lastPassedTeam.length}
              <button class="btn btn-sm btn-outline w-full justify-start gap-2" onclick={() => setTeam(lastPassedTeam)}>
                <Trophy size={16} /> Last Passed
              </button>
            {/if}
            {#if lastProposedTeam.length}
              <button class="btn btn-sm btn-outline w-full justify-start gap-2" onclick={() => setTeam(lastProposedTeam)}>
                <Repeat size={16} /> Run It Back
              </button>
            {/if}
          {/if}

          <!-- Ez-Approve -->
          <div class="divider text-xs my-1">Ez-Approve</div>
          <div class="flex gap-2">
            <button class="btn btn-sm btn-outline btn-success flex-1 justify-start gap-2" onclick={ezApproveTeam}>
              <ThumbsUp size={16} /> Team
            </button>
            <button class="btn btn-sm btn-outline btn-success flex-1 justify-start gap-2" onclick={ezApproveEveryone}>
              <ThumbsUp size={16} /> Everyone
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
