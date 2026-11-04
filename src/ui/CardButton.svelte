<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    /** Muted text after the title ("2 min ago"). */
    note?: string;
    /** Where the card leads ("FileSize"), shown at the top right with an arrow. */
    link: string;
    onclick: () => void;
    children: Snippet;
  }

  let { title, note, link, onclick, children }: Props = $props();
</script>

<button class="card" {onclick}>
  <span class="head">
    <span class="title"
      >{title}{#if note}<span class="note">{` · ${note}`}</span>{/if}</span
    >
    <span class="link">{link} →</span>
  </span>
  {@render children()}
</button>

<style>
  .card {
    width: 100%;
    padding: 14px 16px;
    border-radius: 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    text-align: left;
    background: var(--surface);
    border: 1px solid var(--border);
    transition: background-color var(--motion-fast);
  }

  .card:hover {
    background: var(--surface-hover);
  }

  .head {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    font-size: 13.5px;
    font-weight: 600;
  }

  .note {
    font-size: 12px;
    font-weight: 400;
    color: var(--text-subtle);
  }

  .link {
    flex: none;
    font-size: 12.5px;
    font-weight: 500;
    color: var(--accent);
  }
</style>
