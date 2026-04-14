import { cn, type NodeProps } from "../../component-support.ts";
import { Icon } from "./icon.tsx";

type SpinnerProps = NodeProps & { size?: number; name?: string } & Record<string, unknown>;

export function Spinner({ size = 16, name = "loader-circle", className, ...props }: SpinnerProps) {
  return <Icon {...props} name={name} size={size} className={cn("text-primary", className)} />;
}
