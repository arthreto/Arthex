export type NoteKind = 'new' | 'improved' | 'fixed';

export interface NoteSection {
  kind: NoteKind;
  items: string[];
}

const KINDS: Record<string, NoteKind> = {
  new: 'new',
  nouveau: 'new',
  improved: 'improved',
  amélioré: 'improved',
  fixed: 'fixed',
  corrigé: 'fixed',
};

/** The sections of one language from a release description (format in docs/DEVELOPMENT.md →
 * Releasing). Falls back to English when the language has no section. */
export function releaseNotes(notes: string, french: boolean): NoteSection[] {
  const byLanguage: Record<'en' | 'fr', NoteSection[]> = { en: [], fr: [] };
  let language: 'en' | 'fr' | null = null;
  let section: NoteSection | null = null;
  for (const raw of notes.split('\n')) {
    const line = raw.trim();
    const heading = /^(#{3,4})\s+(.+)$/.exec(line);
    if (heading?.[1] === '###') {
      const name = heading[2]?.toLowerCase() ?? '';
      language = name.startsWith('fr') ? 'fr' : name.startsWith('en') ? 'en' : null;
      section = null;
    } else if (heading && language) {
      const kind = KINDS[heading[2]?.toLowerCase() ?? ''];
      section = kind ? { kind, items: [] } : null;
      if (section) byLanguage[language].push(section);
    } else if (section && /^[-*]\s+/.test(line)) {
      section.items.push(line.replace(/^[-*]\s+/, ''));
    }
  }
  const chosen = french && byLanguage.fr.length > 0 ? byLanguage.fr : byLanguage.en;
  return chosen.filter((entry) => entry.items.length > 0);
}
