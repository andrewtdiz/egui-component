import { cn, type Handler, type NodeProps, nodeProps, scopedId } from "../component-support.ts";
import { Card } from "./card.tsx";
import { Color } from "./primary/color.tsx";
import { ColorInput } from "./color-input.tsx";
import { Label, LabelMuted } from "./primary/label.tsx";

const defaultPalette = ["#111827", "#2896ff", "#10b981", "#f43f5e", "#f59e0b", "#8b5cf6", "#06b6d4", "#84cc16"];
const DEFAULT_SWATCH_STROKE = { width: 1, color: "#94a3b8" };
const SELECTED_SWATCH_STROKE = { width: 2, color: "#2896ff" };

export type PaletteColorInputProps = NodeProps & { value?: string; palette?: string[]; allowCustom?: boolean; customLabel?: string; onChange?: Handler } & Record<string, unknown>;

export function PaletteColorInput({ value = "#2896ff", palette = defaultPalette, allowCustom = true, customLabel = "Custom", className, onChange, ...props }: PaletteColorInputProps) {
  const baseId = props.id ?? props.nodeId ?? "palette-color";

  return (
    <PaletteColorInputRoot {...props} className={className}>
      <PaletteColorInputHeader />
      <PaletteColorInputSwatches id={`${baseId}-swatches`} value={value} palette={palette} />
      {allowCustom && <PaletteColorInputCustom id={scopedId(props, "palette-color", "custom")} label={customLabel} value={value} onChange={onChange} />}
    </PaletteColorInputRoot>
  );
}

export function PaletteColorInputRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <Card {...props} className={cn("bg-muted border-border rounded-md", className)} paddingX={props.paddingX ?? 10} paddingY={props.paddingY ?? 10}><div className="flex flex-col gap-2">{children}</div></Card>;
}

export function PaletteColorInputHeader({ text = "Palette color", className, ...props }: NodeProps & { text?: string } & Record<string, unknown>) {
  return <Label {...props} text={text} className={cn("text-sm font-medium", className)} />;
}

export function PaletteColorInputSwatches({ value = "#2896ff", palette = defaultPalette, className, ...props }: NodeProps & { value?: string; palette?: string[] } & Record<string, unknown>) {
  const baseId = props.id ?? props.nodeId ?? "palette-color-swatches";

  return (
    <div {...nodeProps(props, cn("flex flex-row flex-wrap items-center gap-1.5", className))}>
      {palette.map((fill, index) => <PaletteColorInputSwatch key={`${fill}-${index}`} id={`${baseId}-${index}`} fill={fill} selected={fill === value} />)}
    </div>
  );
}

export function PaletteColorInputSwatch({ fill = "#2896ff", selected = false, className, ...props }: NodeProps & { selected?: boolean } & Record<string, unknown>) {
  return <Color {...props} fill={fill} size={props.size ?? 22} stroke={selected ? SELECTED_SWATCH_STROKE : DEFAULT_SWATCH_STROKE} cornerRadius={props.cornerRadius ?? 6} className={className} />;
}

export function PaletteColorInputCustomLabel({ text = "Custom", className, ...props }: NodeProps & { text?: string } & Record<string, unknown>) {
  return <LabelMuted {...props} text={text} className={cn("text-xs font-semibold", className)} />;
}

export function PaletteColorInputCustom({ value = "#2896ff", label = "Custom", onChange, className, ...props }: NodeProps & { value?: string; label?: string; onChange?: Handler } & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-1", className))}><PaletteColorInputCustomLabel text={label} /><ColorInput id={scopedId(props, "palette-color", "input")} value={value} onChange={onChange} /></div>;
}
