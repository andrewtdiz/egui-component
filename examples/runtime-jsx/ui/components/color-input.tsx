import { cn, type Handler, type NodeProps, nodeProps, scopedId } from "../component-support.ts";
import { Color } from "./primary/color.tsx";
import { Input } from "./input.tsx";
import { LabelMuted } from "./primary/label.tsx";

export type ColorInputProps = NodeProps & { value?: string; label?: string; onChange?: Handler } & Record<string, unknown>;

export function ColorInput({ value = "#2896ff", label = "Color", className, onChange, ...props }: ColorInputProps) {
  return (
    <ColorInputRoot {...props} className={className}>
      <ColorInputSwatch id={scopedId(props, "color-input", "swatch")} fill={value} />
      <ColorInputControl id={scopedId(props, "color-input", "value")} value={value} onChange={onChange} />
      <ColorInputLabel id={scopedId(props, "color-input", "label")} text={label} />
    </ColorInputRoot>
  );
}

export function ColorInputRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-row items-center gap-2", className))}>{children}</div>;
}

export function ColorInputSwatch({ fill = "#2896ff", className, ...props }: NodeProps & Record<string, unknown>) {
  return <Color {...props} fill={fill} size={props.size ?? 22} stroke={{ width: 1, color: "#94a3b8" }} cornerRadius={props.cornerRadius ?? 6} className={className} />;
}

export function ColorInputControl({ value = "#2896ff", className, ...props }: NodeProps & Record<string, unknown>) {
  return <Input {...props} value={value} width={props.width ?? 120} className={className} />;
}

export function ColorInputLabel(props: NodeProps & { text?: string } & Record<string, unknown>) {
  return <LabelMuted {...props} />;
}
