/** Mirrors `src-tauri/src/modules/shortcuts/site.rs`, to explain a refusal before asking Rust. */
export type SiteProblem = 'missing' | 'scheme' | 'invalid';

const SCHEME = /^[a-z][a-z0-9+.-]*:/i;

export function siteProblem(input: string): SiteProblem | null {
  const trimmed = input.trim();
  if (!trimmed) return 'missing';
  if (SCHEME.test(trimmed) && !/^https?:\/\//i.test(trimmed)) return 'scheme';
  try {
    const host = new URL(SCHEME.test(trimmed) ? trimmed : `https://${trimmed}`).hostname;
    return host.includes('.') ? null : 'invalid';
  } catch {
    return 'invalid';
  }
}

/** The name Rust gives a site when none is typed: `www.github.com` gives `Github`. */
export function guessName(input: string): string {
  const label = input
    .trim()
    .replace(/^https?:\/\//i, '')
    .replace(/^www\./i, '')
    .split(/[/.:?#]/)[0];
  return label ? label.charAt(0).toUpperCase() + label.slice(1) : '';
}

/** `https://github.com/` reads better as `github.com`; Rust adds the https:// back when saving. */
export function shortAddress(url: string): string {
  return url.replace(/^https:\/\//i, '').replace(/\/$/, '');
}
