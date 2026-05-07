<script lang="ts">
  import type { FullGameState } from '../types';
  import { BASE_PATH } from '../api';
  import GameSetup from './GameSetup.svelte';
  import GameBoard from './GameBoard.svelte';
  import GameSummary from './GameSummary.svelte';

  interface Props {
    gameId: string;
    onNavigate: (path: string) => void;
  }

  let { gameId, onNavigate }: Props = $props();

  let gameState = $state<FullGameState | null>(null);
  let error = $state('');
  let ws = $state<WebSocket | null>(null);

  // Derived game phase
  let isSetup = $derived(
    gameState !== null && gameState.quests.every(q => q.rounds.length === 0)
  );
  let isFinished = $derived(gameState !== null && gameState.game.finished_at !== null);
  let isActive = $derived(gameState !== null && !isSetup && !isFinished);

  function connectWs() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const url = `${protocol}//${window.location.host}${BASE_PATH}api/games/${gameId}/ws`;
    const socket = new WebSocket(url);

    socket.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        if (msg.type === 'game_state') {
          gameState = msg.data;
        }
      } catch (e) {
        console.error('WS parse error:', e);
      }
    };

    socket.onclose = () => {
      // Reconnect after 1 second
      setTimeout(connectWs, 1000);
    };

    socket.onerror = () => {
      error = 'WebSocket connection error';
    };

    ws = socket;
  }

  $effect(() => {
    connectWs();
    return () => ws?.close();
  });
</script>

{#if error}
  <div class="alert alert-error mb-4">{error}</div>
{/if}

{#if !gameState}
  <div class="flex justify-center p-8">
    <span class="loading loading-spinner loading-lg"></span>
  </div>
{:else if isSetup}
  <GameSetup gameState={gameState} {onNavigate} />
{:else if isActive}
  <GameBoard gameState={gameState} {onNavigate} />
{:else if isFinished}
  <GameSummary gameState={gameState} {onNavigate} />
{/if}
