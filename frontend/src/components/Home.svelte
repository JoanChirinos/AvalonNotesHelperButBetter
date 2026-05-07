<script lang="ts">
  import { api } from '../api';
  import type { GameSummary, Role } from '../types';
  import { teamForRole, ROLE_DISPLAY_NAMES } from '../constants';

  interface Props {
    onNavigate: (path: string) => void;
  }

  let { onNavigate }: Props = $props();

  let games = $state<GameSummary[]>([]);
  let error = $state('');
  let loading = $state(true);

  // Pagination for finished games
  const PAGE_SIZE = 10;
  let currentPage = $state(1);

  // Parse page from hash
  $effect(() => {
    const match = window.location.hash.match(/page=(\d+)/);
    if (match) currentPage = parseInt(match[1]);
  });

  let activeGames = $derived(games.filter(g => !g.game.finished_at));
  let finishedGames = $derived(games.filter(g => g.game.finished_at));
  let totalPages = $derived(Math.max(1, Math.ceil(finishedGames.length / PAGE_SIZE)));
  let pagedFinished = $derived(
    finishedGames.slice((currentPage - 1) * PAGE_SIZE, currentPage * PAGE_SIZE)
  );

  async function loadGames() {
    try {
      loading = true;
      games = await api.listGames();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function createGame() {
    try {
      const state = await api.createGame({});
      onNavigate(`#/game/${state.game.id}`);
    } catch (e) {
      error = String(e);
    }
  }

  function goToPage(page: number) {
    currentPage = page;
    window.location.hash = `page=${page}`;
  }

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleString([], { dateStyle: 'medium', timeStyle: 'short' });
  }

  $effect(() => {
    loadGames();

    // Re-fetch when tab becomes visible (covers other browser tabs)
    const onVisible = () => {
      if (document.visibilityState === 'visible') loadGames();
    };
    document.addEventListener('visibilitychange', onVisible);
    return () => document.removeEventListener('visibilitychange', onVisible);
  });
</script>

<div class="flex items-center justify-between mb-6">
  <h2 class="text-2xl font-bold">Games</h2>
  <button class="btn btn-primary" onclick={createGame}>+ New Game</button>
</div>

{#if error}
  <div class="alert alert-error mb-4">{error}</div>
{/if}

{#if loading}
  <div class="flex justify-center p-8">
    <span class="loading loading-spinner loading-lg"></span>
  </div>
{:else}
  <!-- Active Games -->
  {#if activeGames.length > 0}
    <h3 class="text-lg font-semibold mb-3 text-base-content/70">Active</h3>
    <div class="grid gap-3 mb-8">
      {#each activeGames as summary}
        <button
          class="card bg-base-100 shadow-sm cursor-pointer hover:shadow-md transition-shadow text-left w-full"
          onclick={() => onNavigate(`#/game/${summary.game.id}`)}
        >
          <div class="card-body p-4">
            <div class="flex items-center justify-between">
              <span class="text-sm text-base-content/60">{formatDate(summary.game.created_at)}</span>
              {#if summary.player_names.length > 0}
                <span class="badge badge-sm badge-outline">{summary.player_names.length} players</span>
              {/if}
            </div>
            {#if summary.player_names.length > 0}
              <div class="text-sm mt-1">{summary.player_names.join(', ')}</div>
            {/if}
            <div class="text-sm text-base-content/50 mt-1">
              {summary.has_started ? `In Progress — Quest ${summary.game.current_quest}` : 'Setting Up'}
            </div>
          </div>
        </button>
      {/each}
    </div>
  {/if}

  <!-- Finished Games -->
  {#if finishedGames.length > 0}
    <h3 class="text-lg font-semibold mb-3 text-base-content/70">Completed</h3>
    <div class="grid gap-3 mb-4">
      {#each pagedFinished as summary}
        <button
          class="card bg-base-100 shadow-sm cursor-pointer hover:shadow-md transition-shadow text-left w-full"
          onclick={() => onNavigate(`#/game/${summary.game.id}`)}
        >
          <div class="card-body p-4">
            <div class="flex items-center justify-between">
              <span class="text-sm text-base-content/60">{formatDate(summary.game.finished_at!)}</span>
              <span class="badge badge-sm badge-outline">{summary.player_names.length} players</span>
            </div>
            {#if summary.player_names.length > 0}
              {@const paired = summary.player_names.map((name, i) => ({ name, role: summary.player_roles[i] })).sort((a, b) => {
                const order: Record<string, number> = { good: 0, evil: 1, z: 2 };
                const teamA = a.role ? teamForRole(a.role) : 'z';
                const teamB = b.role ? teamForRole(b.role) : 'z';
                if (teamA !== teamB) return order[teamA] - order[teamB];
                return a.name.localeCompare(b.name);
              })}
              <div class="flex flex-wrap gap-1 mt-1">
                {#each paired as { name, role }}
                  <span
                    class="rounded-full px-2 py-0.5 text-xs"
                    class:bg-success={role && teamForRole(role) === 'good'}
                    class:text-success-content={role && teamForRole(role) === 'good'}
                    class:bg-error={role && teamForRole(role) === 'evil'}
                    class:text-error-content={role && teamForRole(role) === 'evil'}
                    class:bg-base-200={!role}
                  >{name}</span>
                {/each}
              </div>
            {/if}
            {#if summary.result}
              <div class="mt-1">
                <span class="badge badge-sm" class:badge-success={summary.result === 'good'} class:badge-error={summary.result === 'evil'}>
                  {summary.result === 'good' ? 'Good Wins' : 'Evil Wins'}
                </span>
              </div>
            {/if}
          </div>
        </button>
      {/each}
    </div>

    <!-- Pagination -->
    {#if totalPages > 1}
      <div class="flex justify-center gap-2">
        {#each Array.from({ length: totalPages }, (_, i) => i + 1) as page}
          <button
            class="btn btn-sm"
            class:btn-active={page === currentPage}
            onclick={() => goToPage(page)}
          >
            {page}
          </button>
        {/each}
      </div>
    {/if}
  {/if}

  {#if games.length === 0}
    <div class="text-center py-12 text-base-content/50">
      <p class="text-lg">No games yet</p>
      <p class="text-sm mt-1">Create one to get started!</p>
    </div>
  {/if}
{/if}
