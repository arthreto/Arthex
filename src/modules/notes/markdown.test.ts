import { describe, expect, it } from 'vitest';
import { continueList, inline, parse, plain, tasks, toggleTask, togglePrefix, wrap } from './markdown';

describe('parse', () => {
  it('reads headings, lists, tasks and paragraphs, with one gap between groups', () => {
    const blocks = parse('# Week\n\n\n- [x] bread\n- milk\n2. call\ntext');
    expect(blocks.map((block) => block.kind)).toEqual(['h1', 'gap', 'task', 'li', 'ol', 'p']);
    expect(blocks[2]).toMatchObject({ kind: 'task', done: true, line: 3 });
  });

  it('styles inline marks and keeps HTML as plain text', () => {
    expect(inline('**bold** and *it* `x` ~~old~~ <b>')).toEqual([
      { text: 'bold', bold: true },
      { text: ' and ' },
      { text: 'it', italic: true },
      { text: ' ' },
      { text: 'x', code: true },
      { text: ' ' },
      { text: 'old', strike: true },
      { text: ' <b>' },
    ]);
  });
});

describe('tasks', () => {
  it('lists and toggles tasks', () => {
    const text = '- [ ] **bread**\nnote\n- [x] milk';
    expect(tasks(text)).toEqual([
      { text: 'bread', done: false },
      { text: 'milk', done: true },
    ]);
    expect(toggleTask(text, 0)).toBe('- [x] **bread**\nnote\n- [x] milk');
    expect(toggleTask(text, 2)).toBe('- [ ] **bread**\nnote\n- [ ] milk');
    expect(plain('## A *b* title')).toBe('A b title');
  });
});

describe('editing', () => {
  it('wraps the selection or a placeholder', () => {
    expect(wrap('say hi', 4, 6, '**', 'bold')).toEqual({ text: 'say **hi**', start: 6, end: 8 });
    expect(wrap('', 0, 0, '*', 'text').text).toBe('*text*');
  });

  it('adds, swaps and removes a line prefix', () => {
    expect(togglePrefix('a\nitem', 3, 3, '- ').text).toBe('a\n- item');
    expect(togglePrefix('- item', 2, 2, '- [ ] ').text).toBe('- [ ] item');
    expect(togglePrefix('## Title', 4, 4, '## ').text).toBe('Title');
  });

  it('continues a list on Enter and ends it on an empty item', () => {
    expect(continueList('- [x] done', 10)).toEqual({ text: '- [x] done\n- [ ] ', start: 17, end: 17 });
    expect(continueList('1. one', 6)?.text).toBe('1. one\n2. ');
    expect(continueList('- one\n- ', 8)).toEqual({ text: '- one\n', start: 6, end: 6 });
    expect(continueList('plain', 5)).toBeNull();
  });
});
