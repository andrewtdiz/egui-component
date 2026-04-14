import { cn, type NodeProps, nodeProps } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";
import { Label, LabelMuted } from "./primary/label.tsx";

export function DialogueRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <Card
      {...nodeProps(props, cn("bg-card border border-border rounded-lg shadow-lg", className))}
      width={props.width ?? 360}
      paddingX={12}
      paddingY={12}
    >
      {children}
    </Card>
  );
}

export function DialogueTitle({ title = "Dialog", intent = "default", className, ...props }: NodeProps & { title?: string; intent?: string } & Record<string, unknown>) {
  const alert = intent === "alert" || intent === "destructive";
  return (
    <Label
      {...props}
      text={title}
      tone={alert ? "destructive" : "primary"}
      weight="bold"
      className={cn("text-base font-bold", className)}
    />
  );
}

export function DialogueDescription({ description, className, ...props }: NodeProps & { description?: string } & Record<string, unknown>) {
  if (description == null || description === "") return null;
  return <LabelMuted {...props} text={description} className={cn("text-xs", className)} />;
}

export function DialogueClose({ className, onClose, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <Button
      {...props}
      variant="ghost"
      iconOnly
      leadingIcon="x"
      className={cn("h-7 w-7", className)}
      size="sm"
      onClick={onClose}
    />
  );
}

export function DialogueHeader({
  title = "Dialog",
  description,
  intent = "default",
  onClose,
  className,
  children,
  ...props
}: NodeProps & { title?: string; description?: string; intent?: string; onClose?: NodeProps["onClose"] } & Record<string, unknown>) {
  const baseId = props.id ?? props.nodeId ?? "dialogue";
  return (
    <div {...nodeProps(props, cn("flex flex-row items-start gap-2", className))}>
      <div className="flex flex-col gap-1">
        <DialogueTitle title={title} intent={intent} />
        <DialogueDescription description={description} />
        {children}
      </div>
      <div data-slot="spacer" />
      <DialogueClose id={`${baseId}-close`} onClose={onClose} />
    </div>
  );
}

export function DialogueFooter({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-row items-center justify-end gap-2", className))}>{children}</div>;
}

export function DialogueAction({ children, variant = "secondary", ...props }: NodeProps & { variant?: string } & Record<string, unknown>) {
  return <Button {...props} variant={variant}>{children}</Button>;
}

export function DialogueContent({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-12", className))}>{children}</div>;
}

export function DialogueModal({ children, open = false, title = "Dialog", description, intent = "default", className, confirmLabel = "Confirm", cancelLabel = "Cancel", onCancel, onClose, onConfirm, ...props }: NodeProps & { open?: boolean; title?: string; description?: string; intent?: string; confirmLabel?: string; cancelLabel?: string } & Record<string, unknown>) {
  if (!open) return null;
  const baseId = props.id ?? props.nodeId ?? "dialogue";
  const alert = intent === "alert" || intent === "destructive";
  return (
    <DialogueRoot {...nodeProps(props, className)}>
      <DialogueContent>
        <DialogueHeader id={`${baseId}-header`} title={title} description={description} intent={intent} onClose={onClose ?? onCancel} />
        {children}
        <DialogueFooter id={`${baseId}-footer`}>
          <DialogueAction id={`${baseId}-cancel`} variant="ghost" onClick={onCancel ?? onClose}>{cancelLabel}</DialogueAction>
          <DialogueAction id={`${baseId}-confirm`} variant={alert ? "primary" : "secondary"} onClick={onConfirm}>{confirmLabel}</DialogueAction>
        </DialogueFooter>
      </DialogueContent>
    </DialogueRoot>
  );
}

export const Dialogue = DialogueModal;
