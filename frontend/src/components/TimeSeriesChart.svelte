<script lang="ts">
  import { bucketDates, type Bucket } from '../stats';

  interface Props {
    dates: string[];
  }
  let { dates }: Props = $props();

  let bucket = $state<Bucket>('week');
  let windowKey = $state<'all' | '3m' | '6m' | '1y'>('all');

  const WINDOW_DAYS: Record<string, number> = { '3m': 90, '6m': 182, '1y': 365 };

  let range = $derived.by(() => {
    if (windowKey === 'all') return undefined;
    return { start: new Date(Date.now() - WINDOW_DAYS[windowKey] * 86400_000), end: new Date() };
  });

  let buckets = $derived(bucketDates(dates, bucket, range));
  let max = $derived(Math.max(1, ...buckets.map((b) => b.count)));

  const W = 640, H = 220, padL = 30, padR = 14, padT = 14, padB = 28;
  const innerW = W - padL - padR;
  const innerH = H - padT - padB;

  let points = $derived(
    buckets.map((b, i) => ({
      x: buckets.length > 1 ? padL + (i * innerW) / (buckets.length - 1) : padL + innerW / 2,
      y: H - padB - (b.count / max) * innerH,
      label: b.label,
      count: b.count,
    }))
  );
  let linePath = $derived(points.map((p) => `${p.x},${p.y}`).join(' '));

  let yTicks = $derived.by(() => {
    const vals = [...new Set([0, Math.round(max / 2), max])];
    return vals.map((v) => ({ v, y: H - padB - (v / max) * innerH }));
  });

  let labelIdxs = $derived.by(() => {
    const n = points.length;
    if (n <= 5) return points.map((_, i) => i);
    const step = (n - 1) / 4;
    return [0, 1, 2, 3, 4].map((k) => Math.round(k * step));
  });

  const BUCKETS: { key: Bucket; label: string }[] = [
    { key: 'day', label: 'Daily' }, { key: 'week', label: 'Weekly' }, { key: 'month', label: 'Monthly' },
  ];
  const WINDOWS: { key: 'all' | '3m' | '6m' | '1y'; label: string }[] = [
    { key: 'all', label: 'All' }, { key: '1y', label: '1y' }, { key: '6m', label: '6m' }, { key: '3m', label: '3m' },
  ];
</script>

<div class="space-y-2">
  <div class="flex flex-wrap items-center justify-between gap-2">
    <div class="join">
      {#each BUCKETS as b}
        <button class="btn btn-xs join-item" class:btn-active={bucket === b.key} onclick={() => (bucket = b.key)}>{b.label}</button>
      {/each}
    </div>
    <div class="join">
      {#each WINDOWS as w}
        <button class="btn btn-xs join-item" class:btn-active={windowKey === w.key} onclick={() => (windowKey = w.key)}>{w.label}</button>
      {/each}
    </div>
  </div>

  {#if points.length === 0}
    <p class="text-sm text-base-content/50">No games in this window.</p>
  {:else}
    <svg viewBox={`0 0 ${W} ${H}`} class="h-auto w-full" role="img" aria-label="Games over time">
      <rect x={padL} y={padT} width={innerW} height={innerH} rx="4" style="fill: var(--color-base-200)" />
      {#each yTicks as t}
        <line x1={padL} y1={t.y} x2={W - padR} y2={t.y} style="stroke: var(--color-base-content); opacity: 0.14" stroke-width="1" />
        <text x={padL - 6} y={t.y + 3} text-anchor="end" style="fill: var(--color-base-content); opacity: 0.55" class="text-[9px]">{t.v}</text>
      {/each}
      {#if points.length > 1}
        <polyline points={linePath} fill="none" style="stroke: var(--color-primary)" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round" />
      {/if}
      {#if points.length <= 60}
        {#each points as p}
          <circle cx={p.x} cy={p.y} r="3.5" style="fill: var(--color-primary)">
            <title>{p.label}: {p.count} game{p.count === 1 ? '' : 's'}</title>
          </circle>
        {/each}
      {/if}
      {#each labelIdxs as i}
        <text x={points[i].x} y={H - padB + 15} text-anchor="middle" style="fill: var(--color-base-content); opacity: 0.55" class="text-[9px]">{points[i].label}</text>
      {/each}
    </svg>
  {/if}
</div>
