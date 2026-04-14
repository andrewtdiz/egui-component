import { cn, type NodeProps, textFromChildren } from "../../component-support.ts";
import { Card } from "../card.tsx";
import { Label } from "./label.tsx";

export function Kbd({ children, text, className, ...props }: NodeProps & { text?: string } & Record<string, unknown>) {
  const resolvedText = text ?? textFromChildren(children) ?? "Ctrl";
  return (
    <Card {...props} variant="plain" className={cn("bg-muted border-border rounded-md", className)} paddingX={6} paddingY={2}>
      <Label text={resolvedText} className="text-xs font-medium text-muted-foreground" />
    </Card>
  );
}
