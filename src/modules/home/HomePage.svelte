<script lang="ts">
  import logo from '../../assets/logo.png';
  import { t } from './i18n';
</script>

<div class="home">
  <img src={logo} alt="" width="88" height="88" />
  <h2>{t('welcome')}</h2>
  <p>{t('soon')}</p>
</div>

<style>
  .home {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 14px;
    text-align: center;
  }

  h2 {
    margin-top: 10px;
    font-size: 26px;
    font-weight: 600;
  }

  p {
    font-size: 14.5px;
    color: var(--text-muted);
  }
</style>
