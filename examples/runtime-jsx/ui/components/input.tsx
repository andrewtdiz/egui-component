import { type NodeProps, nodeProps } from "../component-support.ts";

export function Input({ value = "", className, ...props }: NodeProps & Record<string, unknown>) {
  return <input data-slot="input" {...nodeProps(props, className)} value={value} />;
}
