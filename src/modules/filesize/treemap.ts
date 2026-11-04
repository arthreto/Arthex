export interface Tile<T> {
  item: T;
  x: number;
  y: number;
  width: number;
  height: number;
}

/** Squarified treemap: rows of tiles kept as close to squares as possible, largest first. */
export function squarify<T>(
  items: readonly T[],
  sizeOf: (item: T) => number,
  width: number,
  height: number,
): Tile<T>[] {
  const total = items.reduce((sum, item) => sum + sizeOf(item), 0);
  if (total <= 0 || width <= 0 || height <= 0) return [];
  const nodes = items
    .map((item) => ({ item, area: (sizeOf(item) / total) * width * height }))
    .filter((node) => node.area > 0);
  const tiles: Tile<T>[] = [];
  let x = 0;
  let y = 0;
  let w = width;
  let h = height;

  const worst = (row: typeof nodes, side: number) => {
    const sum = row.reduce((total, node) => total + node.area, 0);
    const areas = row.map((node) => node.area);
    const largest = Math.max(...areas);
    const smallest = Math.min(...areas);
    return Math.max((side * side * largest) / (sum * sum), (sum * sum) / (side * side * smallest));
  };

  const place = (row: typeof nodes) => {
    const sum = row.reduce((total, node) => total + node.area, 0);
    if (w >= h) {
      const columnWidth = sum / h;
      let top = y;
      for (const node of row) {
        const tileHeight = node.area / columnWidth;
        tiles.push({ item: node.item, x, y: top, width: columnWidth, height: tileHeight });
        top += tileHeight;
      }
      x += columnWidth;
      w -= columnWidth;
    } else {
      const rowHeight = sum / w;
      let left = x;
      for (const node of row) {
        const tileWidth = node.area / rowHeight;
        tiles.push({ item: node.item, x: left, y, width: tileWidth, height: rowHeight });
        left += tileWidth;
      }
      y += rowHeight;
      h -= rowHeight;
    }
  };

  let row: typeof nodes = [];
  for (const node of nodes) {
    const side = Math.min(w, h);
    if (row.length > 0 && worst([...row, node], side) > worst(row, side)) {
      place(row);
      row = [];
    }
    row.push(node);
  }
  if (row.length > 0) place(row);
  return tiles;
}
