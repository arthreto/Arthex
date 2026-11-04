<script lang="ts">
  /** Round flag of a language. Languages without a drawing show their code. */
  let { code }: { code: string } = $props();
</script>

<span class="flag" aria-hidden="true">
  {#if code === 'fr'}
    <svg viewBox="0 0 3 2" preserveAspectRatio="xMidYMid slice">
      <rect width="1" height="2" fill="#002654" />
      <rect x="1" width="1" height="2" fill="#ffffff" />
      <rect x="2" width="1" height="2" fill="#ce1126" />
    </svg>
  {:else if code === 'en'}
    <svg viewBox="0 0 60 30" preserveAspectRatio="xMidYMid slice">
      <rect width="60" height="30" fill="#012169" />
      <path d="M0,0 L60,30 M60,0 L0,30" stroke="#ffffff" stroke-width="6" />
      <path d="M0,0 L60,30 M60,0 L0,30" stroke="#c8102e" stroke-width="2" />
      <path d="M30,0 v30 M0,15 h60" stroke="#ffffff" stroke-width="10" />
      <path d="M30,0 v30 M0,15 h60" stroke="#c8102e" stroke-width="6" />
    </svg>
  {:else}
    <span class="code">{code.toUpperCase()}</span>
  {/if}
</span>

<style>
  .flag {
    width: 54px;
    height: 54px;
    flex: none;
    border-radius: 50%;
    overflow: hidden;
    box-shadow:
      0 0 0 2px var(--border-strong),
      0 6px 16px rgba(0, 0, 0, 0.25);
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--surface-raised);
  }

  svg {
    width: 100%;
    height: 100%;
    display: block;
  }

  .code {
    font-size: 16px;
    font-weight: 700;
  }
</style>
