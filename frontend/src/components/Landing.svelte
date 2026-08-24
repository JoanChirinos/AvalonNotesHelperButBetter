<script lang="ts">
  import { api } from '../api';
  import { Swords, ArrowRight } from 'lucide-svelte';

  interface Props {
    onSelect: (namespace: string) => void;
  }

  let { onSelect }: Props = $props();

  let value = $state('');
  let namespaces = $state<string[]>([]);

  function submit() {
    const ns = value.trim();
    if (ns) onSelect(ns);
  }

  $effect(() => {
    api.listNamespaces().then((ns) => { namespaces = ns; }).catch(() => {});
  });
</script>

<div class="flex flex-col items-center justify-center py-16 gap-6">
  <div class="text-center">
    <div class="flex justify-center mb-2"><Swords size={40} /></div>
    <h1 class="text-3xl font-bold">Avalon Notes Helper</h1>
    <p class="text-base-content/60 mt-2">Pick a namespace to see its games, or type a new one to start your own group.</p>
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
    <p class="text-xs text-base-content/50 mt-2">Case-sensitive. Anyone with the name can view and edit its games.</p>
  </div>

  {#if namespaces.length > 0}
    <div class="w-full max-w-sm">
      <div class="text-sm text-base-content/60 mb-2">Existing namespaces</div>
      <div class="flex flex-wrap gap-2">
        {#each namespaces as ns}
          <button class="btn btn-sm btn-outline" onclick={() => onSelect(ns)}>{ns}</button>
        {/each}
      </div>
    </div>
  {/if}
</div>
