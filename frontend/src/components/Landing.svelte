<script lang="ts">
  import { Swords, ArrowRight } from 'lucide-svelte';

  interface Props {
    onSelect: (namespace: string) => void;
  }

  let { onSelect }: Props = $props();

  let value = $state('');
  const recents = JSON.parse(localStorage.getItem('namespace_recents') || '[]') as string[];

  function submit() {
    const ns = value.trim();
    if (ns) onSelect(ns);
  }
</script>

<div class="flex flex-col items-center justify-center py-16 gap-6">
  <div class="text-center">
    <div class="flex justify-center mb-2"><Swords size={40} /></div>
    <h1 class="text-3xl font-bold">Avalon Notes Helper</h1>
    <p class="text-base-content/60 mt-2">Enter a namespace to see its games. Share the name with your group to play together.</p>
  </div>

  <div class="w-full max-w-sm">
    <div class="join w-full">
      <input
        class="input input-bordered join-item w-full"
        placeholder="Namespace (e.g. SGW)"
        bind:value
        onkeydown={(e) => { if (e.key === 'Enter') submit(); }}
        autofocus
      />
      <button class="btn btn-primary join-item" onclick={submit} disabled={!value.trim()}>
        <ArrowRight size={18} />
      </button>
    </div>
    <p class="text-xs text-base-content/50 mt-2">Namespaces are case-sensitive. Anyone with the name can view and edit its games.</p>
  </div>

  {#if recents.length > 0}
    <div class="w-full max-w-sm">
      <div class="text-sm text-base-content/60 mb-2">Recent</div>
      <div class="flex flex-wrap gap-2">
        {#each recents as ns}
          <button class="btn btn-sm btn-outline" onclick={() => onSelect(ns)}>{ns}</button>
        {/each}
      </div>
    </div>
  {/if}
</div>
