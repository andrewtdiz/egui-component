import { type NodeProps, nodeProps } from "../component-support.ts";

export function ContextMenu({ children, entries = [], className, ...props }: NodeProps & { entries?: unknown[] } & Record<string, unknown>) {
  return <div data-slot="context-menu" {...nodeProps(props, className)} entries={entries}>{children}</div>;
}
