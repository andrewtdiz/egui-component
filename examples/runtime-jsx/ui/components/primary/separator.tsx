import { cn, type NodeProps } from "../../component-support.ts";
import { Card } from "../card.tsx";

export function Separator({ className, ...props }: NodeProps & Record<string, unknown>) {
  return <Card {...props} variant="plain" className={cn("bg-border border-transparent rounded-none w-full h-[1px]", className)} paddingX={0} paddingY={0} />;
}
