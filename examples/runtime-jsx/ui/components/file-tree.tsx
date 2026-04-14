import { styles } from "../lib/styles.ts";
import { cn, itemId, itemLabel, type Handler, type Item, type NodeProps, nodeProps } from "../component-support.ts";
import { Card } from "./card.tsx";
import { Icon } from "./primary/icon.tsx";
import { Label } from "./primary/label.tsx";

function itemEvent(event: unknown, item: Item, id: string, index: number) {
  const base = typeof event === "object" && event != null ? event as Record<string, unknown> : {};
  const metadata = typeof base.metadata === "object" && base.metadata != null ? base.metadata as Record<string, unknown> : {};
  return { ...base, value: id, metadata: { ...metadata, item_id: id, item_index: index, item } };
}

export function FileTreeRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <Card {...props} className={cn(styles.surfaceMuted, "rounded-md", className)} paddingX={6} paddingY={6}>{children}</Card>;
}

export function FileTreeRow({ item, selectedItemId, depth, treeId, fallback, onSelect }: { item: Item; selectedItemId?: string; depth: number; treeId: string; fallback: string; onSelect?: Handler }) {
  const id = itemId(item, fallback);
  const children = Array.isArray(item.children) ? item.children as Item[] : [];
  const open = item.open ?? item.expanded ?? true;
  const kind = String(item.kind ?? "file");
  const selected = selectedItemId === id;
  const rowClassName = cn("rounded-sm h-5", selected && "bg-accent");
  const labelClassName = selected ? "text-accent-foreground text-xs font-medium" : "text-foreground text-xs font-medium";
  const iconClassName = selected ? "text-accent-foreground" : "text-muted-foreground";

  return (
    <div className="flex flex-col gap-0">
      <div {...nodeProps({ id: `${treeId}-${id}`, paddingLeft: depth * 14, onClick: (event) => onSelect?.(itemEvent(event, item, id, depth), id) })}>
        <Card variant="plain" className={rowClassName} paddingX={4} paddingY={2}>
          <div className="flex flex-row items-center gap-1.5">
            <Icon name={children.length > 0 ? (open ? "bootstrap:caret-down-fill" : "bootstrap:caret-right-fill") : "bootstrap:file-earmark-text-fill"} size={10} className={iconClassName} />
            <Icon name={fileTreeIcon(kind, Boolean(open))} size={13} className={iconClassName} />
            <Label text={itemLabel(item, id)} className={labelClassName} />
          </div>
        </Card>
      </div>
      {open && children.map((child, index) => <FileTreeRow key={itemId(child, `${id}-${index}`)} item={child} selectedItemId={selectedItemId} depth={depth + 1} treeId={treeId} fallback={`${id}-${index}`} onSelect={onSelect} />)}
    </div>
  );
}

export function FileTree({ items = [], selectedItemId, className, onSelect, ...props }: NodeProps & { items?: Item[]; selectedItemId?: string; onSelect?: Handler } & Record<string, unknown>) {
  return <FileTreeRoot {...props} className={className}>{items.map((item, index) => <FileTreeRow key={itemId(item, String(index))} item={item} selectedItemId={selectedItemId} depth={0} treeId={props.id ?? props.nodeId ?? "file-tree"} fallback={String(index)} onSelect={onSelect} />)}</FileTreeRoot>;
}

function fileTreeIcon(kind: string, open: boolean) {
  if (kind === "folder") return open ? "bootstrap:folder2-open" : "bootstrap:folder2";
  if (kind === "collection") return "bootstrap:collection-fill";
  if (kind === "script") return "bootstrap:file-earmark-code-fill";
  if (kind === "project") return "bootstrap:gear-fill";
  if (kind === "markdown") return "bootstrap:filetype-md";
  return "bootstrap:file-earmark-text-fill";
}
