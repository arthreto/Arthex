<script lang="ts" generics="Value extends string">
  interface Option {
    value: Value;
    label: string;
    disabled?: boolean;
  }

  interface Props {
    options: readonly Option[];
    value: Value;
    label: string;
    onchange: (value: Value) => void;
  }

  let { options, value, label, onchange }: Props = $props();
</script>

<div class="segmented" role="radiogroup" aria-label={label}>
  {#each options as option (option.value)}
    <button
      type="button"
      role="radio"
      aria-checked={option.value === value}
      class:on={option.value === value}
      disabled={option.disabled}
      onclick={() => onchange(option.value)}>{option.label}</button
    >
  {/each}
</div>

<style>
  .segmented {
    display: inline-flex;
    align-self: flex-start;
    gap: 2px;
    padding: 3px;
    border-radius: 9px;
    background: var(--surface-sunken);
  }

  button {
    height: 30px;
    padding: 0 14px;
    border-radius: 7px;
    font-size: 12.5px;
    color: var(--text-muted);
    white-space: nowrap;
  }

  button:hover:enabled {
    color: var(--text);
  }

  button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .on {
    background: var(--surface-hover);
    color: var(--text);
  }
</style>
