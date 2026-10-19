/**
 * @license MIT
 * Copyright Example Ltd.
 *
 * Use of this source code is governed by an MIT-style license.
 */
/** Declares functions with docs, headers and exports. */
import { helper } from "./helper";

/** Adds two numbers. */
export function add(a: number, b: number): number {
  return a + b;
}

/** @param value the value */
function tagged(value: number) {
  return value;
}

// A plain comment, not a doc.
const local = () => 1;

export class Counter {
  /** Increments the count. */
  increment = () => {
    this.count += 1;
  };
  count = 0;

  /** Resets the count. */
  @logged
  reset() {
    this.count = 0;
  }
}

export const handlers = {
  /** Handles a click. */
  click: () => helper(),
};

export { local };
export default tagged;
export { other } from "./other";
function other() {}

/** Renders the copyright footer. */
export function footer() {}

/**
 * @deprecated
 *
 * Use add instead.
 */
export function legacy() {}

/** A banner that documents nothing below it. */
// ----------
/** @internal */
export function internal() {}

export class Store {
  /** Handles a click. */
  @action
  handle = () => 1;
  onKey = debounce((event) => event);
}

export const routes = { home: () => "/", about: () => "/about" };

export const loaders = {
  /** Loads the user. */
  user:
    async (id) => id,
};

function wrapper() {
  function local() {}
  return local;
}
