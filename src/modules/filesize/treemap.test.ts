import { describe, expect, it } from 'vitest';
import { squarify } from './treemap';

describe('squarify', () => {
  const sizes = [600, 300, 60, 30, 10];
  const tiles = squarify(sizes, (size) => size, 400, 250);

  it('gives each item an area in proportion to its size', () => {
    for (const tile of tiles) {
      expect(tile.width * tile.height).toBeCloseTo((tile.item / 1000) * 400 * 250, 6);
    }
  });

  it('fills the space without leaving it', () => {
    expect(tiles).toHaveLength(sizes.length);
    for (const tile of tiles) {
      expect(tile.x + tile.width).toBeLessThanOrEqual(400 + 1e-6);
      expect(tile.y + tile.height).toBeLessThanOrEqual(250 + 1e-6);
    }
  });

  it('skips empty input and empty items', () => {
    expect(squarify([], (size: number) => size, 400, 250)).toEqual([]);
    expect(squarify([5, 0], (size) => size, 100, 100)).toHaveLength(1);
  });
});
