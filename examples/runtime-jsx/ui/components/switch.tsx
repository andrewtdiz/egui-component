import { boolValue, type NodeProps, nodeProps, textFromChildren } from "../component-support.ts";

export function Switch({ children, checked, value, label, className, ...props }: NodeProps & { checked?: boolean; value?: boolean; label?: string } & Record<string, unknown>) {
  const resolvedValue = boolValue(value, checked);
  return <input data-slot="switch" type="checkbox" role="switch" {...nodeProps(props, className)} checked={resolvedValue} label={label ?? textFromChildren(children)} />;
}
