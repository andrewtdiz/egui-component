import { useState } from "egui";
import { styles } from "../lib/styles.ts";
import { cn, itemId, itemLabel, type Handler, type Item, type NodeProps, nodeProps } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";
import { LabelMuted } from "./primary/label.tsx";
import { Separator } from "./primary/separator.tsx";

export type MenuEntry = Item | string;

export type DropdownMenuProps = NodeProps & {
  entries?: MenuEntry[];
  options?: string[];
  triggerLabel?: string;
  triggerVariant?: string;
  open?: boolean;
  width?: number;
} & Record<string, unknown>;

export function DropdownMenuRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-1.5", className))}>{children}</div>;
}

export function DropdownMenuTrigger({ children, className, triggerLabel = "Open", triggerVariant = "secondary", ...props }: NodeProps & { triggerLabel?: string; triggerVariant?: string } & Record<string, unknown>) {
  return (
    <Button
      {...props}
      variant={triggerVariant}
      trailingIcon="chevron-down"
      className={cn(styles.actionLabel, className)}
    >
      {children ?? triggerLabel}
    </Button>
  );
}

export function DropdownMenuSurface({ children, className, width = 220, ...props }: NodeProps & { width?: number } & Record<string, unknown>) {
  return (
    <Card
      {...nodeProps(props, cn("bg-popover border border-border rounded-md shadow-md", className))}
      width={width}
      paddingX={4}
      paddingY={4}
    >
      {children}
    </Card>
  );
}

export function DropdownMenu({
  entries = [],
  options = [],
  triggerLabel = "Open",
  triggerVariant = "secondary",
  open,
  width = 220,
  className,
  onCommand,
  onOpen,
  onClose,
  ...props
}: DropdownMenuProps) {
  const resolvedEntries = entries.length > 0 ? entries : options.map((label, index) => ({ itemId: String(index), label }));
  const baseId = props.id ?? props.nodeId ?? "dropdown-menu";
  const [internalOpen, setInternalOpen] = useState(false);
  const resolvedOpen = open ?? internalOpen;
  const setOpen = (next: boolean, event?: unknown) => {
    if (open == null) {
      setInternalOpen(next);
    }
    if (next) onOpen?.(event ?? null, resolvedOpen);
    else onClose?.(event ?? null, resolvedOpen);
  };

  return (
    <DropdownMenuRoot {...props} className={className}>
      <DropdownMenuTrigger
        id={`${baseId}-trigger`}
        triggerVariant={triggerVariant}
        onClick={(event) => setOpen(!resolvedOpen, event)}
      >
        {triggerLabel}
      </DropdownMenuTrigger>
      {resolvedOpen && (
        <DropdownMenuSurface id={`${baseId}-content`} width={width}>
          <DropdownMenuContent
            id={`${baseId}-items`}
            entries={resolvedEntries}
            onCommand={onCommand}
            onClose={() => setOpen(false)}
          />
        </DropdownMenuSurface>
      )}
    </DropdownMenuRoot>
  );
}

export function DropdownMenuContent({
  id = "dropdown-menu-content",
  entries = [],
  onCommand,
  onClose,
  className,
  children,
}: NodeProps & {
  entries?: MenuEntry[];
  onCommand?: Handler;
  onClose?: () => void;
} & Record<string, unknown>) {
  return (
    <div id={id} className={cn("flex flex-col gap-0.5", className)}>
      {children ?? entries.map((entry, index) => (
        <DropdownMenuEntry key={String(index)} baseId={id} entry={entry} index={index} onCommand={onCommand} onClose={onClose} />
      ))}
    </div>
  );
}

export function DropdownMenuEntry({
  baseId = "menu-entry",
  entry,
  index,
  onCommand,
  onClose,
}: {
  baseId?: string;
  entry: MenuEntry;
  index: number;
  onCommand?: Handler;
  onClose?: () => void;
}) {
  if (entry === "-" || entry === "separator") {
    return <DropdownMenuSeparator id={`${baseId}-separator-${index}`} />;
  }

  if (typeof entry !== "object" || entry == null) {
    return <DropdownMenuLabel id={`${baseId}-text-${index}`} text={String(entry)} />;
  }

  return <DropdownMenuItem baseId={baseId} item={entry} index={index} onCommand={onCommand} onClose={onClose} />;
}

export function DropdownMenuItem({ baseId = "menu-item", item, index = 0, onCommand, onClose, className }: { baseId?: string; item: Item; index?: number; onCommand?: Handler; onClose?: () => void; className?: string }) {
  const id = itemId(item, String(index));
  const label = itemLabel(item, id);
  const nested = Array.isArray(item.entries) ? (item.entries as MenuEntry[]) : [];
  const disabled = item.enabled === false || item.disabled === true;
  const selected = item.selected === true;
  const leadingIcon = item.leadingIcon ?? item.icon;
  const shortcut = item.shortcut == null ? undefined : String(item.shortcut);
  const actionId = String(item.actionId ?? item.itemId ?? item.id ?? id);
  const metadata = { item_id: actionId, item_label: label, item_index: index, entry: item };

  return (
    <div id={`${baseId}-${id}`} className={cn("flex flex-col gap-0.5", className)}>
      <Button
        id={`${baseId}-${id}-button`}
        actionId={actionId}
        variant={selected ? "secondary" : "ghost"}
        selected={selected}
        size="sm"
        disabled={disabled}
        leadingIcon={leadingIcon == null ? undefined : String(leadingIcon)}
        trailingText={shortcut}
        trailingIcon={nested.length > 0 ? "chevron-right" : undefined}
        onClick={(event) => {
          onCommand?.({ ...(event as object), metadata }, actionId);
          onClose?.();
        }}
      >
        {label}
      </Button>
      {nested.length > 0 && (
        <DropdownMenuSubContent id={`${baseId}-${id}-children`}>
          {nested.map((child, childIndex) => (
            <DropdownMenuEntry key={`${id}-${childIndex}`} baseId={`${baseId}-${id}`} entry={child} index={childIndex} onCommand={onCommand} onClose={onClose} />
          ))}
        </DropdownMenuSubContent>
      )}
    </div>
  );
}

export function DropdownMenuSubContent({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-0.5 pl-4", className))}>{children}</div>;
}

export function DropdownMenuLabel({ text, className, ...props }: NodeProps & { text?: string } & Record<string, unknown>) {
  return <LabelMuted {...props} text={text} className={cn(styles.muted, className)} />;
}

export function DropdownMenuSeparator({ className, ...props }: NodeProps & Record<string, unknown>) {
  return <Separator {...props} className={cn("my-1 bg-border", className)} />;
}
