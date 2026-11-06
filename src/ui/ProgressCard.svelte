<script lang="ts">
  import { formatPercent } from '../core/format';
  import FileIcon from './FileIcon.svelte';

  interface Props {
    title: string;
    detail?: string;
    /** Between 0 and 100, or `null` when it cannot be estimated: then there is no bar. */
    percent: number | null;
    /** A reflection on the bar, for a page that waits for it. */
    shine?: boolean;
  }

  let { title, detail, percent, shine = false }: Props = $props();
</script>

<div class="card" role="status">
  <div class="line">
    <FileIcon folder size={16} />
    <span class="title">{title}</span>
    {#if detail}<span class="detail">{detail}</span>{/if}
    {#if percent !== null}<span class="percent">{formatPercent(percent / 100, 0)}</span>{/if}
  </div>
  {#if percent !== null}
    <div class="track"><div class="bar" class:shine style:width="{percent}%"></div></div>
  {/if}
</div>

<style>
  .card {
    padding: 12px 14px;
    border-radius: 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 9px;
  }

  .line {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
  }

  .title {
    font-weight: 600;
  }

  .detail {
    color: var(--text-subtle);
  }

  .percent {
    margin-left: auto;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .track {
    height: 4px;
    border-radius: 2px;
    background: var(--border);
    overflow: hidden;
  }

  .bar {
    position: relative;
    overflow: hidden;
    height: 100%;
    border-radius: 2px;
    background: var(--accent);
    transition: width 400ms var(--ease-out);
  }

  .shine::before {
    content: '';
    position: absolute;
    inset: 0;
    transform: translateX(-100%);
    background: linear-gradient(90deg, transparent, var(--progress-shine), transparent);
    animation: shine 1.4s ease-in-out infinite;
  }

  @keyframes shine {
    to {
      transform: translateX(100%);
    }
  }
</style>
