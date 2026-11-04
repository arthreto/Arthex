export interface Crumb {
  label: string;
  path: string;
}

/** The folders from `root` down to `path`, to go back up in one click. */
export function crumbs(root: string, path: string): Crumb[] {
  const list = [{ label: root, path: root }];
  if (!path.startsWith(root)) return list;
  const separator = root.includes('\\') || !root.includes('/') ? '\\' : '/';
  let current = root;
  for (const part of path.slice(root.length).split(/[\\/]/).filter(Boolean)) {
    current = current.endsWith(separator) ? current + part : current + separator + part;
    list.push({ label: part, path: current });
  }
  return list;
}
