import { cn, type Handler, type NodeProps, nodeProps } from "../../component-support.ts";
import { Button } from "../button.tsx";
import { Card } from "../card.tsx";
import { Color } from "../primary/color.tsx";
import { Input } from "../input.tsx";
import { Label, LabelMuted } from "../primary/label.tsx";
import { NumberInput } from "../slider.tsx";

export function CanvaInspectorCard({ children, title = "Inspector", description, className, ...props }: NodeProps & { title?: string; description?: string } & Record<string, unknown>) {
  return <Card {...props} className={cn("bg-card border-border rounded-lg", className)}><div className="flex flex-col gap-2"><CanvaInspectorHeader title={title} description={description} />{children}</div></Card>;
}

export function CanvaInspectorHeader({ title = "Inspector", description, className, ...props }: NodeProps & { title?: string; description?: string } & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-1", className))}><Label text={title} className="text-base font-semibold" />{description != null && <LabelMuted text={description} />}</div>;
}

export function CanvaAxisField({ axis = "X", value = 0, className, onChange, ...props }: NodeProps & { axis?: string; value?: number; onChange?: Handler } & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-row items-center gap-2", className))}><Label text={axis} className="text-xs text-muted-foreground" /><NumberInput id={`${props.id ?? props.nodeId ?? "axis"}-${axis}`} value={value} width={72} suffix="px" onChange={onChange} /></div>;
}

export function CanvaColorStop({ label = "Stop", color = "#2896ff", className, onChange, ...props }: NodeProps & { label?: string; color?: string; onChange?: Handler } & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-row items-center gap-2", className))}><Color fill={color} size={22} cornerRadius={6} /><Label text={label} /><Input id={`${props.id ?? props.nodeId ?? "color-stop"}-input`} value={color} width={112} onChange={onChange} /></div>;
}

export function CanvaNinePoint({ selected = "center", className, ...props }: NodeProps & { selected?: string } & Record<string, unknown>) {
  const points = ["top-left", "top", "top-right", "left", "center", "right", "bottom-left", "bottom", "bottom-right"];
  return <div {...nodeProps(props, cn("flex flex-col gap-1", className))}>{[0, 3, 6].map((start) => <div key={String(start)} className="flex flex-row items-center gap-1">{points.slice(start, start + 3).map((point) => <Button key={point} id={`${props.id ?? props.nodeId ?? "nine-point"}-${point}`} variant={point === selected ? "primary" : "secondary"} size="sm" selected={point === selected}>{point === selected ? "*" : "."}</Button>)}</div>)}</div>;
}

export function CanvaTimelineAction({ label = "Trim", icon = "scissors", className, ...props }: NodeProps & { label?: string; icon?: string } & Record<string, unknown>) {
  return <Button {...props} className={cn(className)} variant="secondary" leadingIcon={icon}>{label}</Button>;
}
