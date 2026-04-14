import { boolValue, type NodeProps, nodeProps, textFromChildren } from "../component-support.ts";

export function Checkbox({ children, checked, value, label, className, ...props }: NodeProps & { checked?: boolean; value?: boolean; label?: string } & Record<string, unknown>) {
  const resolvedValue = boolValue(value, checked);
  return <input data-slot="checkbox" type="checkbox" {...nodeProps(props, className)} checked={resolvedValue} label={label ?? textFromChildren(children)} />;
}
