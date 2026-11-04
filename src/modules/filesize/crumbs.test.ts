import { describe, expect, it } from 'vitest';
import { crumbs } from './crumbs';

describe('crumbs', () => {
  it('walks from a drive down to the folder', () => {
    expect(crumbs('C:\\', 'C:\\Users\\Léa')).toEqual([
      { label: 'C:\\', path: 'C:\\' },
      { label: 'Users', path: 'C:\\Users' },
      { label: 'Léa', path: 'C:\\Users\\Léa' },
    ]);
  });

  it('starts from a chosen folder and keeps other separators', () => {
    expect(crumbs('/home/lea', '/home/lea/Videos').map((crumb) => crumb.path)).toEqual([
      '/home/lea',
      '/home/lea/Videos',
    ]);
    expect(crumbs('E:\\Photos', 'D:\\Other')).toEqual([{ label: 'E:\\Photos', path: 'E:\\Photos' }]);
  });
});
