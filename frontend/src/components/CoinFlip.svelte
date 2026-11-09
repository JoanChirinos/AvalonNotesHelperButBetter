<script lang="ts">
  import { onDestroy } from 'svelte';
  import type { FullGameState } from '../types';
  import { playerNameById } from '../derived';
  interface Props {
    gameState: FullGameState;
  }

  let { gameState }: Props = $props();

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

  // Zero-sided coin: it collapses into a black hole and swallows the page.
  // Purely visual; nothing about the game changes, and a refresh undoes it.
  let zeroSided = $state(false);
  let voidLevel = $state(0);
  let swallowed = $state(false);
  let coinEl: HTMLElement | undefined = $state();
  let suck: Animation | null = null;
  let savedScroll = 0;

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

  const reducedMotion = () => matchMedia('(prefers-reduced-motion: reduce)').matches;

  function swallowPage() {
    const app = document.getElementById('app');
    document.documentElement.classList.add('anh-void');
    if (!app || !coinEl || reducedMotion()) {
      swallowed = true;
      return;
    }
    // A transformed #app becomes the containing block for its fixed children, so pin
    // it to the viewport first or the open popup would jump on a scrolled page.
    savedScroll = window.scrollY;
    Object.assign(app.style, { position: 'fixed', inset: '0', overflow: 'hidden' });
    app.scrollTop = savedScroll;
    const coin = coinEl.getBoundingClientRect();
    app.style.transformOrigin = `${coin.left + coin.width / 2}px ${coin.top + coin.height / 2}px`;
    suck = app.animate(
      [
        { transform: 'none', filter: 'none', opacity: 1 },
        { transform: 'scale(0.35) rotate(240deg)', filter: 'blur(2px)', opacity: 1, offset: 0.6 },
        { transform: 'scale(0) rotate(720deg)', filter: 'blur(8px)', opacity: 0 },
      ],
      { duration: 1500, easing: 'cubic-bezier(0.55, 0, 0.9, 0.35)', fill: 'forwards' },
    );
    suck.onfinish = () => (swallowed = true);
  }

  function resetVoid() {
    cancelAnimationFrame(frame);
    suck?.cancel();
    suck = null;
    const app = document.getElementById('app');
    if (app && app.style.position === 'fixed') {
      Object.assign(app.style, { position: '', inset: '', overflow: '', transformOrigin: '' });
      window.scrollTo(0, savedScroll);
    }
    document.documentElement.classList.remove('anh-void');
    swallowed = false;
    zeroSided = false;
    voidLevel = 0;
    flipping = false;
    angle = 0;
    lift = 0;
    drift = 0;
  }

  function restore() {
    swallowed = false;
    if (!suck) {
      resetVoid();
      return;
    }
    suck.onfinish = resetVoid;
    suck.reverse();
  }

  function flipNothing() {
    zeroSided = true;
    winner = null;
    flipping = true;
    if (reducedMotion()) {
      voidLevel = 1;
      swallowPage();
      return;
    }
    const riseMs = 700;
    const collapseMs = 650;
    const t0 = performance.now();
    const step = (now: number) => {
      const t = now - t0;
      if (t < riseMs) {
        const p = t / riseMs;
        lift = 90 * easeOutCubic(p);
        angle = 900 * easeOutCubic(p);
      } else if (t < riseMs + collapseMs) {
        const p = (t - riseMs) / collapseMs;
        lift = 90;
        angle = 900 + 120 * p;
        voidLevel = p;
      } else {
        voidLevel = 1;
        swallowPage();
        return;
      }
      frame = requestAnimationFrame(step);
    };
    frame = requestAnimationFrame(step);
  }

  function flip() {
    if (flipping) return;
    if (candidates.length === 0) {
      flipNothing();
      return;
    }
    const names = candidates.map((p) => playerNameById(gameState, p.id));
    const n = names.length;
    const pick = randomInt(n);
    const turns = 12 + randomInt(6);
    const start = (((pick - turns) % n) + n) % n;
    sequence = Array.from({ length: turns + 2 }, (_, k) => names[(start + k) % n]);
    winner = null;

    const finalAngle = turns * 180;
    const height = 70 + randomInt(25);
    const sideways = randomInt(41) - 20;

    if (reducedMotion()) {
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
        lift = 14 * decay * Math.abs(Math.sin(s * Math.PI * 2));
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

  onDestroy(resetVoid);

  // Render outside #app so the restore screen isn't swallowed along with the page.
  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return { destroy: () => node.remove() };
  }

  // Shrink long names to fit the face instead of truncating them.
  const nameSize = (name: string) => `${Math.min(16, 92 / (Math.max(name.length, 1) * 0.62))}px`;

  // Shadow shrinks and fades as the coin rises.
  let shadowScale = $derived(1 - Math.min(lift, 95) / 140);
  // Light sweeps across the face as it turns.
  let sheen = $derived(50 + 45 * Math.sin((angle * Math.PI) / 180));
</script>

<div class="flex flex-col items-center gap-3">
  <div class="relative flex h-60 w-full items-end justify-center">
    <div
      class="absolute bottom-3 h-3 w-24 rounded-[50%] bg-black/30 blur-md"
      style:transform={`translateX(${drift}px) scale(${shadowScale})`}
      style:opacity={0.35 + 0.65 * shadowScale}
    ></div>

    <div class="coin-stage mb-6" style:transform={`translate(${drift}px, ${-lift}px)`}>
      <div class="coin" bind:this={coinEl} style:transform={`rotateX(${angle}deg) scale(${1 + 0.35 * voidLevel})`}>
        <div class="face" style:--sheen={`${sheen}%`}>
          <div class="abyss" style:opacity={voidLevel}></div>
          {#if !zeroSided}
            <span class="name" style:font-size={nameSize(frontName)}>{frontName || '?'}</span>
          {/if}
        </div>
        <div class="face back" style:--sheen={`${100 - sheen}%`}>
          <div class="abyss" style:opacity={voidLevel}></div>
          {#if !zeroSided}
            <span class="name" style:font-size={nameSize(backName)}>{backName || '?'}</span>
          {/if}
        </div>
      </div>
    </div>
  </div>

  <div class="h-6 text-center">
    {#if winner}
      <p class="text-lg font-semibold">{winner}</p>
    {:else if candidates.length === 0}
      <p class="text-xs text-base-content/60">Tap players to put them on the coin.</p>
    {/if}
  </div>

  <div class="flex flex-wrap justify-center gap-1">
    {#each gameState.players as p (p.id)}
      <button
        class="btn btn-xs"
        class:btn-primary={selected.includes(p.id)}
        class:btn-outline={!selected.includes(p.id)}
        disabled={flipping}
        onclick={() => toggle(p.id)}
      >
        {playerNameById(gameState, p.id)}
      </button>
    {/each}
  </div>

  <button class="btn btn-sm btn-primary w-full" disabled={flipping} onclick={flip}>
    {winner ? 'Flip again' : 'Flip'}
  </button>
</div>

{#if swallowed}
  <div class="void-screen" use:portal>
    <div class="singularity"></div>
    <p class="text-sm text-white/50">The void has claimed your game.</p>
    <button class="btn btn-sm btn-outline border-white/30 text-white/80 hover:bg-white/10" onclick={restore}>Restore</button>
  </div>
{/if}

<style>
  .coin-stage {
    perspective: 900px;
  }
  .coin {
    position: relative;
    width: 7.5rem;
    height: 7.5rem;
    transform-style: preserve-3d;
  }
  .face {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0.85rem;
    border-radius: 9999px;
    backface-visibility: hidden;
    background:
      linear-gradient(115deg, transparent calc(var(--sheen) - 18%), rgb(255 255 255 / 0.45) var(--sheen), transparent calc(var(--sheen) + 18%)),
      radial-gradient(circle at 35% 30%, #fde68a, #eab308 45%, #a16207 100%);
    box-shadow:
      inset 0 0 0 4px #ca8a04,
      inset 0 0 0 6px #fde68a,
      inset 0 0 0 8px #a16207,
      0 4px 10px rgb(0 0 0 / 0.25);
  }
  .back {
    transform: rotateX(180deg);
  }
  .abyss {
    position: absolute;
    inset: 0;
    border-radius: 9999px;
    background:
      conic-gradient(from 0deg, transparent, rgb(147 51 234 / 0.55), transparent 35%, rgb(59 130 246 / 0.4), transparent 70%),
      radial-gradient(circle, #000 38%, #0b0215 62%, #1e0b3a 100%);
    box-shadow: 0 0 24px 6px rgb(88 28 135 / 0.6);
    animation: swirl 1.2s linear infinite;
  }
  .void-screen {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    background: #000;
    animation: fade-in 0.6s ease-out;
  }
  .singularity {
    width: 3rem;
    height: 3rem;
    border-radius: 9999px;
    background:
      conic-gradient(from 0deg, transparent, rgb(147 51 234 / 0.6), transparent 40%, rgb(59 130 246 / 0.45), transparent 75%),
      radial-gradient(circle, #000 45%, #0b0215 70%, transparent 100%);
    box-shadow: 0 0 30px 8px rgb(88 28 135 / 0.45);
    animation: swirl 2.4s linear infinite;
  }
  @keyframes swirl {
    to {
      transform: rotate(360deg);
    }
  }
  @keyframes fade-in {
    from {
      opacity: 0;
    }
  }
  :global(html.anh-void),
  :global(html.anh-void body) {
    background: #000;
    transition: background 0.8s ease-in;
  }
  .name {
    max-width: 100%;
    white-space: nowrap;
    text-align: center;
    font-weight: 800;
    letter-spacing: 0.02em;
    color: #713f12;
    text-shadow: 0 1px 0 rgb(255 255 255 / 0.5), 0 -1px 0 rgb(0 0 0 / 0.25);
  }
</style>
