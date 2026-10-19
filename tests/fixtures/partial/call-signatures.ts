type Store = {
  (): number
  <U>(selector: U): U
};

/** Creates an empty store. */
export function create(): Store | null {
  return null;
}
