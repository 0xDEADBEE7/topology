/** Renders a conditional greeting. */
import type { ReactNode } from 'react';

/** Chooses the greeting content. */
export const Greeting = ({ name }: { name?: string }): ReactNode => {
  const label = name ?? 'world';
  return <section>{name && <strong>{label}</strong>}</section>;
};

/** Displays a greeting or a placeholder. */
export default function App({ ready }: { ready: boolean }) {
  return ready ? <Greeting name="Ada" /> : <></>;
}
