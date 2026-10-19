/** Exercises component naming, callback folding, and cognitive complexity rules. */
import React, { forwardRef, memo, useCallback, useEffect } from "react";

/** Renders a labelled button. */
export const Button = React.forwardRef<HTMLButtonElement, { label: string }>((props, ref) => (
  <button ref={ref}>{props.label}</button>
));

export const Card = memo(({ title }: { title?: string }) => <h2>{title ?? "Untitled"}</h2>);

export function List({ items }: { items: string[] }) {
  const select = useCallback((item: string) => item.length > 2 && item !== "none", []);
  useEffect(() => {
    if (items.length) {
      console.log("ready");
    }
  }, [items]);
  return <ul>{items.map((item) => (select(item) ? <li>{item}</li> : null))}</ul>;
}

export default (value: number) => value + 1;

class Store {
  count = 0;
  increment = () => {
    this.count += 1;
  };
  reset() {
    this.count = 0;
  }
}

const handlers = {
  onSave: () => true,
};

function sumOfPrimes(max: number) {
  let total = 0;
  OUT: for (let i = 1; i <= max; ++i) {
    for (let j = 2; j < i; ++j) {
      if (i % j === 0) {
        continue OUT;
      }
    }
    total += i;
  }
  return total;
}

function getWords(count: number) {
  switch (count) {
    case 1:
      return "one";
    case 2:
      return "a couple";
    default:
      return "lots";
  }
}

function classify(a: boolean, b: boolean, c: boolean, d: boolean) {
  if (a && b && c || d) {
    return 1;
  } else if (a) {
    return 2;
  } else {
    return a ?? b;
  }
}

describe("store", () => {
  it("increments", () => {
    if (new Store().count === 0) {
      expect(handlers.onSave()).toBe(true);
    }
  });
});

register(() => 1, () => 2);

stream
  .on("data", (chunk: string) => {
    console.log(chunk);
  })
  .on("end", () => {
    console.log("end");
  });

it.each([
  { pick: (n: number) => n },
  { pick: (n: number) => -n },
])("picks %#", ({ pick }) => {
  expect(pick(1)).toBeTruthy();
});

const RowBase = ({ id }: { id: string }) => <li>{id}</li>;
export const Row = memo(RowBase, (prev, next) => prev.id === next.id);

[
  "a",
  "b",
].forEach((code) => console.log(code));
