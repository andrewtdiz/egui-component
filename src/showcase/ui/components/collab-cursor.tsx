import { cn, type NodeProps, nodeProps } from "../component-support.ts";
import { Card } from "./card.tsx";
import { Color } from "./primary/color.tsx";
import { Label } from "./primary/label.tsx";

export function CollabCursor({ name = "Taylor", color = "#ff7a24", className, ...props }: NodeProps & { color?: string } & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-row items-center gap-1", className))}><Color fill={color} size={12} cornerRadius={6} /><Card className="bg-primary border-primary rounded-md" paddingX={8} paddingY={4}><Label text={name} className="text-xs font-semibold text-primary-foreground" /></Card></div>;
}
