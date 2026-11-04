/**
 * Evaluates a typed calculation: + - * / % ^, parentheses, sqrt() and pi, with a comma or a dot as
 * decimal separator. Returns `null` for anything that is not clearly a calculation, so a search
 * for "2024 report" stays a search.
 */
export function evaluate(input: string): number | null {
  const raw = input.trim();
  const source = raw
    .replace(/^=\s*/, '')
    .replace(/,/g, '.')
    .replace(/×/g, '*')
    .replace(/÷/g, '/')
    .toLowerCase();
  if (!/\d|pi/.test(source) || !/^(?:[\d+\-*/%^().\s]|sqrt|pi)+$/.test(source)) return null;
  if (!/[+\-*/%^]|sqrt/.test(source) && !raw.startsWith('=')) return null;

  let index = 0;
  const peek = () => {
    while (source[index] === ' ') index += 1;
    return source[index];
  };
  const expression = (): number => {
    let value = term();
    for (;;) {
      const sign = peek();
      if (sign !== '+' && sign !== '-') return value;
      index += 1;
      const right = term();
      value = sign === '+' ? value + right : value - right;
    }
  };
  const term = (): number => {
    let value = unary();
    for (;;) {
      const sign = peek();
      if (sign !== '*' && sign !== '/' && sign !== '%') return value;
      index += 1;
      const right = unary();
      value = sign === '*' ? value * right : sign === '/' ? value / right : value % right;
    }
  };
  const unary = (): number => {
    const sign = peek();
    if (sign !== '-' && sign !== '+') return power();
    index += 1;
    return sign === '-' ? -unary() : unary();
  };
  const power = (): number => {
    const base = atom();
    if (peek() !== '^') return base;
    index += 1;
    return base ** unary();
  };
  const atom = (): number => {
    if (peek() === '(') {
      index += 1;
      const value = expression();
      if (peek() !== ')') throw new Error('missing )');
      index += 1;
      return value;
    }
    if (source.startsWith('sqrt', index)) return ((index += 4), Math.sqrt(atom()));
    if (source.startsWith('pi', index)) return ((index += 2), Math.PI);
    const number = /^(\d+\.?\d*|\.\d+)/.exec(source.slice(index));
    if (!number) throw new Error('number expected');
    index += number[0].length;
    return parseFloat(number[0]);
  };

  try {
    const value = expression();
    return peek() === undefined && Number.isFinite(value) ? value : null;
  } catch {
    return null;
  }
}
