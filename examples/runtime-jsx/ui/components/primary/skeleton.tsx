import { cn, type NodeProps } from "../../component-support.ts";
import { Card } from "../card.tsx";

type SkeletonProps = NodeProps & {
  width?: number;
  height?: number;
  cornerRadius?: number;
  circle?: boolean;
  animated?: boolean;
} & Record<string, unknown>;

export function Skeleton({ width = 120, height = 16, cornerRadius, circle = false, animated = true, className, ...props }: SkeletonProps) {
  const resolvedWidth = Math.max(1, width);
  const resolvedHeight = Math.max(1, height);
  const radiusClass = circle ? "rounded-full" : cornerRadius != null && cornerRadius >= 12 ? "rounded-lg" : "rounded-md";
  return (
    <Card
      {...props}
      variant="plain"
      className={cn(
        "bg-muted border-transparent",
        animated && "opacity-80",
        radiusClass,
        `w-[${resolvedWidth}px]`,
        `h-[${resolvedHeight}px]`,
        className,
      )}
      paddingX={0}
      paddingY={0}
    />
  );
}
