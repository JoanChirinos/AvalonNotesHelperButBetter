<script lang="ts">
  import { onDestroy } from 'svelte';
  import { api } from '../api';
  import type { FullGameState, ClaimedAffiliation } from '../types';
  import { playerNameById } from '../derived';
  import { X, Play, Pause, RotateCcw, Eye } from 'lucide-svelte';

  interface Props {
    gameState: FullGameState;
  }

  let { gameState }: Props = $props();

  let open = $state(false);
  let error = $state('');

  let players = $derived(gameState.players);
  let holders = $derived(gameState.lady_holders);
  let investigations = $derived(gameState.lady_investigations);
  let currentQuest = $derived(gameState.game.current_quest);
  let isEnabled = $derived(gameState.modules.some(m => m.module === 'lady_of_the_lake'));

  // Current holder is the last in the chain
  let currentHolder = $derived(
    holders.length > 0 ? holders[holders.length - 1] : null
  );

  // Players who have already held the Lady (can't be investigated)
  let holderIds = $derived(new Set(holders.map(h => h.player_id)));

  // Available targets (not already held)
  let availableTargets = $derived(
    players.filter(p => !holderIds.has(p.id))
  );

  // Should we show the Lady for this quest? (quests 3, 4, 5)
  let ladyQuestNumbers = $derived(new Set([3, 4, 5]));
  let shouldShow = $derived(isEnabled && ladyQuestNumbers.has(currentQuest));

  // Has Lady already been used for the current quest?
  let currentQuestId = $derived(
    gameState.quests.find(q => q.quest.quest_number === currentQuest)?.quest.id
  );
  let alreadyUsedThisQuest = $derived(
    investigations.some(inv => inv.quest_id === currentQuestId)
  );

  // Auto-open when entering quest 3/4/5 and not yet used
  let lastAutoOpened = $state(0);
  $effect(() => {
    if (shouldShow && !alreadyUsedThisQuest && currentQuest !== lastAutoOpened) {
      open = true;
      lastAutoOpened = currentQuest;
    }
  });

  // Form state
  let investigatorId = $state('');
  let targetId = $state('');
  let claimed = $state<ClaimedAffiliation>('good');

  // Derive initial Lady holder: player to the left of the first king
  let initialLadyPlayerId = $derived.by(() => {
    if (holders.length > 0) return null;
    const firstQuest = gameState.quests[0];
    if (!firstQuest || firstQuest.rounds.length === 0) return null;
    const firstLeaderId = firstQuest.rounds[0].round.leader_player_id;
    const leaderIdx = players.findIndex(p => p.id === firstLeaderId);
    if (leaderIdx === -1) return null;
    const ladyIdx = (leaderIdx - 1 + players.length) % players.length;
    return players[ladyIdx]?.id ?? null;
  });

  // Auto-fill investigator from current holder (or initial derivation)
  $effect(() => {
    if (currentHolder) {
      investigatorId = currentHolder.player_id;
    } else if (initialLadyPlayerId) {
      investigatorId = initialLadyPlayerId;
    }
  });

  // Timer
  let timerTotal = $state(120);
  let timerRemaining = $state(120);
  let timerRunning = $state(false);
  let timerInterval: ReturnType<typeof setInterval> | null = null;

  onDestroy(() => { if (timerInterval) clearInterval(timerInterval); });

  function startTimer() {
    if (timerRemaining <= 0) return;
    timerRunning = true;
    timerInterval = setInterval(() => {
      timerRemaining--;
      if (timerRemaining <= 0) {
        timerRemaining = 0;
        stopTimer();
        try {
          const ctx = new AudioContext();
          for (let i = 0; i < 3; i++) {
            const osc = ctx.createOscillator();
            const gain = ctx.createGain();
            osc.connect(gain);
            gain.connect(ctx.destination);
            osc.frequency.value = 800;
            gain.gain.value = 0.3;
            osc.start(ctx.currentTime + i * 0.3);
            osc.stop(ctx.currentTime + i * 0.3 + 0.15);
          }
        } catch {}
      }
    }, 1000);
  }

  function stopTimer() {
    timerRunning = false;
    if (timerInterval) { clearInterval(timerInterval); timerInterval = null; }
  }

  function restartTimer() {
    stopTimer();
    timerRemaining = timerTotal;
  }

  let timerDisplay = $derived(
    `${Math.floor(timerRemaining / 60)}:${String(timerRemaining % 60).padStart(2, '0')}`
  );

  async function submit() {
    if (!targetId || !currentQuestId) return;
    try {
      await api.createLadyInvestigation(gameState.game.id, {
        quest_id: currentQuestId,
        investigator_player_id: investigatorId,
        target_player_id: targetId,
        claimed_affiliation: claimed,
      });
      stopTimer();
      open = false;
      targetId = '';
    } catch (e) {
      error = String(e);
    }
  }
