import { cn, isDisabled, itemId, itemLabel, type Handler, type Item, type NodeProps, nodeProps } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";

function selectionEvent(event: unknown, item: Item, value: string, label: string, index: number) {
  const base = typeof event === "object" && event != null ? event as Record<string, unknown> : {};
  const metadata = typeof base.metadata === "object" && base.metadata != null ? base.metadata as Record<string, unknown> : {};
  return { ...base, value, metadata: { ...metadata, item_id: value, item_label: label, item_index: index, item } };
}

export function Select({ items = [], selectedItemId, placeholder = "Select an option", leadingIcon, variant = "default", className, onSelect, onChange, ...props }: NodeProps & { items?: Item[]; selectedItemId?: string; placeholder?: string; leadingIcon?: string; variant?: string } & Record<string, unknown>) {
  const baseId = props.id ?? props.nodeId ?? "select";
  const selectedItem = items.find((item, index) => itemId(item, String(index)) === selectedItemId);
  const selectedLabel = selectedItem == null ? placeholder : itemLabel(selectedItem, selectedItemId ?? "selected");
  const disabled = isDisabled(props);

  return (
    <SelectRoot {...props} className={className}>
      <SelectTrigger
        id={`${baseId}-trigger`}
        label={selectedLabel}
        placeholder={selectedItem == null}
        variant={variant}
        leadingIcon={leadingIcon}
        disabled={disabled}
        onChange={onChange}
        selectedItemId={selectedItemId}
      />
      <SelectContent id={`${baseId}-options`}>
        {items.map((item, index) => {
          const id = itemId(item, String(index));
          const label = itemLabel(item, id);
          return (
            <SelectItem
              key={id}
              id={`${baseId}-option-${id}`}
              item={item}
              itemId={id}
              label={label}
              index={index}
              selected={id === selectedItemId}
              disabled={disabled}
              onSelect={onSelect}
            />
          );
        })}
      </SelectContent>
    </SelectRoot>
  );
}

export function SelectRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-1.5", className))}>{children}</div>;
}

export function SelectTrigger({ label, placeholder = false, variant = "default", leadingIcon, className, onChange, selectedItemId, ...props }: NodeProps & { label?: string; placeholder?: boolean; variant?: string; leadingIcon?: string; selectedItemId?: string; onChange?: Handler } & Record<string, unknown>) {
  const disabled = isDisabled(props);
  return (
    <Button
      {...props}
      variant={variant === "secondary" ? "secondary" : "ghost"}
      leadingIcon={leadingIcon}
      trailingIcon="chevron-down"
      disabled={disabled}
      className={cn(placeholder && "text-muted-foreground", className)}
      onClick={(event) => onChange?.(event, selectedItemId)}
    >
      {label}
    </Button>
  );
}

export function SelectContent({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <Card {...props} variant="muted" className={className} paddingX={4} paddingY={4}>
      <div className="flex flex-row flex-wrap items-center gap-1">{children}</div>
    </Card>
  );
}

export function SelectItem({ item, itemId: id, label, index = 0, selected = false, className, onSelect, children, ...props }: NodeProps & { item?: Item; itemId?: string; label?: string; index?: number; selected?: boolean; onSelect?: Handler } & Record<string, unknown>) {
  const value = id ?? String(index);
  const text = label ?? (item == null ? value : itemLabel(item, value));
  const disabled = isDisabled(props);

  return (
    <Button
      {...props}
      size="sm"
      variant={selected ? "primary" : "ghost"}
      selected={selected}
      disabled={disabled}
      className={className}
      onClick={(event) => onSelect?.(item == null ? event : selectionEvent(event, item, value, text, index), value)}
    >
      {children ?? text}
    </Button>
  );
}
