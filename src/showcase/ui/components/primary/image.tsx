import { type NodeProps, nodeProps } from "../../component-support.ts";

export function Image({ source = "builtin:showcase-image", className, ...props }: NodeProps & Record<string, unknown>) {
  return <img data-slot="image" {...nodeProps(props, className)} src={source} />;
}
