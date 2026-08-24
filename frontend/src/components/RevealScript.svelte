<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { Role, Module } from '../types';
  import { buildRevealScript, pauseMs } from '../reveal';
  import { X, Play } from 'lucide-svelte';

  interface Props {
    roles: Role[];
    modules: Module[];
    onClose: () => void;
  }

  let { roles, modules, onClose }: Props = $props();

  const script = $derived(buildRevealScript(roles, modules));

  let started = $state(false);
  let done = $state(false);
  let currentIndex = $state(-1);
  let cancelled = false;
  let timer: ReturnType<typeof setTimeout> | null = null;

  const ttsAvailable = typeof window !== 'undefined' && 'speechSynthesis' in window;

  // ── Voice selection ────────────────────────────────────────────────────────
  // We only narrate English, so the picker lists en-US voices. Voices come from
  // the viewer's browser/OS and vary in quality; auto-pick a good one, allow override.
  let voices = $state<SpeechSynthesisVoice[]>([]);
  let selectedVoiceName = $state<string>(localStorage.getItem('reveal_voice') || '');

  const isEnUs = (v: SpeechSynthesisVoice) =>
    v.lang.replace('_', '-').toLowerCase().startsWith('en-us');

  // Quality rank, lower = better. Premium/Enhanced/Siri appear on Chrome/Safari
  // (Firefox macOS lists base voices only). Novelties (Bad News, Zarvox…) sink.
  function voiceRank(v: SpeechSynthesisVoice): number {
    const n = v.name;
    if (/premium/i.test(n)) return 0;
    if (/enhanced/i.test(n)) return 1;
    if (/siri/i.test(n)) return 2;
    if (/google|natural/i.test(n) || !v.localService) return 3;
    if (/samantha|ava|allison|alex|evan|nathan|tom|serena|zoe|susan|joelle/i.test(n)) return 4;
    return 5;
  }

  const byRank = (a: SpeechSynthesisVoice, b: SpeechSynthesisVoice) =>
    voiceRank(a) - voiceRank(b) || a.name.localeCompare(b.name);

  // Best voices first so they're reachable without scrolling a huge native list.
  const sortedVoices = $derived([...voices.filter(isEnUs)].sort(byRank));

  function loadVoices() {
    if (!ttsAvailable) return;
    const all = window.speechSynthesis.getVoices();
    voices = all;
    const en = all.filter(isEnUs).sort(byRank);
    if (en.length && !en.some((v) => v.name === selectedVoiceName)) {
      selectedVoiceName = en[0].name;
    }
  }

  // onMount (not $effect): voice loading writes the same state it reads, and
  // Firefox fires `voiceschanged` async — doing this in an $effect loops.
  onMount(() => {
    if (!ttsAvailable) return;
    loadVoices();
    window.speechSynthesis.addEventListener('voiceschanged', loadVoices);
    return () => window.speechSynthesis.removeEventListener('voiceschanged', loadVoices);
  });

  function onVoiceChange() {
    if (selectedVoiceName) localStorage.setItem('reveal_voice', selectedVoiceName);
  }

  // ── Playback ─────────────────────────────────────────────────────────────
  function speak(text: string): Promise<void> {
    return new Promise((resolve) => {
      if (!ttsAvailable) {
        resolve();
        return;
      }
      const u = new SpeechSynthesisUtterance(text);
      const voice = voices.find((v) => v.name === selectedVoiceName);
      if (voice) u.voice = voice;

      let finished = false;
      let watchdog: ReturnType<typeof setTimeout>;
      const finish = () => {
        if (finished) return;
        finished = true;
        clearTimeout(watchdog);
        resolve();
      };
      u.onend = finish;
      u.onerror = finish;
      // Safety net: if onend never fires (Firefox quirk), resolve after a generous
      // estimate so playback can't hard-hang.
      const words = text.split(/\s+/).length;
      watchdog = setTimeout(finish, 3000 + words * 450);

      window.speechSynthesis.speak(u);
    });
  }

  function delay(ms: number): Promise<void> {
    return new Promise((resolve) => {
      timer = setTimeout(resolve, ms);
    });
  }

  async function play() {
    started = true;
    for (let i = 0; i < script.length; i++) {
      if (cancelled) return;
      currentIndex = i;
      await speak(script[i].text);
      if (cancelled) return;
      await delay(pauseMs(script[i].pause));
    }
    if (!cancelled) done = true;
  }

  function stop() {
    cancelled = true;
    if (timer) clearTimeout(timer);
    if (ttsAvailable) window.speechSynthesis.cancel();
    onClose();
  }

  onDestroy(() => {
    cancelled = true;
    if (timer) clearTimeout(timer);
    if (ttsAvailable) window.speechSynthesis.cancel();
  });
</script>

<div class="fixed inset-0 z-50 bg-base-300 flex flex-col items-center justify-center p-8 text-center">
  <button class="btn btn-ghost btn-sm absolute top-4 right-4" onclick={stop} aria-label="Close">
    <X size={20} />
  </button>

  {#if !started}
    <h2 class="text-3xl font-bold mb-3">Role Reveal</h2>
    <p class="max-w-md text-base-content/70 mb-2">
      This will narrate the reveal out loud. Press Start, then <strong>everyone close your eyes</strong> and follow along.
    </p>
    {#if !ttsAvailable}
      <p class="text-warning text-sm mb-2">Your browser can't speak — the lines will show on screen only.</p>
    {:else if sortedVoices.length}
      <div class="w-full max-w-xs mt-4 text-left">
        <span class="text-sm text-base-content/60 mb-1 block">Narrator voice</span>
        <select
          class="w-full rounded-lg border border-base-300 bg-base-100 p-1 text-sm focus:outline-none"
          size="8"
          bind:value={selectedVoiceName}
          onchange={onVoiceChange}
        >
          {#each sortedVoices as v}
            <option value={v.name} class="rounded px-2 py-1">{v.name}</option>
          {/each}
        </select>
      </div>
    {/if}
    <button class="btn btn-lg btn-primary mt-4" onclick={play}>
      <Play size={18} /> Start
    </button>
  {:else if !done}
    <p class="text-sm text-base-content/50 mb-6">{currentIndex + 1} / {script.length}</p>
    <p class="text-3xl md:text-5xl font-semibold leading-snug max-w-4xl">
      {script[currentIndex]?.text ?? ''}
    </p>
  {:else}
    <h2 class="text-4xl font-bold mb-6">Everyone, open your eyes.</h2>
    <button class="btn btn-lg btn-primary" onclick={onClose}>Done</button>
  {/if}
</div>
