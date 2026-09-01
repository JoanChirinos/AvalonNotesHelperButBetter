<script lang="ts">
  import type { StatResult } from '../stats';
  import TimeSeriesChart from './TimeSeriesChart.svelte';

  interface Props {
    title: string;
    result: StatResult;
  }

  let { title, result }: Props = $props();
  let view = $derived(result.view);
  let hcMax = $derived(view.kind === 'heatcells' ? Math.max(1, ...view.cells.map((c) => c.value)) : 1);

  const toneClass: Record<string, string> = {
    good: 'bg-success',
    evil: 'bg-error',
    neutral: 'bg-primary',
  };

  // For bar splits: total for proportion.
  let barsTotal = $derived(
    view.kind === 'bars' ? view.segments.reduce((s, seg) => s + seg.value, 0) : 0
  );
  // For leaderboards: max value so bars are proportional.
  let lbMax = $derived(
    view.kind === 'leaderboard' ? Math.max(1, ...view.rows.map((r) => r.value)) : 1
  );
</script>

<div class="card bg-base-100 shadow-sm">
  <div class="card-body p-4 gap-3">
    <div class="flex items-baseline justify-between gap-2">
      <h3 class="text-sm font-semibold text-base-content/70 uppercase tracking-wide">{title}</h3>
      {#if result.note}
        <span class="text-xs text-base-content/50">{result.note}</span>
      {/if}
    </div>

    {#if view.kind === 'kpis'}
      <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
        {#each view.items as item}
          <div class="rounded-lg bg-base-200 px-3 py-2">
            <div class="text-2xl font-bold tabular-nums">{item.value}</div>
            <div class="text-xs text-base-content/60">{item.label}</div>
          </div>
        {/each}
      </div>

    {:else if view.kind === 'bars'}
      {#if barsTotal === 0}
        <p class="text-sm text-base-content/50">No decided games yet.</p>
      {:else}
        <div class="flex h-6 w-full gap-0.5 overflow-hidden rounded-lg">
          {#each view.segments as seg}
            {#if seg.value > 0}
              <div class="{toneClass[seg.tone]} h-full" style:width={`${(seg.value / barsTotal) * 100}%`}></div>
            {/if}
          {/each}
        </div>
        <div class="flex flex-wrap gap-x-4 gap-y-1 text-sm">
          {#each view.segments as seg}
            <span class="flex items-center gap-1.5">
              <span class="inline-block h-2.5 w-2.5 rounded-sm {toneClass[seg.tone]}"></span>
              <span>{seg.label}</span>
              <span class="text-base-content/60 tabular-nums">{seg.value} ({Math.round((seg.value / barsTotal) * 100)}%)</span>
            </span>
          {/each}
        </div>
      {/if}

    {:else if view.kind === 'leaderboard'}
      {#if view.rows.length === 0}
        <p class="text-sm text-base-content/50">Not enough data yet.</p>
      {:else}
        <div class="space-y-1.5">
          {#each view.rows as row, i}
            <div class="flex items-center gap-2 text-sm">
              <span class="w-4 text-right text-xs text-base-content/40 tabular-nums">{i + 1}</span>
              <span class="w-28 shrink-0 truncate font-medium">{row.label}</span>
              <div class="relative h-4 flex-1 rounded bg-base-200">
                <div class="absolute inset-y-0 left-0 rounded bg-primary" style:width={`${(row.value / lbMax) * 100}%`}></div>
              </div>
              <span class="w-24 shrink-0 text-right text-xs text-base-content/70 tabular-nums">{row.display}</span>
            </div>
          {/each}
        </div>
      {/if}

    {:else if view.kind === 'table'}
      {#if view.rows.length === 0}
        <p class="text-sm text-base-content/50">No data yet.</p>
      {:else}
        <div class="overflow-x-auto">
          <table class="table table-sm">
            <thead>
              <tr>
                {#each view.columns as col}
                  <th class={col.align === 'right' ? 'text-right' : 'text-left'}>{col.label}</th>
                {/each}
              </tr>
            </thead>
            <tbody>
              {#each view.rows as row}
                <tr>
                  {#each view.columns as col}
                    <td class="{col.align === 'right' ? 'text-right tabular-nums' : 'text-left'}">{row[col.key]}</td>
                  {/each}
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}

    {:else if view.kind === 'heatcells'}
      <div class="flex gap-1">
        {#each view.cells as c}
          <div
            class="flex-1 rounded-md px-1 py-2 text-center"
            style:background-color={`color-mix(in oklab, var(--color-success) ${Math.round((c.value / hcMax) * 100)}%, var(--color-neutral))`}
          >
            <div class="text-xs font-semibold text-neutral-content">{c.label}</div>
            <div class="text-sm font-bold tabular-nums text-neutral-content">{c.value}</div>
          </div>
        {/each}
      </div>

    {:else if view.kind === 'timeseries'}
      <TimeSeriesChart dates={view.dates} />
    {/if}
  </div>
</div>
