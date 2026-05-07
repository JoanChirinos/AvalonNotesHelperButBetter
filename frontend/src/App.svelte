<script lang="ts">
  import './app.css';
  import Home from './components/Home.svelte';
  import GamePage from './components/GamePage.svelte';
  import RoundTimer from './components/RoundTimer.svelte';
  import { Moon, Sun, Swords } from 'lucide-svelte';

  let isDark = $state(
    localStorage.getItem('theme')
      ? localStorage.getItem('theme') === 'dark'
      : window.matchMedia('(prefers-color-scheme: dark)').matches
  );

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
    {:else}
      <Home onNavigate={navigate} />
    {/if}
  </div>
</div>
