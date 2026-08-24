<script lang="ts">
  import './app.css';
  import Home from './components/Home.svelte';
  import GamePage from './components/GamePage.svelte';
  import Landing from './components/Landing.svelte';
  import RoundTimer from './components/RoundTimer.svelte';
  import { Moon, Sun, Swords, RefreshCw } from 'lucide-svelte';

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

  $effect(() => {
    document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light');
    localStorage.setItem('theme', isDark ? 'dark' : 'light');
  });
</script>

<div class="min-h-screen bg-base-200">
  <div class="navbar bg-base-100 shadow-sm">
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
  </div>

  <div class="container mx-auto p-4">
    {#if gameId}
      <GamePage {gameId} onNavigate={navigate} />
    {:else if namespace}
      <Home {namespace} onNavigate={navigate} />
    {:else}
      <Landing onSelect={selectNamespace} />
    {/if}
  </div>
</div>
