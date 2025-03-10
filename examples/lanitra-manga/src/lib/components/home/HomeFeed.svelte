<script lang="ts">
  import transport from "$lib/protos/tauri-transport";
  import SimplePost from "../post/SimplePost.svelte";
  import { homeQuery } from "./query";

  const homeFeedQuery = homeQuery(transport);
  let homeFeedQueryRes = $derived($homeFeedQuery);

  let data = $derived.by(
    () => homeFeedQueryRes.data?.pages.flatMap((e) => e.feed) ?? []
  );
</script>

<section>
  {#if homeFeedQueryRes.error != null}
    <div class="error">
      <p>{homeFeedQueryRes.error.name}</p>
      <p>{homeFeedQueryRes.error.message}</p>
    </div>
  {/if}
  {#each data as inner_data}
    {#if inner_data.post}
      <SimplePost {...inner_data.post} />
    {/if}
  {:else}
    <p>No posts in the main feed</p>
  {/each}
  {#if homeFeedQueryRes.isFetching}
    <div>
      <p>Loading...</p>
    </div>
  {/if}

  {#if homeFeedQueryRes.hasNextPage}
    <button
      onclick={() => {
        homeFeedQueryRes.fetchNextPage();
      }}
      disabled={homeFeedQueryRes.isFetching}
    >
      Load more...
    </button>
  {/if}
</section>
