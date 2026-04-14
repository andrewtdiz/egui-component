import { useState } from "egui";
import { styles } from "../lib/styles.ts";
import { cn, type NodeProps, nodeProps, requireNodeId } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";

export function CollapsibleRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <Card {...nodeProps(props, cn(styles.surfaceMuted, className))} paddingX={12} paddingY={12}>{children}</Card>;
}

export function CollapsibleTrigger({ title = "Section", open = true, leadingIcon, trailingIcon, className, ...props }: NodeProps & { title?: string; open?: boolean; leadingIcon?: string; trailingIcon?: string } & Record<string, unknown>) {
  const icon = leadingIcon ?? (open ? "chevron-down" : "chevron-right");
  return (
    <Button
      {...props}
      variant="ghost"
      className={cn("w-full px-0", className)}
      selected={open}
      leadingIcon={icon}
      trailingIcon={trailingIcon}
      label={title}
    />
  );
}

export function CollapsibleContent({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-2 pt-3", className))}>{children}</div>;
}

export function Collapsible({
  children,
  title = "Section",
  open,
  className,
  leadingIcon,
  trailingIcon,
  onToggle,
  ...props
}: NodeProps & { title?: string; open?: boolean; leadingIcon?: string; trailingIcon?: string } & Record<string, unknown>) {
  const [internalOpen, setInternalOpen] = useState(open ?? true);
  const resolvedOpen = open ?? internalOpen;
  const baseId = requireNodeId(props, "Collapsible");

  const setOpen = (next: boolean, event?: unknown) => {
    if (open == null) {
      setInternalOpen(next);
    }
    const base = typeof event === "object" && event != null ? event as Record<string, unknown> : {};
    const metadata = typeof base.metadata === "object" && base.metadata != null ? base.metadata as Record<string, unknown> : {};
    onToggle?.({ ...base, value: next, metadata: { ...metadata, open: next } }, next);
  };

  return (
    <CollapsibleRoot {...props} className={className}>
      <CollapsibleTrigger
        id={`${baseId}-trigger`}
        title={title}
        open={resolvedOpen}
        leadingIcon={leadingIcon}
        trailingIcon={trailingIcon}
        onClick={(event) => setOpen(!resolvedOpen, event)}
      />
      {resolvedOpen && <CollapsibleContent id={`${baseId}-content`}>{children}</CollapsibleContent>}
    </CollapsibleRoot>
  );
}
