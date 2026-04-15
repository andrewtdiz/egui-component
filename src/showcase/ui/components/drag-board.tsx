import { styles } from "../lib/styles.ts";
import { cn, eventWithValue, itemId, itemLabel, type Handler, type Item, type NodeProps, nodeProps } from "../component-support.ts";
import { Card } from "./card.tsx";
import { Button } from "./button.tsx";
import { Label, LabelMuted } from "./primary/label.tsx";

export function DragBoardRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-row items-stretch gap-3", className))}>{children}</div>;
}

export function DragBoardCard({ title, description, empty = false, className, children, ...props }: NodeProps & { title?: string; description?: string; empty?: boolean } & Record<string, unknown>) {
  return <Card {...props} className={cn(empty ? "bg-card border-border rounded-md" : cn(styles.surfaceMuted, "rounded-lg"), className)} paddingX={12} paddingY={12}><div className="flex flex-col items-stretch gap-2">{title != null && <Label text={title} className="font-semibold" />}{description != null && <LabelMuted text={description} />}{children}</div></Card>;
}

function moveEvent(event: unknown, item: Item, from: string, to: string, index: number, id: string) {
  const value = { item_id: id, from, to };
  return eventWithValue(event, value, { item_id: id, item_index: index, from, to, item });
}

export function DragBoardColumn({ title, region, items, onChange }: { title: string; region: string; items: Item[]; onChange?: Handler<{ item_id: string; from: string; to: string }> }) {
  const normalizedRegion = region.toLowerCase();
  const regionItems = items.filter((item) => String(item.region ?? "left").toLowerCase() === normalizedRegion);

  return (
    <DragBoardCard title={title} empty={regionItems.length === 0}>
      {regionItems.length === 0 ? (
        <LabelMuted text="Drop card here" />
      ) : (
        regionItems.map((item, index) => {
          const id = itemId(item, `${normalizedRegion}-${index}`);
          const nextRegion = normalizedRegion === "left" ? "right" : "left";
          const description = item.description == null ? undefined : String(item.description);
          return (
            <Card key={id} className="bg-card border-border rounded-md" paddingX={12} paddingY={12} onClick={(event) => onChange?.(moveEvent(event, item, normalizedRegion, nextRegion, index, id), { item_id: id, from: normalizedRegion, to: nextRegion })}>
              <div className="flex flex-col items-stretch gap-1">
                <Label text={String(item.title ?? itemLabel(item, String(index)))} className="font-semibold" />
                {description != null && <LabelMuted text={description} />}
                <Button size="sm" variant="ghost" onClick={(event) => onChange?.(moveEvent(event, item, normalizedRegion, nextRegion, index, id), { item_id: id, from: normalizedRegion, to: nextRegion })}>{nextRegion === "left" ? "Move left" : "Move right"}</Button>
              </div>
            </Card>
          );
        })
      )}
    </DragBoardCard>
  );
}

export function DragBoard({ items = [], leftTitle = "Left", rightTitle = "Right", className, onChange, ...props }: NodeProps & { items?: Item[]; leftTitle?: string; rightTitle?: string; onChange?: Handler<{ item_id: string; from: string; to: string }> } & Record<string, unknown>) {
  return <DragBoardRoot {...props} className={className}><DragBoardColumn title={leftTitle} region="left" items={items} onChange={onChange} /><DragBoardColumn title={rightTitle} region="right" items={items} onChange={onChange} /></DragBoardRoot>;
}
