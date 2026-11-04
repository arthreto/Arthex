<script lang="ts">
  import type { HTMLInputAttributes } from 'svelte/elements';

  interface Props extends HTMLInputAttributes {
    label: string;
    /** Hides the label visually, for fields whose purpose is obvious (a search box). */
    hideLabel?: boolean;
    value: string;
  }

  let { label, hideLabel = false, value = $bindable(), autofocus = false, ...rest }: Props = $props();

  /** `autofocus` is ignored on elements added after the page loaded (dialogs), so focus by hand. */
  function focusOnMount(input: HTMLInputElement) {
    if (autofocus) input.focus();
  }
</script>

<label class="field">
  <span class:visually-hidden={hideLabel}>{label}</span>
  <input bind:value {...rest} use:focusOnMount />
</label>

<style>
  .field {
    display: flex;
    flex-direction: column;
    gap: 7px;
    font-size: 13px;
    color: var(--text-muted);
  }

  input {
    height: 44px;
    padding: 0 14px;
    border-radius: 10px;
    background: var(--surface-sunken);
    border: 1px solid var(--border-strong);
    font: inherit;
    font-size: 15px;
    color: var(--text);
    outline: none;
    transition: border-color var(--motion-fast);
  }

  input:focus {
    border-color: var(--accent);
  }

  input::placeholder {
    color: var(--text-subtle);
  }
</style>
