import { type NodeProps, nodeProps } from "../../component-support.ts";

export function Color({ fill = "#ffffff", className, ...props }: NodeProps & Record<string, unknown>) {
  return <span data-slot="color" {...nodeProps(props, className)} fill={fill} />;
}
