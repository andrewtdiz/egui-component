import { type NodeProps, nodeProps, requireNodeId } from "../component-support.ts";

export function Input({ value = "", className, ...props }: NodeProps & Record<string, unknown>) {
  return <input data-slot="input" {...nodeProps({ ...props, id: requireNodeId(props, "Input") }, className)} value={value} />;
}
