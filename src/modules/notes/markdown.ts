/** A run of text with its inline style. Rendered as text nodes, never as HTML. */
export interface Span {
  text: string;
  bold?: boolean;
  italic?: boolean;
  strike?: boolean;
  code?: boolean;
}

export type Block =
  | { kind: 'h1' | 'h2' | 'h3' | 'p' | 'li'; spans: Span[]; line: number }
  | { kind: 'ol'; number: string; spans: Span[]; line: number }
  | { kind: 'task'; done: boolean; spans: Span[]; line: number }
  | { kind: 'gap'; line: number };

const INLINE = /(\*\*[^*]+\*\*|__[^_]+__|~~[^~]+~~|`[^`]+`|\*[^*\s][^*]*\*|(?<!\w)_[^_\s][^_]*_(?!\w))/g;
const TASK = /^\s*[-*]\s+\[( |x|X)\]\s*(.*)$/;

export function inline(source: string): Span[] {
  const spans: Span[] = [];
  let last = 0;
  for (const match of source.matchAll(INLINE)) {
    const token = match[0];
    if (match.index > last) spans.push({ text: source.slice(last, match.index) });
    if (token.startsWith('**') || token.startsWith('__'))
      spans.push({ text: token.slice(2, -2), bold: true });
    else if (token.startsWith('~~')) spans.push({ text: token.slice(2, -2), strike: true });
    else if (token.startsWith('`')) spans.push({ text: token.slice(1, -1), code: true });
    else spans.push({ text: token.slice(1, -1), italic: true });
    last = match.index + token.length;
  }
  if (last < source.length) spans.push({ text: source.slice(last) });
  return spans;
}

/** Headings `#` to `###`, bullet and numbered lists, `- [ ]` tasks and paragraphs, line by line. */
export function parse(text: string): Block[] {
  const blocks: Block[] = [];
  text.split('\n').forEach((raw, line) => {
    const content = raw.trimEnd();
    let match: RegExpExecArray | null;
    if (!content.trim()) {
      if (blocks.length > 0 && blocks.at(-1)?.kind !== 'gap') blocks.push({ kind: 'gap', line });
    } else if ((match = /^(#{1,3})\s+(.*)$/.exec(content))) {
      const level = match[1]?.length ?? 1;
      blocks.push({
        kind: level === 1 ? 'h1' : level === 2 ? 'h2' : 'h3',
        spans: inline(match[2] ?? ''),
        line,
      });
    } else if ((match = TASK.exec(content))) {
      blocks.push({ kind: 'task', done: match[1] !== ' ', spans: inline(match[2] ?? ''), line });
    } else if ((match = /^\s*[-*]\s+(.*)$/.exec(content))) {
      blocks.push({ kind: 'li', spans: inline(match[1] ?? ''), line });
    } else if ((match = /^\s*(\d+)[.)]\s+(.*)$/.exec(content))) {
      blocks.push({ kind: 'ol', number: match[1] ?? '1', spans: inline(match[2] ?? ''), line });
    } else {
      blocks.push({ kind: 'p', spans: inline(content), line });
    }
  });
  return blocks;
}

/** A line without its Markdown marks, for previews. */
export function plain(line: string): string {
  return line
    .replace(/^#{1,3}\s+/, '')
    .replace(/^\s*[-*]\s+(\[[ xX]\]\s*)?/, '')
    .replace(/^\s*\d+[.)]\s+/, '')
    .replace(/\*\*|__|~~|`/g, '')
    .replace(/(^|\W)[*_]([^*_]+)[*_](?=\W|$)/g, '$1$2');
}

export function tasks(text: string): { text: string; done: boolean }[] {
  return text.split('\n').flatMap((line) => {
    const match = TASK.exec(line);
    return match ? [{ text: plain(match[2] ?? ''), done: match[1] !== ' ' }] : [];
  });
}

/** Ticks or unticks the task on a line of the source. */
export function toggleTask(text: string, line: number): string {
  const lines = text.split('\n');
  const current = lines[line];
  if (current === undefined) return text;
  lines[line] = current.replace(/\[( |x|X)\]/, (_, mark: string) => (mark === ' ' ? '[x]' : '[ ]'));
  return lines.join('\n');
}

/** The result of an edit: the new text and the selection to restore. */
export interface Edit {
  text: string;
  start: number;
  end: number;
}

/** Surrounds the selection with a mark (`**`), or a placeholder when nothing is selected. */
export function wrap(text: string, start: number, end: number, mark: string, placeholder: string): Edit {
  const selected = text.slice(start, end) || placeholder;
  return {
    text: text.slice(0, start) + mark + selected + mark + text.slice(end),
    start: start + mark.length,
    end: start + mark.length + selected.length,
  };
}

const PREFIX = /^(#{1,3}\s+|\s*[-*]\s+\[[ xX]\]\s+|\s*[-*]\s+)/;

/** Gives the line under the caret a prefix (`## `, `- `, `- [ ] `), replaces another one, or
 * removes it when it is already there. */
export function togglePrefix(text: string, start: number, end: number, prefix: string): Edit {
  const lineStart = text.lastIndexOf('\n', start - 1) + 1;
  const found = text.indexOf('\n', start);
  const lineEnd = found < 0 ? text.length : found;
  const line = text.slice(lineStart, lineEnd);
  const current = PREFIX.exec(line)?.[0];
  const next =
    current === prefix ? line.slice(prefix.length) : prefix + (current ? line.slice(current.length) : line);
  const delta = next.length - line.length;
  return {
    text: text.slice(0, lineStart) + next + text.slice(lineEnd),
    start: Math.max(lineStart, start + delta),
    end: Math.max(lineStart, end + delta),
  };
}

/** Enter in a list: continues it with a new item, or ends it when the item is empty. `null`
 * when the caret is not in a list. */
export function continueList(text: string, caret: number): Edit | null {
  const lineStart = text.lastIndexOf('\n', caret - 1) + 1;
  const line = text.slice(lineStart, caret);
  const match = /^(\s*)([-*]\s+\[[ xX]\]\s+|[-*]\s+|(\d+)[.)]\s+)/.exec(line);
  if (!match) return null;
  if (!line.slice(match[0].length).trim()) {
    return { text: text.slice(0, lineStart) + text.slice(caret), start: lineStart, end: lineStart };
  }
  const marker = match[3] ? `${Number(match[3]) + 1}. ` : (match[2] ?? '').replace(/\[[xX]\]/, '[ ]');
  const insert = `\n${match[1] ?? ''}${marker}`;
  const position = caret + insert.length;
  return { text: text.slice(0, caret) + insert + text.slice(caret), start: position, end: position };
}
