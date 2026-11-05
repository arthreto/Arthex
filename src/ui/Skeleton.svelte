<script lang="ts">
  /** A shape standing for content that is still loading. */
  let { width, height, radius = '6px' }: { width: string; height: string; radius?: string } = $props();
</script>

<span class="skeleton" style:width style:height style:border-radius={radius}></span>

<style>
  .skeleton {
    position: relative;
    overflow: hidden;
    display: block;
    flex: none;
    background: var(--skeleton);
  }

  .skeleton::before {
    content: '';
    position: absolute;
    inset: 0;
    transform: translateX(-100%);
    background: linear-gradient(90deg, transparent, var(--skeleton-shine), transparent);
    animation: shimmer 1.4s ease-in-out infinite;
  }

  @keyframes shimmer {
    to {
      transform: translateX(100%);
    }
  }
</style>
