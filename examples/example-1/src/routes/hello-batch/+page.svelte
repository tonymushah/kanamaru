<script lang="ts">
  let inputs: string[] = $state([]);
</script>

<form
  onsubmit={(e) => {
    e.preventDefault();
    console.log(inputs);
  }}
>
  {#each inputs as _, index}
    <input bind:value={inputs[index]} />
  {/each}
  <button
    type="button"
    onclick={() => {
      inputs.push("");
    }}>New</button
  >
  <button type="submit">Send</button>
</form>
