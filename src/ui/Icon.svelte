<script lang="ts">
  import { icons, type IconName } from './icons';

  interface Props {
    /** A shell icon name, or raw path data (module icons). */
    name?: IconName;
    path?: string;
    size?: number;
    stroke?: number;
    filled?: boolean;
  }

  let { name, path, size = 18, stroke = 1.8, filled = false }: Props = $props();
  const d = $derived(path ?? (name ? icons[name] : ''));
</script>

<svg
  aria-hidden="true"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill={filled ? 'currentColor' : 'none'}
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"><path {d} /></svg
>
