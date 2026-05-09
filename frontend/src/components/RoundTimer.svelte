<script lang="ts">
  import { onDestroy } from 'svelte';
  import { Play, Pause, RotateCcw, Plus, Minus } from 'lucide-svelte';

  let totalSeconds = $state(120);
  let remaining = $state(120);
  let running = $state(false);
  let interval: ReturnType<typeof setInterval> | null = null;

  function handleRoundChange() { restart(); start(); }

  $effect(() => {
    window.addEventListener('anh:round-change', handleRoundChange);
    return () => window.removeEventListener('anh:round-change', handleRoundChange);
  });

  onDestroy(() => { if (interval) clearInterval(interval); });

  let minutes = $derived(Math.floor(remaining / 60));
  let seconds = $derived(remaining % 60);
  let display = $derived(`${minutes}:${String(seconds).padStart(2, '0')}`);
  let isExpired = $derived(remaining <= 0);

  function start() {
    if (remaining <= 0) return;
    running = true;
    interval = setInterval(() => {
      remaining--;
      if (remaining <= 0) {
        remaining = 0;
        stop();
        playAlarm();
      }
    }, 1000);
  }

  function stop() {
    running = false;
    if (interval) { clearInterval(interval); interval = null; }
  }

  function restart() {
    stop();
    remaining = totalSeconds;
  }

  function addMinute() {
    totalSeconds += 60;
    remaining += 60;
  }

  function subtractMinute() {
    if (totalSeconds <= 60) return;
    totalSeconds -= 60;
    remaining = Math.max(0, remaining - 60);
  }

  function playAlarm() {
    try {
      const ctx = new AudioContext();
      for (let i = 0; i < 3; i++) {
        const osc = ctx.createOscillator();
        const gain = ctx.createGain();
        osc.connect(gain);
        gain.connect(ctx.destination);
        osc.frequency.value = 800;
        gain.gain.value = 0.3;
        osc.start(ctx.currentTime + i * 0.3);
        osc.stop(ctx.currentTime + i * 0.3 + 0.15);
      }
    } catch {}
  }
</script>

<div class="flex items-center gap-1">
  <button class="btn btn-ghost btn-xs" onclick={subtractMinute}><Minus size={12} /></button>
  <span
    class="font-mono text-sm font-semibold min-w-12 text-center"
    class:text-error={isExpired}
    class:animate-pulse={isExpired}
  >{display}</span>
  <button class="btn btn-ghost btn-xs" onclick={addMinute}><Plus size={12} /></button>

  {#if running}
    <button class="btn btn-ghost btn-xs" onclick={stop}><Pause size={14} /></button>
  {:else}
    <button class="btn btn-ghost btn-xs" onclick={start}><Play size={14} /></button>
  {/if}
  <button class="btn btn-ghost btn-xs" onclick={restart}><RotateCcw size={14} /></button>
</div>
