export function classify(value: number, limit: number): string {
  if (value < 0) return "negative";
  for (let index = 0; index < limit; index += 1) {
    if (index === value) return "match";
  }
  return value > limit ? "high" : "low";
}

const render = (value: number): string => {
  return value ? `value: ${value}` : "empty";
};
