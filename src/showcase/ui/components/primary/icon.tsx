import { cn, type NodeProps, nodeProps } from "../../component-support.ts";

export function Icon({ name = "sparkles", className, ...props }: NodeProps & Record<string, unknown>) {
  return <span data-slot="icon" aria-hidden="true" {...nodeProps(props, cn("text-muted-foreground", className))} name={name} />;
}
