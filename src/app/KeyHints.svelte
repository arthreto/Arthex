<script lang="ts">
  import { t } from '../core/i18n';
  import Keycap from '../ui/Keycap.svelte';
</script>

<footer>
  <span><span class="keys"><Keycap>↑</Keycap><Keycap>↓</Keycap></span>{t('hints.navigate')}</span>
  <span><Keycap>{t('keys.enter')}</Keycap>{t('hints.open')}</span>
  <span
    ><span class="keys"><Keycap>{t('keys.ctrl')}</Keycap>+<Keycap>{t('keys.enter')}</Keycap></span>{t(
      'hints.openFolder',
    )}</span
  >
  <span
    ><span class="keys"><Keycap>{t('keys.ctrl')}</Keycap>+<Keycap>{t('keys.tab')}</Keycap></span>{t(
      'hints.switchTab',
    )}</span
  >
  <span><Keycap>{t('keys.escape')}</Keycap>{t('hints.close')}</span>
</footer>

<style>
  footer {
    height: 28px;
    flex: none;
    margin-top: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 28px;
    font-size: 12px;
    color: var(--text-subtle);
  }

  footer > span {
    display: flex;
    align-items: center;
    gap: 9px;
  }

  .keys {
    display: flex;
    align-items: center;
    gap: 4px;
  }
</style>
