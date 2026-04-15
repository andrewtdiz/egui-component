import { cn, type NodeProps, requireNodeId } from "../../component-support.ts";
import { Card } from "../card.tsx";

type ProgressProps = NodeProps & { value?: number; width?: number; height?: number } & Record<string, unknown>;

export function Progress({ value = 0, width = 188, height = 10, className, ...props }: ProgressProps) {
  const baseId = requireNodeId(props, "Progress");
  const resolvedWidth = Math.max(1, width);
  const resolvedHeight = Math.max(2, height);
  const clampedValue = Math.min(Math.max(value, 0), 1);
  const fillWidth = Math.max(clampedValue * 100, 0);

  return (
    <Card
      {...props}
      variant="plain"
      className={cn("bg-muted border-border rounded-full overflow-hidden", `w-[${resolvedWidth}px]`, `h-[${resolvedHeight}px]`, className)}
      paddingX={0}
      paddingY={0}
    >
      {clampedValue > 0 && (
        <Card
          id={`${baseId}-fill`}
          variant="plain"
          className={cn("bg-primary border-transparent rounded-full", `w-[${fillWidth}%]`, `h-[${resolvedHeight}px]`)}
          paddingX={0}
          paddingY={0}
        />
      )}
    </Card>
  );
}
