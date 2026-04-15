import { type NodeProps, nodeProps } from "../component-support.ts";

export function Tooltip({ triggerLabel = "Hover", text = "Tooltip content", className, ...props }: NodeProps & { triggerLabel?: string; text?: string } & Record<string, unknown>) {
  return <span data-slot="tooltip" title={text} {...nodeProps(props, className)} triggerLabel={triggerLabel} text={text} />;
}
