<script lang="ts">
  import logo from '../assets/logo.png';
  import type { WhatsNew } from '../core/api';
  import { getLocale, t } from '../core/i18n';
  import Button from '../ui/Button.svelte';
  import { releaseNotes, type NoteKind } from './releaseNotes';

  let { notes, onclose }: { notes: WhatsNew; onclose: () => void } = $props();

  const titleId = $props.id();
  const sections = $derived(releaseNotes(notes.notes, getLocale().startsWith('fr')));
  const colors: Record<NoteKind, string> = {
    new: 'var(--tint-green)',
    improved: 'var(--tint-sky)',
    fixed: 'var(--warning)',
  };

  function onkeydown(event: KeyboardEvent) {
    if (event.key !== 'Escape') return;
    event.stopPropagation();
    onclose();
  }
</script>

<div class="scrim fade-in">
  <button class="backdrop" aria-label={t('common.close')} tabindex="-1" onclick={onclose}></button>
  <div
    class="dialog enter"
    role="dialog"
    aria-modal="true"
    aria-labelledby={titleId}
    tabindex="-1"
    {onkeydown}
  >
    <header>
      <img src={logo} alt="" width="58" height="58" />
      <div>
        <h2 id={titleId}>{t('whatsNew.title')}</h2>
        <span class="installed">{t('whatsNew.installed', { version: notes.version })}</span>
      </div>
    </header>
    <div class="sections scroll">
      {#each sections as section (section.kind)}
        <section style:--kind={colors[section.kind]}>
          <h3>{t(`whatsNew.${section.kind}`)}</h3>
          <ul>
            {#each section.items as item, position (position)}
              <li>{item}</li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>
    <div class="actions">
      <Button variant="primary" size="large" onclick={onclose}>{t('whatsNew.continue')}</Button>
    </div>
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 30;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--scrim);
    backdrop-filter: blur(22px) saturate(120%);
  }

  .backdrop {
    position: absolute;
    inset: 0;
    cursor: default;
  }

  .dialog {
    position: relative;
    width: min(920px, calc(100vw - 48px));
    height: min(640px, calc(100vh - 48px));
    padding: 36px 44px 30px;
    border-radius: 26px;
    display: flex;
    flex-direction: column;
    gap: 26px;
    background: var(--surface-dialog);
    border: 1px solid var(--border-strong);
    box-shadow: var(--shadow-dialog);
  }

  header {
    display: flex;
    align-items: center;
    gap: 18px;
  }

  h2 {
    font-size: 28px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }

  .installed {
    font-size: 14px;
    color: var(--text-muted);
  }

  .sections {
    flex: 1;
    min-height: 0;
    padding-right: 10px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  h3 {
    margin-bottom: 10px;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--kind);
  }

  ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  li {
    display: flex;
    gap: 12px;
    font-size: 15.5px;
    line-height: 1.55;
  }

  li::before {
    content: '•';
    color: var(--kind);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
  }
</style>
