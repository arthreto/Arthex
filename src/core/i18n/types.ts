export interface MessageTree {
  readonly [key: string]: string | MessageTree;
}

export type MessageValues = Record<string, string | number | Date>;

type Join<Head extends string, Tail extends string> = `${Head}.${Tail}`;

/** Dotted keys of a message tree, e.g. `settings.general.theme`. Keys starting with `$` are metadata. */
export type MessageKey<T> = {
  [K in Exclude<keyof T & string, `$${string}`>]: T[K] extends string
    ? K
    : T[K] extends MessageTree
      ? Join<K, MessageKey<T[K]>>
      : never;
}[Exclude<keyof T & string, `$${string}`>];

export interface LocaleMeta {
  code: string;
  name: string;
  nativeName: string;
}
