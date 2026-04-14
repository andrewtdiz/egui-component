import { cn, type NodeProps, scopedId } from "../../component-support.ts";
import { Card } from "../card.tsx";
import { ColorStrip } from "./color-strip.tsx";

export type PalettePreviewProps = NodeProps & { colors?: string[] } & Record<string, unknown>;

export function PalettePreview({ colors = ["#111827", "#2896ff", "#10b981", "#f43f5e"], className, ...props }: PalettePreviewProps) {
  return (
    <PalettePreviewRoot {...props} className={className}>
      <PalettePreviewSwatches id={scopedId(props, "palette", "strip")} colors={colors} />
    </PalettePreviewRoot>
  );
}

export function PalettePreviewRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <Card {...props} className={cn("bg-muted border-border rounded-md", className)} paddingX={props.paddingX ?? 10} paddingY={props.paddingY ?? 10}>{children}</Card>;
}

export function PalettePreviewSwatches(props: PalettePreviewProps) {
  return <ColorStrip {...props} />;
}
