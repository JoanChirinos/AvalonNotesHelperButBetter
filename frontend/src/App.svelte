<script lang="ts">
  import './app.css';
  import Home from './components/Home.svelte';
  import GamePage from './components/GamePage.svelte';
  import Landing from './components/Landing.svelte';
  import StatsPage from './components/StatsPage.svelte';
  import RoundTimer from './components/RoundTimer.svelte';
  import { api } from './api';
  import { Moon, Sun, Swords, RefreshCw, RotateCcw, Settings, X } from 'lucide-svelte';

  let isDark = $state(
    localStorage.getItem('theme')
      ? localStorage.getItem('theme') === 'dark'
      : window.matchMedia('(prefers-color-scheme: dark)').matches
  );

  // Selected namespace (null = show landing picker). Namespaces are case-sensitive.
  let namespace = $state<string | null>(localStorage.getItem('namespace'));

  // Simple hash router
  let route = $state(window.location.hash || '#/');

  function navigate(path: string) {
    window.location.hash = path;
    route = path;
  }

  window.addEventListener('hashchange', () => {
    route = window.location.hash || '#/';
  });

  let gameId = $derived(
    route.startsWith('#/game/') ? route.slice('#/game/'.length) : null
  );

  function selectNamespace(ns: string) {
    namespace = ns;
    localStorage.setItem('namespace', ns);
    const recents = JSON.parse(localStorage.getItem('namespace_recents') || '[]') as string[];
    const next = [ns, ...recents.filter((r) => r !== ns)].slice(0, 8);
    localStorage.setItem('namespace_recents', JSON.stringify(next));
    navigate('#/');
  }

  function switchNamespace() {
    namespace = null;
    localStorage.removeItem('namespace');
    navigate('#/');
  }

  let showSettings = $state(false);

  // "Again!": create a fresh game duplicating the current one's players, roles, and
  // modules (same namespace). Does NOT start it — lands on the setup screen.
  let creatingAgain = $state(false);
  let againError = $state('');
  async function playAgain() {
    if (!gameId || creatingAgain) return;
    creatingAgain = true;
    againError = '';
    try {
      const g = await api.getGame(gameId);
      const nameById = new Map(g.known_players.map((kp) => [kp.id, kp.name]));
      const player_names = g.players
        .map((p) => nameById.get(p.known_player_id))
        .filter((n): n is string => !!n);
      const roles = g.roles.map((r) => r.role);
      const modules = g.modules.map((m) => m.module);
      const created = await api.createGame({ player_names, roles, modules, namespace: g.game.namespace });
      showSettings = false;
      navigate(`#/game/${created.game.id}`);
    } catch (e) {
      againError = `Couldn't create the game: ${String(e)}`;
    } finally {
      creatingAgain = false;
    }
  }

  $effect(() => {
    document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
    localStorage.setItem('theme', isDark ? 'dark' : 'light');
  });
</script>

<div class="min-h-screen bg-base-200">
  <div class="navbar bg-base-100 shadow-sm sticky top-0 z-40">
    <div class="flex-1">
      <button class="btn btn-ghost text-xl" onclick={() => navigate('#/')}><Swords size={20} /> Avalon Notes Helper</button>
    </div>
    {#if gameId}
      {#key gameId}
        <div class="flex-1 flex justify-center">
          <RoundTimer />
        </div>
      {/key}
    {:else if namespace}
      <div class="flex-none pr-2">
        <button class="btn btn-ghost btn-sm gap-1" onclick={switchNamespace} title="Switch namespace">
          <span class="badge badge-primary badge-sm">{namespace}</span>
          <RefreshCw size={14} />
        </button>
      </div>
    {/if}
    <div class="flex-none pr-2">
      <label class="swap swap-rotate">
        <input type="checkbox" bind:checked={isDark} />
        <span class="swap-on"><Moon size={18} /></span>
        <span class="swap-off"><Sun size={18} /></span>
      </label>
    </div>
    {#if gameId}
      <div class="flex-none pr-2">
        <button class="btn btn-ghost btn-sm btn-square" onclick={() => showSettings = true} title="Settings" aria-label="Settings">
          <Settings size={18} />
        </button>
      </div>
    {/if}
  </div>

  <div class="mx-auto w-full max-w-[110rem] p-4">
    {#if gameId}
      <GamePage {gameId} onNavigate={navigate} />
    {:else if route.startsWith('#/stats') && namespace}
      <StatsPage {namespace} onNavigate={navigate} />
    {:else if namespace}
      <Home {namespace} onNavigate={navigate} />
    {:else}
      <Landing onSelect={selectNamespace} />
    {/if}
  </div>

  {#if showSettings}
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4" role="presentation" onclick={() => showSettings = false}>
      <div class="card bg-base-100 shadow-xl w-full max-w-sm" role="dialog" onclick={(e) => e.stopPropagation()}>
        <div class="card-body gap-4">
          <div class="flex items-center justify-between">
            <h3 class="card-title">Settings</h3>
            <button class="btn btn-ghost btn-sm btn-square" onclick={() => showSettings = false} aria-label="Close"><X size={18} /></button>
          </div>
          {#if gameId}
            <button class="btn btn-primary gap-2 w-full" onclick={playAgain} disabled={creatingAgain}>
              <RotateCcw size={16} /> Again!
            </button>
            {#if againError}
              <p class="text-error text-xs">{againError}</p>
            {/if}
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>
