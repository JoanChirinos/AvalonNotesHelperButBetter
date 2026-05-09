<script lang="ts">
  import { MessageCircle, X, Send } from 'lucide-svelte';

  interface Props {
    gameId: string;
  }

  let { gameId }: Props = $props();

  interface NoteMessage {
    id: string;
    text: string;
    timestamp: number;
  }

  let open = $state(false);
  let input = $state('');
  let messages = $state<NoteMessage[]>([]);
  let messagesEl: HTMLDivElement;

  function storageKey() {
    return `anh_notes_${gameId}`;
  }

  function loadMessages(): NoteMessage[] {
    try {
      const raw = localStorage.getItem(storageKey());
      return raw ? JSON.parse(raw) : [];
    } catch { return []; }
  }

  function saveMessages() {
    localStorage.setItem(storageKey(), JSON.stringify(messages));
  }

  // Load on mount and when gameId changes
  $effect(() => {
    messages = loadMessages();
  });

  function scrollToBottom() {
    requestAnimationFrame(() => {
      if (messagesEl) messagesEl.scrollTop = messagesEl.scrollHeight;
    });
  }

  function sendMessage() {
    const text = input.trim();
    if (!text) return;
    messages = [...messages, {
      id: crypto.randomUUID(),
      text,
      timestamp: Date.now(),
    }];
    saveMessages();
    input = '';
    scrollToBottom();
  }

  function deleteMessage(id: string) {
    messages = messages.filter(m => m.id !== id);
    saveMessages();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }

  function formatTime(ts: number): string {
    return new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  $effect(() => {
    if (open) scrollToBottom();
  });
</script>

<!-- Floating button -->
<div class="fixed bottom-4 right-4 z-30">
  {#if open}
    <!-- Chat panel -->
    <div class="bg-base-100 border border-base-300 rounded-xl shadow-xl w-72 h-96 flex flex-col mb-2">
      <!-- Header -->
      <div class="flex items-center justify-between px-3 py-2 border-b border-base-300">
        <span class="text-sm font-semibold">Notes</span>
        <button class="btn btn-ghost btn-xs" onclick={() => open = false}>
          <X size={14} />
        </button>
      </div>

      <!-- Messages -->
      <div class="flex-1 overflow-y-auto p-2 space-y-2" bind:this={messagesEl}>
        {#if messages.length === 0}
          <p class="text-xs text-base-content/40 text-center mt-8">No notes yet</p>
        {:else}
          {#each messages as msg}
            <div class="group flex gap-1">
              <div class="bg-primary/10 rounded-lg px-2 py-1 flex-1">
                <p class="text-xs">{msg.text}</p>
                <p class="text-[10px] text-base-content/40 mt-0.5">{formatTime(msg.timestamp)}</p>
              </div>
              <button
                class="btn btn-ghost btn-xs opacity-0 group-hover:opacity-100 self-start"
                onclick={() => deleteMessage(msg.id)}
              >
                <X size={10} />
              </button>
            </div>
          {/each}
        {/if}
      </div>

      <!-- Input -->
      <div class="border-t border-base-300 p-2 flex gap-1">
        <input
          type="text"
          class="input input-bordered input-sm flex-1 text-xs"
          placeholder="Type a note..."
          bind:value={input}
          onkeydown={handleKeydown}
        />
        <button class="btn btn-primary btn-sm btn-square" onclick={sendMessage} disabled={!input.trim()}>
          <Send size={14} />
        </button>
      </div>
    </div>
  {/if}

  <!-- Toggle bubble -->
  <button
    class="btn btn-circle btn-primary shadow-lg"
    onclick={() => open = !open}
  >
    <MessageCircle size={20} />
  </button>
</div>
