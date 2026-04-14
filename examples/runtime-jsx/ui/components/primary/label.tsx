import { labelVariants } from "../../lib/styles.ts";
import { cn, type NodeProps, nodeProps, textFromChildren } from "../../component-support.ts";

export type LabelProps = NodeProps & { text?: string; tone?: string; weight?: string; size?: string | number } & Record<string, unknown>;

export function Label({ children, text, className, tone, weight, size, ...props }: LabelProps) {
  const sizeClass = typeof size === "string" ? size : undefined;
  const resolvedSize = typeof size === "number" ? size : undefined;
  const resolvedWeight = weight === "medium" ? undefined : weight;
  return <label data-slot="label" {...nodeProps(props, labelVariants({ tone: tone ?? "primary", weight: weight ?? "medium", size: sizeClass, className }))} text={text ?? textFromChildren(children)} tone={tone} weight={resolvedWeight} size={resolvedSize}>{children}</label>;
}

export function LabelMuted({ children, text, className, ...props }: LabelProps) {
  return <Label {...props} text={text ?? textFromChildren(children)} tone="muted" weight="regular" className={className}>{children}</Label>;
}

export const MutedLabel = LabelMuted;

export function LabelTitle({ children, text, className, ...props }: LabelProps) {
  return <Label {...props} text={text ?? textFromChildren(children)} tone="primary" weight="semibold" size="lg" className={cn("text-lg font-semibold", className)}>{children}</Label>;
}

export function LabelDescription({ children, text, className, ...props }: LabelProps) {
  return <Label {...props} text={text ?? textFromChildren(children)} tone="muted" weight="regular" size="sm" className={cn("text-sm text-muted-foreground", className)}>{children}</Label>;
}
