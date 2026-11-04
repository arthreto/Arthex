<script lang="ts">
  interface Props {
    checked: boolean;
    label: string;
    disabled?: boolean;
    onchange: (checked: boolean) => void;
  }

  let { checked, label, disabled = false, onchange }: Props = $props();
</script>

<button
  type="button"
  role="switch"
  class="switch"
  class:on={checked}
  aria-checked={checked}
  aria-label={label}
  {disabled}
  onclick={() => onchange(!checked)}
></button>

<style>
  .switch {
    position: relative;
    flex: none;
    width: 40px;
    height: 22px;
    border-radius: 11px;
    background: var(--surface-hover);
    border: 1px solid var(--border-strong);
    transition: background-color var(--motion-base);
  }

  .switch::after {
    content: '';
    position: absolute;
    top: 3px;
    left: 3px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--text-muted);
    transition:
      transform var(--motion-base),
      background-color var(--motion-base);
  }

  .on {
    background: var(--accent);
    border-color: var(--accent);
  }

  .on::after {
    transform: translateX(18px);
    background: var(--on-accent);
  }

  .switch:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
</style>
