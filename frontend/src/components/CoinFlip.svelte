<script lang="ts">
  import { onDestroy } from 'svelte';
  import type { FullGameState } from '../types';
  import { playerNameById } from '../derived';
  import { X } from 'lucide-svelte';

  interface Props {
    gameState: FullGameState;
    onClose: () => void;
  }

  let { gameState, onClose }: Props = $props();

  let selected = $state<string[]>([]);
  let candidates = $derived(gameState.players.filter((p) => selected.includes(p.id)));

  // A two-sided coin that shows a new name each time it turns over. Names change
  // only while a face is turned away, so it reads as a real coin with N sides.
  // `sequence[k]` is the name showing after k half-turns.
  let sequence = $state<string[]>([]);
  let angle = $state(0);
  let lift = $state(0);
  let drift = $state(0);
  let flipping = $state(false);
  let winner = $state<string | null>(null);
  let frame = 0;

  const halfTurns = $derived(Math.floor((angle + 90) / 180));
  const nameAt = (k: number) => sequence[Math.max(0, k)] ?? '';
  // Each face shows the current name while visible and the next one while hidden.
  let frontName = $derived(halfTurns % 2 === 0 ? nameAt(halfTurns) : nameAt(halfTurns + 1));
  let backName = $derived(halfTurns % 2 === 1 ? nameAt(halfTurns) : nameAt(halfTurns + 1));

  function toggle(id: string) {
    if (flipping) return;
    winner = null;
    selected = selected.includes(id) ? selected.filter((x) => x !== id) : [...selected, id];
  }

  function randomInt(max: number): number {
    const buf = new Uint32Array(1);
    crypto.getRandomValues(buf);
    return buf[0] % max;
  }

  const easeOutCubic = (t: number) => 1 - (1 - t) ** 3;

  function flip() {
    if (flipping || candidates.length < 2) return;
    const names = candidates.map((p) => playerNameById(gameState, p.id));
    const n = names.length;
    const pick = randomInt(n);
    const turns = 12 + randomInt(6);
    const start = (((pick - turns) % n) + n) % n;
    sequence = Array.from({ length: turns + 2 }, (_, k) => names[(start + k) % n]);
    winner = null;

    const finalAngle = turns * 180;
    const height = 170 + randomInt(60);
    const sideways = randomInt(81) - 40;

    if (matchMedia('(prefers-reduced-motion: reduce)').matches) {
      angle = finalAngle;
      lift = 0;
      drift = 0;
      winner = names[pick];
      return;
    }

    flipping = true;
    angle = 0;
    const flightMs = 1500;
    const settleMs = 700;
    const t0 = performance.now();

    const step = (now: number) => {
      const t = now - t0;
      if (t < flightMs) {
        const p = t / flightMs;
        angle = finalAngle * easeOutCubic(p);
        lift = height * 4 * p * (1 - p);
        drift = sideways * p;
      } else if (t < flightMs + settleMs) {
        // Land: a small bounce and a wobble that dies out, never past edge-on.
        const s = (t - flightMs) / settleMs;
        const decay = (1 - s) ** 2;
        angle = finalAngle + 18 * decay * Math.sin(s * Math.PI * 5);
        lift = 22 * decay * Math.abs(Math.sin(s * Math.PI * 2));
        drift = sideways;
      } else {
        angle = finalAngle;
        lift = 0;
        drift = sideways;
        flipping = false;
        winner = names[pick];
        return;
      }
      frame = requestAnimationFrame(step);
    };
    frame = requestAnimationFrame(step);
  }

  onDestroy(() => cancelAnimationFrame(frame));

  // Shadow shrinks and fades as the coin rises.
  let shadowScale = $derived(1 - Math.min(lift, 230) / 330);
  // Light sweeps across the face as it turns.
  let sheen = $derived(50 + 45 * Math.sin((angle * Math.PI) / 180));
</script>

<div class="fixed inset-0 z-50 flex flex-col items-center justify-center gap-6 bg-base-300 p-6">
  <button class="btn btn-ghost btn-sm absolute right-4 top-4" onclick={onClose} aria-label="Close">
    <X size={20} />
  </button>

  <h2 class="text-2xl font-bold">Flip a Coin</h2>

  <div class="relative flex h-80 w-full max-w-md items-end justify-center">
    <div
      class="absolute bottom-6 h-5 w-36 rounded-[50%] bg-black/30 blur-md"
      style:transform={`translateX(${drift}px) scale(${shadowScale})`}
      style:opacity={0.35 + 0.65 * shadowScale}
    ></div>

    <div class="coin-stage mb-10" style:transform={`translate(${drift}px, ${-lift}px)`}>
      <div class="coin" style:transform={`rotateX(${angle}deg)`}>
        <div class="face" style:--sheen={`${sheen}%`}>
          <span class="name">{frontName || '?'}</span>
        </div>
        <div class="face back" style:--sheen={`${100 - sheen}%`}>
          <span class="name">{backName || '?'}</span>
        </div>
      </div>
    </div>
  </div>

  <div class="h-8 text-center">
    {#if winner}
      <p class="text-xl font-semibold">{winner}</p>
    {:else if candidates.length < 2}
      <p class="text-sm text-base-content/60">Tap at least two players to put them on the coin.</p>
    {/if}
  </div>

  <div class="flex max-w-xl flex-wrap justify-center gap-2">
    {#each gameState.players as p (p.id)}
      <button
        class="btn btn-sm"
        class:btn-primary={selected.includes(p.id)}
        class:btn-outline={!selected.includes(p.id)}
        disabled={flipping}
        onclick={() => toggle(p.id)}
      >
        {playerNameById(gameState, p.id)}
      </button>
    {/each}
  </div>

  <div class="flex gap-2">
    <button class="btn btn-primary" disabled={flipping || candidates.length < 2} onclick={flip}>
      {winner ? 'Flip again' : 'Flip'}
    </button>
    <button class="btn btn-ghost" onclick={onClose}>Done</button>
  </div>
</div>

<style>
  .coin-stage {
    perspective: 900px;
  }
  .coin {
    position: relative;
    width: 11rem;
    height: 11rem;
    transform-style: preserve-3d;
  }
  .face {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.25rem;
    border-radius: 9999px;
    backface-visibility: hidden;
    background:
      linear-gradient(115deg, transparent calc(var(--sheen) - 18%), rgb(255 255 255 / 0.45) var(--sheen), transparent calc(var(--sheen) + 18%)),
      radial-gradient(circle at 35% 30%, #fde68a, #eab308 45%, #a16207 100%);
    box-shadow:
      inset 0 0 0 6px #ca8a04,
      inset 0 0 0 9px #fde68a,
      inset 0 0 0 11px #a16207,
      0 4px 10px rgb(0 0 0 / 0.25);
  }
  .back {
    transform: rotateX(180deg);
  }
  .name {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-align: center;
    font-size: 1.35rem;
    font-weight: 800;
    letter-spacing: 0.02em;
    color: #713f12;
    text-shadow: 0 1px 0 rgb(255 255 255 / 0.5), 0 -1px 0 rgb(0 0 0 / 0.25);
  }
</style>
