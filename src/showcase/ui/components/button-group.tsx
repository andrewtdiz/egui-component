import { cn, eventWithValue, itemId, itemLabel, type Handler, type Item, type NodeProps, nodeProps, requireNodeId } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";

export type ButtonGroupProps = NodeProps & { items?: Item[]; selectedItemId?: string; onSelect?: Handler<string>; onCommand?: Handler<string> } & Record<string, unknown>;

export function ButtonGroup({ children, items, selectedItemId, className, onSelect, onCommand, ...props }: ButtonGroupProps) {
  const baseId = requireNodeId(props, "ButtonGroup");

  if (children != null) {
    return <ButtonGroupRoot {...props} className={className}>{children}</ButtonGroupRoot>;
  }

  return (
    <ButtonGroupRoot {...props} className={className}>
      {(items ?? []).map((item, index) => {
        const id = itemId(item, String(index));
        const label = itemLabel(item, id);
        return (
          <ButtonGroupItem
            key={id}
            id={`${baseId}-${id}`}
            item={item}
            itemId={id}
            label={label}
            index={index}
            selected={id === selectedItemId}
            onSelect={onSelect}
            onCommand={onCommand}
          />
        );
      })}
    </ButtonGroupRoot>
  );
}

export function ButtonGroupRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <Card {...props} variant="muted" className={className} paddingX={4} paddingY={4}>
      <div className="flex flex-row items-center gap-1">{children}</div>
    </Card>
  );
}

export function ButtonGroupItem({ item, itemId: id, label, index = 0, selected = false, className, onSelect, onCommand, children, ...props }: NodeProps & { item?: Item; itemId?: string; label?: string; index?: number; selected?: boolean; onSelect?: Handler<string>; onCommand?: Handler<string> } & Record<string, unknown>) {
  const value = id ?? String(index);
  const text = label ?? (item == null ? value : itemLabel(item, value));

  return (
    <Button
      {...props}
      variant={selected ? "primary" : "ghost"}
      selected={selected}
      className={className}
      onClick={(event) => {
        const nextEvent = item == null ? event : eventWithValue(event, value, { item_id: value, item_label: text, item_index: index, item });
        onSelect?.(nextEvent, value);
        onCommand?.(nextEvent, value);
      }}
    >
      {children ?? text}
    </Button>
  );
}
