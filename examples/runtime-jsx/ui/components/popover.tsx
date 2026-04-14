import { useState } from "egui";
import { cn, type NodeProps, nodeProps } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";

export type PopoverProps = NodeProps & { open?: boolean; triggerLabel?: string; width?: number; side?: string; align?: string } & Record<string, unknown>;

export function PopoverRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-2", className))}>{children}</div>;
}

export function PopoverTrigger({ children, triggerLabel = "Open", className, ...props }: NodeProps & { triggerLabel?: string } & Record<string, unknown>) {
  return <Button {...props} variant="secondary" className={className}>{children ?? triggerLabel}</Button>;
}

export function Popover({ children, open, triggerLabel = "Open", width, className, onOpen, onClose, ...props }: PopoverProps) {
  const baseId = props.id ?? props.nodeId ?? "popover";
  const [internalOpen, setInternalOpen] = useState(false);
  const resolvedOpen = open ?? internalOpen;
  const setOpen = (next: boolean, event?: unknown) => {
    if (open == null) {
      setInternalOpen(next);
    }
    if (next) onOpen?.(event ?? null, next);
    else onClose?.(event ?? null, next);
  };
  return (
    <PopoverRoot {...props} className={className}>
      <PopoverTrigger id={`${baseId}-trigger`} triggerLabel={triggerLabel} onClick={(event) => setOpen(!resolvedOpen, event)} />
      {resolvedOpen && <PopoverContent id={`${baseId}-content`} width={width}>{children}</PopoverContent>}
    </PopoverRoot>
  );
}

export function PopoverContent({ children, id, width, className, ...props }: NodeProps & { width?: number } & Record<string, unknown>) {
  return <Card {...nodeProps({ ...props, id }, cn("bg-popover border border-border rounded-lg shadow-md", className))} width={width} paddingX={12} paddingY={12}>{children}</Card>;
}
