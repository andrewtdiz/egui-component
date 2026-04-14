import { styles } from "../lib/styles.ts";
import { cn, itemId, itemLabel, type Handler, type Item, type NodeProps, nodeProps, requireNodeId } from "../component-support.ts";
import { Card } from "./card.tsx";
import { Icon } from "./primary/icon.tsx";
import { Label } from "./primary/label.tsx";
import { Twemoji } from "./primary/twemoji.tsx";

function itemEvent(event: unknown, item: Item, id: string, index: number) {
  const base = typeof event === "object" && event != null ? event as Record<string, unknown> : {};
  const metadata = typeof base.metadata === "object" && base.metadata != null ? base.metadata as Record<string, unknown> : {};
  return { ...base, value: id, metadata: { ...metadata, item_id: id, item_index: index, item } };
}

export function HierarchyRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <Card {...props} className={cn(styles.surfaceMuted, "rounded-lg", className)} paddingX={8} paddingY={8}>{children}</Card>;
}

export function HierarchyGlyph({ item, iconStyle }: { item: Item; iconStyle: string }) {
  const emoji = item.emoji ?? hierarchyEmoji(String(item.kind ?? "gameObject"));
  if (iconStyle === "icons") return <Icon name={String(item.iconName ?? item.icon_name ?? hierarchyIcon(String(item.kind ?? "gameObject")))} size={14} className="text-muted-foreground" />;
  return <Twemoji emoji={String(emoji)} size={16} />;
}

export function HierarchyRow({ item, selectedIds, iconStyle, depth, treeId, fallback, rowHeight = 32, onSelect }: { item: Item; selectedIds: string[]; iconStyle: string; depth: number; treeId: string; fallback: string; rowHeight?: number; onSelect?: Handler }) {
  const id = itemId(item, fallback);
  const children = Array.isArray(item.children) ? item.children as Item[] : [];
  const open = item.open ?? item.expanded ?? true;
  const selected = selectedIds.includes(id);
  const locked = item.locked === true;
  const rowClassName = cn("rounded-md", `h-[${rowHeight}px]`, selected && "bg-accent");
  const emphasisTextClassName = selected ? "text-accent-foreground font-semibold" : "text-foreground font-medium";
  const mutedIconClassName = selected ? "text-accent-foreground" : "text-muted-foreground";

  return (
    <div className="flex flex-col items-stretch gap-0">
      <div {...nodeProps({ id: `${treeId}-${id}`, paddingLeft: depth * 18, onClick: (event) => onSelect?.(itemEvent(event, item, id, depth), id) })}>
        <Card variant="plain" className={rowClassName} paddingX={8} paddingY={6}>
          <div className="flex flex-row items-center gap-2">
            <Icon name={children.length > 0 ? (open ? "chevron-down" : "chevron-right") : "dot"} size={12} className={mutedIconClassName} />
            {iconStyle === "icons"
              ? <Icon name={String(item.iconName ?? item.icon_name ?? hierarchyIcon(String(item.kind ?? "gameObject")))} size={14} className={mutedIconClassName} />
              : <HierarchyGlyph item={item} iconStyle={iconStyle} />}
            <Label text={itemLabel(item, id)} className={cn("text-sm", emphasisTextClassName)} />
            {locked && <Icon name="lock" size={13} className={mutedIconClassName} />}
          </div>
        </Card>
      </div>
      {open && children.map((child, index) => <HierarchyRow key={itemId(child, `${id}-${index}`)} item={child} selectedIds={selectedIds} iconStyle={iconStyle} depth={depth + 1} treeId={treeId} fallback={`${id}-${index}`} rowHeight={rowHeight} onSelect={onSelect} />)}
    </div>
  );
}

export function Hierarchy({ items = [], selectedItemId, selectedItemIds, iconStyle = "emoji", className, onSelect, rowHeight = 32, ...props }: NodeProps & { items?: Item[]; selectedItemId?: string; selectedItemIds?: string[]; iconStyle?: string; rowHeight?: number } & Record<string, unknown>) {
  const selectedIds = selectedItemIds ?? (selectedItemId != null ? [selectedItemId] : []);
  const baseId = requireNodeId(props, "Hierarchy");
  return <HierarchyRoot {...props} className={className}>{items.map((item, index) => <HierarchyRow key={itemId(item, String(index))} item={item} selectedIds={selectedIds} iconStyle={iconStyle} depth={0} treeId={baseId} fallback={String(index)} rowHeight={rowHeight} onSelect={onSelect} />)}</HierarchyRoot>;
}

function hierarchyEmoji(kind: string) {
  if (kind === "folder") return "📁";
  if (kind === "player") return "🧍";
  if (kind === "weapon") return "⚔️";
  if (kind === "clothing") return "👕";
  if (kind === "hitbox") return "🎯";
  if (kind === "vector") return "🔷";
  if (kind === "frame") return "🧩";
  if (kind === "group") return "⚙️";
  return "📦";
}

function hierarchyIcon(kind: string) {
  if (kind === "folder") return "bootstrap:folder-fill";
  if (kind === "player") return "person-standing";
  if (kind === "weapon") return "sword";
  if (kind === "clothing") return "shirt";
  if (kind === "hitbox") return "crosshair";
  if (kind === "vector") return "diamond";
  if (kind === "frame") return "frame";
  if (kind === "group") return "group";
  return "box";
}
