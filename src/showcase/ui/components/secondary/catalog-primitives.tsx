import { componentPreviewStyles, styles } from "../../lib/styles.ts";
import { cn, type NodeProps } from "../../component-support.ts";
import { Button, type ButtonProps } from "../button.tsx";
import { Card } from "../card.tsx";
import { Label, LabelMuted, LabelTitle } from "../primary/label.tsx";

export function Surface({ id, nodeId, title, description, className, children, paddingX = 16, paddingY = 16 }: { id?: string; nodeId?: string; title?: string; description?: string; className?: string; children?: NodeProps["children"]; paddingX?: number; paddingY?: number }) {
  return <Card id={nodeId ?? id} paddingX={paddingX} paddingY={paddingY} className={cn(componentPreviewStyles.shell, className)}><div className="flex flex-col items-stretch gap-2.5">{(title != null || description != null) && <div className="flex flex-col items-stretch gap-1">{title != null && <LabelTitle text={title} />}{description != null && <LabelMuted text={description} />}</div>}{children}</div></Card>;
}

export const Title = LabelTitle;
export const MutedText = LabelMuted;

export function BadgeText({ id, text, className }: { id?: string; text: string; className?: string }) {
  return <Label id={id} text={text} className={cn(styles.badge, className)} />;
}

export function ActionButton(props: ButtonProps) {
  return <Button {...props} />;
}
