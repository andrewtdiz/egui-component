import { boolValue, cn, itemId, itemLabel, type Item, type NodeProps, nodeProps, requireNodeId, textFromChildren } from "../component-support.ts";

function itemEvent(event: unknown, item: Item, value: string, label: string, index: number) {
  const base = typeof event === "object" && event != null ? event as Record<string, unknown> : {};
  const metadata = typeof base.metadata === "object" && base.metadata != null ? base.metadata as Record<string, unknown> : {};
  return { ...base, value, metadata: { ...metadata, item_id: value, item_label: label, item_index: index, item } };
}

export function Radio({ children, checked, value, label, description, className, ...props }: NodeProps & { checked?: boolean; value?: boolean; label?: string; description?: string } & Record<string, unknown>) {
  const resolvedValue = boolValue(value, checked);
  return <input data-slot="radio" type="radio" {...nodeProps({ ...props, id: requireNodeId(props, "Radio") }, className)} checked={resolvedValue} label={label ?? textFromChildren(children)} description={description} />;
}

export function RadioGroup({ items = [], selectedItemId, className, onSelect, ...props }: NodeProps & { items?: Item[]; selectedItemId?: string } & Record<string, unknown>) {
  const baseId = requireNodeId(props, "RadioGroup");
  return (
    <div {...nodeProps(props, cn("flex flex-col items-stretch gap-2", className))}>
      {items.map((item, index) => {
        const id = itemId(item, String(index));
        const label = itemLabel(item, id);
        const description = typeof item.description === "string" ? item.description : undefined;
        return (
          <Radio
            key={id}
            id={`${baseId}-${id}`}
            value={id === selectedItemId}
            label={label}
            description={description}
            className="px-1"
            onToggle={(event) => onSelect?.(itemEvent(event, item, id, label, index), id)}
          />
        );
      })}
    </div>
  );
}