</script>

<!-- Lady chain display + reopen button -->
{#if shouldShow && !open}
  <div class="flex items-center gap-1 flex-wrap">
    {#if holders.length > 0 || initialLadyPlayerId}
      {#if holders.length > 0}
        {#each holders as holder, i}
          <span class="text-xs font-medium">{playerNameById(gameState, holder.player_id)}</span>
          {#if i < investigations.length}
            {@const inv = investigations[i]}
            <Eye size={14} class={inv.claimed_affiliation === 'good' ? 'text-success' : 'text-error'} />
          {/if}
        {/each}
      {:else}
        <span class="text-xs font-medium">{playerNameById(gameState, initialLadyPlayerId!)}</span>
      {/if}
    {/if}
    <button class="btn btn-xs btn-ghost" onclick={() => open = true}>
      <Eye size={14} />
    </button>
  </div>
{/if}

{#if open}
  <div class="fixed inset-0 bg-black/50 z-40" onclick={() => open = false}></div>

  <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
    <div class="card bg-base-100 shadow-xl w-full max-w-md">
      <div class="card-body p-4">
        <div class="flex items-center justify-between mb-3">
          <h3 class="card-title text-base">Lady of the Lake</h3>
          <div class="flex items-center gap-2">
            <!-- Timer -->
            <span class="font-mono text-sm" class:text-error={timerRemaining <= 0}>{timerDisplay}</span>
            {#if timerRunning}
              <button class="btn btn-ghost btn-xs" onclick={stopTimer}><Pause size={12} /></button>
            {:else}
              <button class="btn btn-ghost btn-xs" onclick={startTimer}><Play size={12} /></button>
            {/if}
            <button class="btn btn-ghost btn-xs" onclick={restartTimer}><RotateCcw size={12} /></button>
            <button class="btn btn-ghost btn-sm" onclick={() => open = false}><X size={16} /></button>
          </div>
        </div>

        {#if error}
          <div class="alert alert-error alert-sm mb-2">
            <span class="text-xs">{error}</span>
          </div>
        {/if}

        {#if alreadyUsedThisQuest}
          <p class="text-sm text-base-content/60">Lady of the Lake already used this quest.</p>
        {:else}
          <div class="flex flex-wrap items-center gap-2 text-sm">
            <select class="select select-bordered select-sm" bind:value={investigatorId}>
              {#each players as p}
                <option value={p.id}>{playerNameById(gameState, p.id)}</option>
              {/each}
            </select>
            <span>checked</span>
            <select class="select select-bordered select-sm" bind:value={targetId}>
              <option value="" disabled>Select target</option>
              {#each availableTargets as p}
                <option value={p.id}>{playerNameById(gameState, p.id)}</option>
              {/each}
            </select>
            <span>and claimed</span>
            <select class="select select-bordered select-sm" bind:value={claimed}>
              <option value="good">Good</option>
              <option value="evil">Evil</option>
            </select>
          </div>

          <button
            class="btn btn-primary btn-sm mt-3"
            disabled={!targetId}
            onclick={submit}
          >Submit</button>
        {/if}

        <!-- Investigation history -->
        {#if investigations.length > 0}
          <div class="divider text-xs my-2">History</div>
          <div class="space-y-1">
            {#each investigations as inv}
              <div class="text-xs text-base-content/60">
                {playerNameById(gameState, inv.investigator_player_id)} checked
                {playerNameById(gameState, inv.target_player_id)} → claimed
                <span class:text-success={inv.claimed_affiliation === 'good'} class:text-error={inv.claimed_affiliation === 'evil'}>
                  {inv.claimed_affiliation}
                </span>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
