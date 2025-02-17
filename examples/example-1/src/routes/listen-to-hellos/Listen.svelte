<script lang="ts">
  import { transport } from "$lib/protos/plugin1";
  import { HelloServiceClient } from "$lib/protos/plugin1/myprotos.client";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  let msg: string | null = $state(null);
  let error: Error | null = $state(null);
  const helloClient = new HelloServiceClient(transport);
  let abortController = new AbortController();

  let unlistens: UnlistenFn[] = [];
  function unlisten() {
    unlistens.forEach((u) => u());
    unlistens = [];
  }
  function clear() {
    msg = null;
    error = null;
  }
  function abort() {
    abortController.abort();
    abortController = new AbortController();
  }
  async function run() {
    const res = helloClient.listenToHellos(
      {},
      {
        abort: abortController.signal,
      }
    );
    let onMess = res.responses.onMessage((mess) => {
      msg = mess.response;
    });
    let onError = res.responses.onError((err) => {
      error = err;
    });
    unlistens.push(onMess, onError);
  }

  onMount(async () => run());

  onDestroy(() => {
    unlisten();
    clear();
    abort();
  });
</script>

<p>
  {#if msg == null}
    <i>No messages</i>
  {:else}
    {msg}
  {/if}
</p>

{#if error != null}
  <p class="error">{error.message}</p>
{/if}

<div>
  <button onclick={abort}> Abort </button>
  <button
    onclick={async () => {
      abort();
      unlisten();
      clear();
      await run();
    }}
  >
    Run
  </button>
  <button onclick={clear}> Clear </button>
</div>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;

    color: #0f0f0f;
    background-color: #f6f6f6;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
  }
  .error {
    color: red;
  }
  button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    color: #0f0f0f;
    background-color: #ffffff;
    transition: border-color 0.25s;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  }

  button {
    cursor: pointer;
  }

  button:hover {
    border-color: #396cd8;
  }
  button:active {
    border-color: #396cd8;
    background-color: #e8e8e8;
  }
  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }

    button {
      color: #ffffff;
      background-color: #0f0f0f98;
    }
    button:active {
      background-color: #0f0f0f69;
    }
  }
</style>
