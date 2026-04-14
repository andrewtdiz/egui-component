import { cn, itemId, itemLabel, type Item, type NodeProps, nodeProps, requireNodeId } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";

function itemEvent(event: unknown, item: Item, value: string, label: string, index: number) {
  const base = typeof event === "object" && event != null ? event as Record<string, unknown> : {};
  const metadata = typeof base.metadata === "object" && base.metadata != null ? base.metadata as Record<string, unknown> : {};
  return { ...base, value, metadata: { ...metadata, item_id: value, item_label: label, item_index: index, item } };
}

export function TabsList({ items = [], selectedItemId, style = "underline", className, onSelect, children, ...props }: NodeProps & { items?: Item[]; selectedItemId?: string; style?: string } & Record<string, unknown>) {
  const baseId = requireNodeId(props, "TabsList");
  const segmented = style === "segmented" || style === "blenderTopbar" || style === "blender_topbar";
  const vertical = style === "stacked" || style === "rail";
  const layoutClassName = vertical ? "flex flex-col items-start" : "flex flex-row items-center";
  const content = (
    <>
      {items.map((item, index) => {
        const id = itemId(item, String(index));
        const label = itemLabel(item, id);
        const selected = id === selectedItemId;
        const icon = typeof item.icon === "string" ? item.icon : undefined;
        const iconOnly = item.iconOnly === true || item.icon_only === true;
        return (
          <TabsTrigger
            key={id}
            id={`${baseId}-${id}`}
            item={item}
            index={index}
            selected={selected}
            label={label}
            icon={icon}
            iconOnly={iconOnly}
            segmented={segmented}
            onSelect={onSelect}
          />
        );
      })}
      {children}
    </>
  );

  if (segmented) {
    return (
      <Card {...props} variant="muted" className={className} paddingX={4} paddingY={4}>
        <div className={cn(layoutClassName, "gap-1")}>{content}</div>
      </Card>
    );
  }

  return <div {...nodeProps(props, cn(layoutClassName, "gap-1.5", className))}>{content}</div>;
}

export function TabsTrigger({
  item,
  index,
  selected,
  label,
  icon,
  iconOnly,
  segmented = false,
  className,
  onSelect,
  ...props
}: NodeProps & {
  item: Item;
  index: number;
  selected?: boolean;
  label: string;
  icon?: string;
  iconOnly?: boolean;
  segmented?: boolean;
  onSelect?: (event: unknown, value: string) => void;
} & Record<string, unknown>) {
  const id = itemId(item, String(index));
  return (
    <Button
      {...nodeProps(props, className)}
      variant={selected ? "primary" : segmented ? "ghost" : "link"}
      size="sm"
      selected={selected}
      leadingIcon={icon}
      iconOnly={iconOnly}
      onClick={(event) => onSelect?.(itemEvent(event, item, id, label, index), id)}
    >
      {iconOnly ? "" : label}
    </Button>
  );
}

export function TabsContent({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-2", className))}>{children}</div>;
}

export function Tabs({ items = [], selectedItemId, style = "underline", className, onSelect, children, ...props }: NodeProps & { items?: Item[]; selectedItemId?: string; style?: string } & Record<string, unknown>) {
  return (
    <TabsList {...props} items={items} selectedItemId={selectedItemId} style={style} className={className} onSelect={onSelect}>
      {children}
    </TabsList>
  );
}
