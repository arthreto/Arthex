<script lang="ts">
  import { t } from '../core/i18n';
  import { search } from './search.svelte';
</script>

<div class="results fade-in">
  <p class="empty">{t('search.noResults', { query: search.query.trim() })}</p>
</div>

<style>
  .results {
    position: absolute;
    inset: 0;
  }

  .empty {
    padding: 70px 0;
    text-align: center;
    font-size: 14px;
    color: var(--text-muted);
  }
</style>
