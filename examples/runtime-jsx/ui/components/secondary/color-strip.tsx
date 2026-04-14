import { cn, type NodeProps, nodeProps } from "../../component-support.ts";
import { Color } from "../primary/color.tsx";

export type ColorStripProps = NodeProps & { colors?: string[] } & Record<string, unknown>;

export function ColorStrip({ colors = ["#2896ff", "#10b981", "#f43f5e"], className, ...props }: ColorStripProps) {
  const baseId = props.id ?? props.nodeId ?? "color-strip";

  return (
    <ColorStripRoot {...props} className={className}>
      {colors.map((fill, index) => <ColorStripSwatch key={`${fill}-${index}`} id={`${baseId}-${index}`} fill={fill} />)}
    </ColorStripRoot>
  );
}

export function ColorStripRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-row items-center gap-1.5", className))}>{children}</div>;
}

export function ColorStripSwatch({ fill = "#2896ff", className, ...props }: NodeProps & Record<string, unknown>) {
  return <Color {...props} fill={fill} size={props.size ?? 24} stroke={{ width: 1, color: "#94a3b8" }} cornerRadius={props.cornerRadius ?? 6} className={className} />;
}
