import { useState } from "egui";
import { cn, eventWithValue, itemId, itemLabel, type Handler, type NodeProps, nodeProps, requireNodeId } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";
import { DropdownMenuContent, type MenuEntry } from "./dropdown-menu.tsx";

export type OpenWithProps = NodeProps & { entries?: MenuEntry[]; selectedItemId?: string; placeholder?: string; open?: boolean; triggerVariant?: string; onCommand?: Handler<string | undefined>; onOpen?: Handler<string | undefined>; onClose?: Handler<string | undefined> } & Record<string, unknown>;

export function OpenWithRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-1.5", className))}>{children}</div>;
}

export function OpenWithTrigger({ baseId = "open-with", label = "Open With", icon, triggerVariant = "secondary", onAction, onMenu, className }: { baseId?: string; label?: string; icon?: string; triggerVariant?: string; onAction?: NodeProps["onClick"]; onMenu?: NodeProps["onClick"]; className?: string }) {
  return (
    <div id={`${baseId}-trigger`} className={cn("flex flex-row items-center gap-0", className)}>
      <Button
        id={`${baseId}-action`}
        variant={triggerVariant}
        leadingIcon={icon}
        onClick={onAction}
      >
        {label}
      </Button>
      <Button
        id={`${baseId}-menu`}
        variant={triggerVariant}
        iconOnly
        leadingIcon="chevron-down"
        onClick={onMenu}
      />
    </div>
  );
}

export function OpenWithMenu({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <Card {...nodeProps(props, cn("bg-popover border border-border rounded-md shadow-md", className))} paddingX={4} paddingY={4}>
      {children}
    </Card>
  );
}

export function OpenWith({ entries = [], selectedItemId, placeholder = "Open With", open, triggerVariant = "secondary", className, onCommand, onOpen, onClose, ...props }: OpenWithProps) {
  const baseId = requireNodeId(props, "OpenWith");
  const [internalOpen, setInternalOpen] = useState(false);
  const resolvedOpen = open ?? internalOpen;
  const current = entries.find((entry) => typeof entry === "object" && entry != null && itemId(entry, "") === selectedItemId);
  const label = current && typeof current === "object" ? itemLabel(current, placeholder) : placeholder;
  const icon = current && typeof current === "object" ? current.leadingIcon ?? current.icon : undefined;

  const setOpen = (next: boolean, event?: unknown) => {
    if (open == null) {
      setInternalOpen(next);
    }
    if (next) onOpen?.(eventWithValue(event, selectedItemId, { open: next }), selectedItemId);
    else onClose?.(eventWithValue(event, selectedItemId, { open: next }), selectedItemId);
  };

  return (
    <OpenWithRoot {...props} className={className}>
      <OpenWithTrigger
        baseId={baseId}
        label={label}
        icon={icon == null ? undefined : String(icon)}
        triggerVariant={triggerVariant}
        onAction={(event) => onCommand?.(eventWithValue(event, selectedItemId, { item_id: selectedItemId, item_label: label }), selectedItemId)}
        onMenu={(event) => setOpen(!resolvedOpen, event)}
      />
      {resolvedOpen && entries.length > 0 && (
        <OpenWithMenu id={`${baseId}-content`}>
          <DropdownMenuContent id={`${baseId}-items`} entries={entries} onCommand={onCommand} onClose={() => setOpen(false)} />
        </OpenWithMenu>
      )}
    </OpenWithRoot>
  );
}
