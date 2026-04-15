import { cn, eventWithValue, itemId, itemLabel, type Handler, type Item, type NodeProps, nodeProps, requireNodeId } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";

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
  onSelect?: Handler<string>;
  onChange?: Handler<string>;
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
        const nextEvent = eventWithValue(event, id, { item_id: id, item_label: label, item_index: index, item });
        onSelect?.(nextEvent, id);
        onChange?.(nextEvent, id);
      }}
    >
      {label}
    </Button>
  );
}

export function ToggleGroup({ items = [], selectedItemId, className, onSelect, onChange, ...props }: NodeProps & { items?: Item[]; selectedItemId?: string } & Record<string, unknown>) {
  const baseId = requireNodeId(props, "ToggleGroup");
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
