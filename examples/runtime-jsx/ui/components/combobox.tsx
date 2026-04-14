import { cn, isDisabled, itemId, itemLabel, type Handler, type Item, type NodeProps, nodeProps } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";
import { Input } from "./input.tsx";
import { LabelMuted } from "./primary/label.tsx";

function selectionEvent(event: unknown, item: Item, value: string[], label: string, index: number, itemValue: string) {
  const base = typeof event === "object" && event != null ? event as Record<string, unknown> : {};
  const metadata = typeof base.metadata === "object" && base.metadata != null ? base.metadata as Record<string, unknown> : {};
  return { ...base, value, metadata: { ...metadata, item_id: itemValue, item_label: label, item_index: index, item } };
}

function toggleSelection(values: string[], id: string) {
  return values.includes(id) ? values.filter((value) => value !== id) : [...values, id];
}

export function Combobox({ items = [], selectedItemIds = [], query = "", placeholder = "Select options", filterPlaceholder = "Filter", searchable = true, className, onSelect, onChange, ...props }: NodeProps & { items?: Item[]; selectedItemIds?: string[]; query?: string; placeholder?: string; filterPlaceholder?: string; searchable?: boolean } & Record<string, unknown>) {
  const baseId = props.id ?? props.nodeId ?? "combobox";
  const disabled = isDisabled(props);
  const selected = new Set(selectedItemIds);
  const selectedLabels = items.map((item, index) => [itemId(item, String(index)), itemLabel(item, String(index))]).filter(([id]) => selected.has(id)).map(([, label]) => label);
  const queryText = String(query ?? "");
  const normalizedQuery = queryText.trim().toLowerCase();
  const visibleItems = searchable && normalizedQuery !== "" ? items.filter((item, index) => itemLabel(item, String(index)).toLowerCase().includes(normalizedQuery)) : items;

  return (
    <ComboboxRoot {...props} className={className}>
      <ComboboxTrigger
        id={`${baseId}-trigger`}
        label={selectedLabels.length === 0 ? placeholder : selectedLabels.join(", ")}
        placeholder={selectedLabels.length === 0}
        disabled={disabled}
      />
      {searchable !== false && <ComboboxInput id={`${baseId}-filter`} value={queryText} placeholder={filterPlaceholder} disabled={disabled} onChange={onChange} />}
      <ComboboxContent id={`${baseId}-list`}>
        {visibleItems.length === 0 && <ComboboxEmpty id={`${baseId}-empty`} />}
        {visibleItems.map((item, index) => {
          const id = itemId(item, String(index));
          const label = itemLabel(item, id);
          const nextValue = toggleSelection(selectedItemIds, id);
          return (
            <ComboboxItem
              key={id}
              id={`${baseId}-option-${id}`}
              item={item}
              itemId={id}
              label={label}
              index={index}
              selected={selected.has(id)}
              value={nextValue}
              disabled={disabled}
              onSelect={onSelect}
            />
          );
        })}
      </ComboboxContent>
    </ComboboxRoot>
  );
}

export function ComboboxRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <Card {...props} className={cn("bg-popover border border-border rounded-md", className)} paddingX={8} paddingY={8}>{children}</Card>;
}

export function ComboboxTrigger({ label, placeholder = false, className, ...props }: NodeProps & { label?: string; placeholder?: boolean } & Record<string, unknown>) {
  const disabled = isDisabled(props);
  return (
    <Button
      {...props}
      variant="ghost"
      trailingIcon="chevron-down"
      disabled={disabled}
      className={cn(placeholder && "text-muted-foreground", className)}
    >
      {label}
    </Button>
  );
}

export function ComboboxInput({ className, ...props }: NodeProps & Record<string, unknown>) {
  return <Input {...props} className={cn("w-full", className)} />;
}

export function ComboboxContent({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-1", className))}>{children}</div>;
}

export function ComboboxEmpty({ text = "No matches", className, ...props }: NodeProps & { text?: string } & Record<string, unknown>) {
  return <LabelMuted {...props} text={text} className={cn("px-2 py-1.5 text-xs text-muted-foreground", className)} />;
}

export function ComboboxItem({ item, itemId: id, label, index = 0, selected = false, value, className, onSelect, children, ...props }: NodeProps & { item?: Item; itemId?: string; label?: string; index?: number; selected?: boolean; value?: string[]; onSelect?: Handler } & Record<string, unknown>) {
  const itemValue = id ?? String(index);
  const text = label ?? (item == null ? itemValue : itemLabel(item, itemValue));
  const nextValue = value ?? [itemValue];
  const disabled = isDisabled(props);

  return (
    <Button
      {...props}
      variant={selected ? "primary" : "ghost"}
      selected={selected}
      leadingIcon={selected ? "check" : undefined}
      disabled={disabled}
      className={className}
      onClick={(event) => onSelect?.(item == null ? event : selectionEvent(event, item, nextValue, text, index, itemValue), nextValue)}
    >
      {children ?? text}
    </Button>
  );
}
