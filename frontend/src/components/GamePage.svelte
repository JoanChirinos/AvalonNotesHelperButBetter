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

  // Derived game phase
  let isSetup = $derived(
    gameState !== null && gameState.quests.every(q => q.rounds.length === 0)
  );
  let isFinished = $derived(gameState !== null && gameState.game.finished_at !== null);
  let isActive = $derived(gameState !== null && !isSetup && !isFinished);

  $effect(() => {
    let destroyed = false;
    let reconnectTimeout: ReturnType<typeof setTimeout>;
    let socket: WebSocket;
    let backoff = 1000;

    gameState = null;
    error = '';

    function connect() {
      if (destroyed) return;
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const url = `${protocol}//${window.location.host}${BASE_PATH}api/games/${gameId}/ws`;
      socket = new WebSocket(url);

      socket.onopen = () => { error = ''; backoff = 1000; };

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
        if (!destroyed) {
          reconnectTimeout = setTimeout(connect, backoff);
          backoff = Math.min(backoff * 2, 30000);
        }
      };

      socket.onerror = () => {
        error = 'WebSocket connection error';
      };
    }

    connect();
    return () => {
      destroyed = true;
      clearTimeout(reconnectTimeout);
      socket?.close();
    };
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
