import { styles } from "../lib/styles.ts";
import { cn, eventWithValue, itemId, itemLabel, type Handler, type Item, type NodeProps, nodeProps, requireNodeId } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";
import { Input } from "./input.tsx";
import { LabelMuted } from "./primary/label.tsx";
import { Separator } from "./primary/separator.tsx";

export type CommandProps = NodeProps & { query?: string; items?: Item[]; placeholder?: string; preview?: boolean; onCommand?: Handler<string> } & Record<string, unknown>;

export function Command({ query = "", items = [], placeholder = "Execute a command...", preview = false, className, onCommand, ...props }: CommandProps) {
  const baseId = requireNodeId(props, "Command");
  const normalizedQuery = String(query).trim().toLowerCase();
  const visibleItems = items.filter((item) => itemLabel(item, "").toLowerCase().includes(normalizedQuery) || String(item.group ?? "").toLowerCase().includes(normalizedQuery));
  const panel = <CommandPanel id={baseId} query={query} placeholder={placeholder} items={visibleItems} onCommand={onCommand} inputProps={props} />;

  return <CommandRoot {...props} preview={preview} className={className}>{panel}</CommandRoot>;
}

export function CommandRoot({ children, preview = false, className, ...props }: NodeProps & { preview?: boolean } & Record<string, unknown>) {
  return (
    <Card {...nodeProps(props, cn(preview ? cn(styles.surfaceMuted, "rounded-md") : "bg-card border border-border rounded-lg shadow-lg", className))} paddingX={preview ? 12 : 10} paddingY={preview ? 12 : 8}>
      {children}
    </Card>
  );
}

export function CommandPanel({ id, query, placeholder, items, onCommand, inputProps, className }: { id: string; query: string; placeholder: string; items: Item[]; onCommand?: Handler<string>; inputProps: Record<string, unknown>; className?: string }) {
  return (
    <div id={`${id}-panel`} className={cn("flex flex-col gap-2", className)}>
      <CommandInput id={`${id}-input`} query={query} placeholder={placeholder} width={inputProps.width ?? 360} onChange={inputProps.onChange as Handler | undefined} />
      <Separator id={`${id}-separator`} className="bg-border" />
      <CommandList id={`${id}-items`} items={items} onCommand={onCommand} />
    </div>
  );
}

export function CommandInput({ query = "", className, ...props }: NodeProps & { query?: string } & Record<string, unknown>) {
  return <Input {...props} value={query} leadingIcon="search" className={cn("w-full", className)} />;
}

export function CommandList({ id = "command-list", items = [], onCommand, className, children }: NodeProps & { items?: Item[]; onCommand?: Handler<string> } & Record<string, unknown>) {
  return (
    <div id={id} className={cn("flex flex-col gap-0.5", className)}>
      {children ?? (items.length === 0 ? <CommandEmpty /> : items.map((item, index) => <CommandItem key={itemId(item, String(index))} item={item} index={index} onCommand={onCommand} />))}
    </div>
  );
}

export function CommandEmpty({ text = "No matches", className, ...props }: NodeProps & { text?: string } & Record<string, unknown>) {
  return <LabelMuted {...props} text={text} className={cn("px-2 py-2", className)} />;
}

export function CommandItem({ item, index = 0, onCommand, className, ...props }: NodeProps & { item: Item; index?: number; onCommand?: Handler<string> } & Record<string, unknown>) {
  const id = itemId(item, String(index));
  const label = itemLabel(item, id);
  const group = item.group == null ? undefined : String(item.group);
  const shortcut = item.shortcut == null ? undefined : String(item.shortcut);
  const displayLabel = group == null ? label : `${group} - ${label}`;

  return (
    <Button
      {...props}
      id={props.id ?? `command-item-${id}`}
      variant="ghost"
      size="sm"
      actionId={id}
      className={className}
      label={displayLabel}
      trailingText={shortcut}
      onClick={(event) => onCommand?.(eventWithValue(event, id, { item_id: id, item_label: label, item_index: index, item }), id)}
    />
  );
}
