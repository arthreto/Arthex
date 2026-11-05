const LIMIT = 20;

/** The calculations of this session, newest first. Kept in memory only. */
export const history = $state<{ expression: string; result: string }[]>([]);

export function remember(expression: string, result: string): void {
  history.unshift({ expression, result });
  history.splice(LIMIT);
}
