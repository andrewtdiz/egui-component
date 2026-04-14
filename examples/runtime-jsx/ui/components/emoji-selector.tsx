import { useState } from "egui";
import { cn, type Handler, type NodeProps, nodeProps, requireNodeId } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";
import { LabelMuted } from "./primary/label.tsx";

const DEFAULT_EMOJIS = ["😀", "😁", "😂", "🙂", "😍", "😎", "🤔", "😢", "🔥", "✨", "🎉", "🚀"];

export function EmojiSelectorRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-2", className))}>{children}</div>;
}

export function EmojiSelectorTrigger({ value = "🙂", placeholder = "🙂", triggerVariant = "secondary", className, ...props }: NodeProps & { value?: string; placeholder?: string; triggerVariant?: string } & Record<string, unknown>) {
  return <Button {...props} className={className} variant={triggerVariant} size="md">{value ?? placeholder}</Button>;
}

export function EmojiSelectorItem({ emoji, selected = false, onSelect }: { emoji: string; selected?: boolean; onSelect?: Handler }) {
  return <Button variant={selected ? "primary" : "ghost"} selected={selected} size="sm" className="h-8 w-8 p-0" label={emoji} onClick={(event) => onSelect?.({ ...(event as object), value: emoji, metadata: { emoji } }, emoji)} />;
}

export function EmojiSelectorContent({ emojis = DEFAULT_EMOJIS, value, onSelect, className, ...props }: NodeProps & { emojis?: string[]; value?: string; onSelect?: Handler } & Record<string, unknown>) {
  return (
    <Card {...props} className={cn("bg-popover border border-border rounded-lg shadow-md", className)} paddingX={12} paddingY={12}>
      <div className="flex flex-col items-stretch gap-1.5">
        <LabelMuted text="Choose an emoji" className="text-xs font-semibold" />
        <div className="flex flex-row flex-wrap items-center gap-1">
          {emojis.map((emoji) => <EmojiSelectorItem key={emoji} emoji={emoji} selected={emoji === value} onSelect={onSelect} />)}
        </div>
      </div>
    </Card>
  );
}

export function EmojiSelector({ value = "🙂", placeholder = "🙂", triggerVariant = "secondary", className, onSelect, ...props }: NodeProps & { value?: string; placeholder?: string; triggerVariant?: string; onSelect?: Handler } & Record<string, unknown>) {
  const [open, setOpen] = useState(false);
  const baseId = requireNodeId(props, "EmojiSelector");
  const resolvedValue = value ?? placeholder;

  return (
    <EmojiSelectorRoot {...props} className={className}>
      <EmojiSelectorTrigger id={`${baseId}-trigger`} value={resolvedValue} placeholder={placeholder} triggerVariant={triggerVariant} onClick={() => setOpen((next) => !next)} />
      {open && <EmojiSelectorContent id={`${baseId}-content`} value={resolvedValue} onSelect={(event, emoji) => { onSelect?.(event, emoji); setOpen(false); }} />}
    </EmojiSelectorRoot>
  );
}
