<script lang="ts">
  import { api } from '../api';
  import type { FullGameState } from '../types';
  import { buildFacts, GLOBAL_BLOCKS, PLAYER_BLOCKS } from '../stats';
  import StatBlockCard from './StatBlockCard.svelte';
  import { ArrowLeft } from 'lucide-svelte';

  interface Props {
    namespace: string;
    onNavigate: (path: string) => void;
  }

  let { namespace, onNavigate }: Props = $props();

  let games = $state<FullGameState[] | null>(null);
  let error = $state('');
  let selected = $state<'global' | string>('global'); // 'global' or a knownPlayerId

  let facts = $derived(games ? buildFacts(games) : null);

  let blocks = $derived.by(() => {
    if (!facts) return [];
    if (selected === 'global') {
      return GLOBAL_BLOCKS.map((b) => ({ id: b.id, title: b.title, result: b.compute(facts!) }));
    }
    return PLAYER_BLOCKS.map((b) => ({ id: b.id, title: b.title, result: b.compute(facts!, selected) }));
  });

  let selectedName = $derived(
    selected === 'global'
      ? 'Global'
      : (facts?.roster.find((r) => r.knownPlayerId === selected)?.name ?? 'Player')
  );

  $effect(() => {
    games = null;
    error = '';
    api.listFullGames(namespace).then((g) => { games = g; }).catch((e) => { error = String(e); });
  });
</script>

<div class="space-y-4">
  <div class="flex items-center gap-3">
    <button class="btn btn-ghost btn-sm" onclick={() => onNavigate('#/')}><ArrowLeft size={16} /> Back</button>
    <h2 class="text-2xl font-bold">Stats <span class="text-lg font-normal text-base-content/50">· {namespace}</span></h2>
  </div>

  {#if error}
    <div class="alert alert-error">{error}</div>
  {:else if !games}
    <div class="flex justify-center p-8"><span class="loading loading-spinner loading-lg"></span></div>
  {:else if games.length === 0}
    <div class="text-center py-12 text-base-content/50">
      <p class="text-lg">No finished games yet</p>
      <p class="text-sm mt-1">Finish some games to see stats here.</p>
    </div>
  {:else}
    <div class="flex items-start gap-4">
      <aside class="sticky top-20 w-44 shrink-0">
        <ul class="menu rounded-box bg-base-100 p-2 shadow-sm">
          <li>
            <button class:menu-active={selected === 'global'} onclick={() => (selected = 'global')}>Global</button>
          </li>
          {#if facts}
            <li class="menu-title">Players</li>
            {#each facts.roster as r (r.knownPlayerId)}
              <li>
                <button class:menu-active={selected === r.knownPlayerId} onclick={() => (selected = r.knownPlayerId)}>{r.name}</button>
              </li>
            {/each}
          {/if}
        </ul>
      </aside>

      <div class="min-w-0 max-w-4xl flex-1 space-y-4">
        <h3 class="text-lg font-semibold">{selectedName}</h3>
        {#each blocks as b (b.id)}
          <StatBlockCard title={b.title} result={b.result} />
        {/each}
      </div>
    </div>
  {/if}
</div>
