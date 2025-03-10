<script lang="ts">
  import { numberFormatter } from "$lib/fmt/number";
  import type { PostViewMessage } from "$lib/protos/posts/view";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import Markdown from "svelte-exmarkdown";

  interface Props extends PostViewMessage {}
  let props: Props = $props();
</script>

<article>
  {#if props.author}
    <section class="author">
      <img src={props.author.avatar} alt={props.author.did} />
      <span>{props.author.displayName}</span>
    </section>
  {/if}
  {#if props.content}
    <section class="text">
      <Markdown md={props.content.markdown} />
    </section>
  {/if}
  {#if props.embed.oneofKind != "others"}
    <section class="embed" class:images={props.embed.oneofKind == "images"}>
      {#if props.embed.oneofKind == "images"}
        {#each props.embed.images.images as image}
          <img src={image.fullsize} alt={image.alt} />
        {/each}
      {:else if props.embed.oneofKind == "external"}
        <a
          href="#____"
          tabindex="0"
          onclick={() => {
            if (props.embed.oneofKind == "external") {
              openUrl(props.embed.external.uri);
            }
          }}
          onkeydown={(e) => {
            if (e.key == "Enter") {
              if (props.embed.oneofKind == "external") {
                openUrl(props.embed.external.uri);
              }
            }
          }}
        >
          <img
            src={props.embed.external.thumb}
            alt={props.embed.external.description}
          />
        </a>
      {:else}
        <p>The post embed can't be shown</p>
      {/if}
    </section>
  {/if}
  <section class="stats">
    {#if props.likeCount}
      {numberFormatter.format(props.likeCount)} like{#if Number(props.likeCount) != 1}s{/if}
    {/if}
    {#if props.replyCount}
      {numberFormatter.format(props.replyCount)} repl{#if Number(props.replyCount) != 1}ies{:else}y{/if}
    {/if}
    {#if props.repostCound}
      {numberFormatter.format(props.repostCound)} repost{#if Number(props.repostCound) != 1}s{/if}
    {/if}
  </section>
</article>

<style lang="scss">
  article {
    --border: 3px;
    --border-radius: 3px;
    border-style: solid;
    border-width: var(--border);
    border-color: var(--post-border-color, #131067);
    border-radius: var(--border-radius);
    box-shadow: var(--post-border-color, #131067) 0px var(--border) 0px;
    display: grid;
  }
</style>
