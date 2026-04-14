import { cn, itemId, itemLabel, type Item, type NodeProps, nodeProps } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";

function itemEvent(event: unknown, item: Item, value: string, label: string, index: number) {
  const base = typeof event === "object" && event != null ? event as Record<string, unknown> : {};
  const metadata = typeof base.metadata === "object" && base.metadata != null ? base.metadata as Record<string, unknown> : {};
  return { ...base, value, metadata: { ...metadata, item_id: value, item_label: label, item_index: index, item } };
}

export function ToggleGroupRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <Card {...props} variant="muted" className={className} paddingX={4} paddingY={4}>
      <div className="flex flex-row items-center gap-1">{children}</div>
    </Card>
  );
}

export function ToggleGroupItem({
  item,
  index,
  selected,
  baseId,
  onSelect,
  onChange,
  className,
  ...props
}: NodeProps & {
  item: Item;
  index: number;
  selected?: boolean;
  baseId?: string;
  onSelect?: (event: unknown, value: string) => void;
  onChange?: (event: unknown, value: string) => void;
} & Record<string, unknown>) {
  const id = itemId(item, String(index));
  const label = itemLabel(item, id);
  return (
    <Button
      {...props}
      id={`${baseId ?? "toggle"}-${id}`}
      variant={selected ? "primary" : "ghost"}
      selected={selected}
      className={className}
      onClick={(event) => {
        const nextEvent = itemEvent(event, item, id, label, index);
        onSelect?.(nextEvent, id);
        onChange?.(nextEvent, id);
      }}
    >
      {label}
    </Button>
  );
}

export function ToggleGroup({ items = [], selectedItemId, className, onSelect, onChange, ...props }: NodeProps & { items?: Item[]; selectedItemId?: string } & Record<string, unknown>) {
  const baseId = props.id ?? props.nodeId ?? "toggle";
  return (
    <ToggleGroupRoot {...props} className={className}>
      {items.map((item, index) => (
        <ToggleGroupItem
          key={itemId(item, String(index))}
          item={item}
          index={index}
          selected={itemId(item, String(index)) === selectedItemId}
          baseId={baseId}
          onSelect={onSelect}
          onChange={onChange}
        />
      ))}
    </ToggleGroupRoot>
  );
}
