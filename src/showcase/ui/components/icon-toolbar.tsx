import { styles } from "../lib/styles.ts";
import { cn, eventWithValue, itemId, itemLabel, type Handler, type Item, type NodeProps, nodeProps, requireNodeId } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";

function itemEvent(event: unknown, item: Item, index: number, id: string, label: string) {
  return eventWithValue(event, id, { item_id: id, item_label: label, item_index: index, item });
}

export function IconToolbarRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <Card {...props} className={cn(styles.surfaceMuted, "rounded-lg", className)} paddingX={6} paddingY={4}>{children}</Card>;
}

export function IconToolbarItemButton({ item, index = 0, selected = false, baseId = "icon-toolbar", onSelect, className, ...props }: NodeProps & { item: Item; index?: number; selected?: boolean; baseId?: string; onSelect?: Handler<string> } & Record<string, unknown>) {
  const id = itemId(item, String(index));
  const label = item.tooltip ?? item.label ?? id;

  return (
    <Button
      {...props}
      id={`${baseId}-${id}`}
      variant={selected ? "primary" : "ghost"}
      selected={selected}
      iconOnly
      leadingIcon={String(item.icon ?? "circle")}
      className={className}
      onClick={(event) => onSelect?.(itemEvent(event, item, index, id, label), id)}
      aria-label={label}
    />
  );
}

export function IconToolbar({ items = [], selectedItemId, className, onSelect, ...props }: NodeProps & { items?: Item[]; selectedItemId?: string; onSelect?: Handler<string> } & Record<string, unknown>) {
  const baseId = requireNodeId(props, "IconToolbar");
  return (
    <IconToolbarRoot {...props} className={className}>
      <div {...nodeProps({}, "flex flex-row items-center gap-1")}>
        {items.map((item, index) => {
          const id = itemId(item, String(index));
          return (
            <IconToolbarItemButton
              key={id}
              item={item}
              index={index}
              baseId={baseId}
              selected={id === selectedItemId}
              onSelect={onSelect}
            />
          );
        })}
      </div>
    </IconToolbarRoot>
  );
}
