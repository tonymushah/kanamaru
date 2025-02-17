<script lang="ts">
  import { transport } from "$lib/protos/plugin1";
  import { HelloService } from "$lib/protos/plugin1/myprotos";
  import { HelloServiceClient } from "$lib/protos/plugin1/myprotos.client";
  import { sleep } from "$lib/sleep";
  import { debounce } from "lodash";
  import { onDestroy } from "svelte";

  const helloClient = new HelloServiceClient(transport);

  let withResponses = $state(false);

  let responses: string[] = $state([]);

  let abortController = new AbortController();

  let inputs: string[] = $state([]);
  let isLoading = $state(false);

  const send = debounce(async () => {
    isLoading = true;
    try {
      responses = [];
      if (withResponses) {
        const req = helloClient.sayHelloWithResponses({
          abort: abortController.signal,
        });
        req.responses.onError(console.error);
        req.responses.onMessage((mess) => {
          responses.push(mess.response);
        });
        for (const name of inputs) {
          await req.requests.send({
            name,
          });
          await sleep(3000);
        }
        req.requests.complete();
        await req;
        console.log("Completed with responses");
      } else {
        const req = helloClient.sayHellos({
          abort: abortController.signal,
        });
        for (const name of inputs) {
          await req.requests.send({
            name,
          });
          await sleep(3000);
        }
        req.requests.complete();
        await req;
        console.log("Completed without responses");
      }
    } catch (e) {
      console.error(e);
    } finally {
      abortController = new AbortController();
      isLoading = false;
    }
  });

  onDestroy(() => {
    abortController.abort();
  });
</script>

{#if isLoading}
  <p class="sending">Sending...</p>
{/if}

<form
  onsubmit={async (e) => {
    e.preventDefault();
    if (!isLoading) {
      await send();
    }
  }}
>
  <div class="res">
    <section class="inputs">
      {#each inputs as _, index}
        <input bind:value={inputs[index]} placeholder="Name" />
      {/each}
    </section>
    <section class="result">
      <ul>
        {#each responses as response}
          <li>{response}</li>
        {:else}
          <i>Nothing...</i>
        {/each}
      </ul>
    </section>
  </div>

  <section class="buttons">
    <button
      type="button"
      onclick={() => {
        inputs.push("");
      }}
      class:disabled={isLoading}
      disabled={isLoading}
    >
      New
    </button>
    <button type="submit" disabled={isLoading} class:disabled={isLoading}>
      Send
    </button>
    <button
      type="button"
      class="with-response"
      class:withResponses
      onclick={() => {
        withResponses = !withResponses;
      }}
      disabled={isLoading}
      class:disabled={isLoading}
    >
      {#if withResponses}
        With
      {:else}
        Without
      {/if}
      responses
    </button>
  </section>
</form>

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
  .res {
    display: grid;
    grid-template-columns: repeat(2, calc(100cqh / 2));
    justify-content: center;
  }
  form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .inputs {
    display: flex;
    flex-direction: column;
    gap: 6px;
    align-items: center;
    justify-content: center;
  }
  input,
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

  input,
  button {
    outline: none;
  }
  p.sending {
    color: green;
  }
  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }

    input,
    button {
      color: #ffffff;
      background-color: #0f0f0f98;
    }
    button:active {
      background-color: #0f0f0f69;
    }
  }
</style>
