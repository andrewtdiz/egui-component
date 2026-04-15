import { type NodeProps, nodeProps, requireNodeId } from "../component-support.ts";

export function ContextMenu({ children, entries = [], className, ...props }: NodeProps & { entries?: unknown[] } & Record<string, unknown>) {
  return <div data-slot="context-menu" {...nodeProps({ ...props, id: requireNodeId(props, "ContextMenu") }, className)} entries={entries}>{children}</div>;
}
