import { cn, type NodeProps, nodeProps, requireNodeId, requiredScopedId } from "../component-support.ts";
import { Input } from "./input.tsx";
import { Label, LabelMuted } from "./primary/label.tsx";

export function FieldLabel({ id, nodeId, label = "Field", className, ...props }: NodeProps & { label?: string } & Record<string, unknown>) {
  return <Label {...props} id={requiredScopedId({ id, nodeId }, "Field", "label")} text={label} tone="secondary" weight="semibold" className={cn("text-sm font-semibold text-secondary-foreground", className)} />;
}

export function FieldControl({ id, nodeId, value = "", className, ...props }: NodeProps & { value?: string } & Record<string, unknown>) {
  return <Input {...props} id={requiredScopedId({ id, nodeId }, "Field", "control")} value={value} className={cn("w-full", className)} />;
}

export function FieldDescription({ id, nodeId, helperText, className, ...props }: NodeProps & { helperText?: string } & Record<string, unknown>) {
  if (helperText == null) return null;
  return <LabelMuted {...props} id={requiredScopedId({ id, nodeId }, "Field", "description")} text={helperText} className={cn("text-xs text-muted-foreground", className)} />;
}

export function Field({ id, nodeId, label = "Field", value = "", helperText, className, ...props }: NodeProps & { label?: string; value?: string; helperText?: string } & Record<string, unknown>) {
  const resolvedNodeId = requireNodeId({ id, nodeId }, "Field");
  return (
    <div {...nodeProps({ id: resolvedNodeId }, cn("flex flex-col gap-1.5", className))}>
      <FieldLabel id={resolvedNodeId} label={label} />
      <FieldControl {...props} id={resolvedNodeId} value={value} />
      <FieldDescription id={resolvedNodeId} helperText={helperText} />
    </div>
  );
}
