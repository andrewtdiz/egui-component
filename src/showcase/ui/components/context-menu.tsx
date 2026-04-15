import { type NodeProps, nodeProps, requireNodeId } from "../component-support.ts";
import { type MenuEntry } from "./dropdown-menu.tsx";

export function ContextMenu({ children, entries = [], className, ...props }: NodeProps & { entries?: MenuEntry[] } & Record<string, unknown>) {
  return <div data-slot="context-menu" {...nodeProps({ ...props, id: requireNodeId(props, "ContextMenu") }, className)} entries={entries}>{children}</div>;
}
