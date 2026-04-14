import { cn, itemId, itemLabel, type Item, type NodeProps, nodeProps } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";
import { Label, LabelMuted } from "./primary/label.tsx";

export type ToastViewportProps = NodeProps & { toasts?: Item[] } & Record<string, unknown>;

export function ToastViewport({ toasts = [], children, className, onClose, ...props }: ToastViewportProps) {
  const maxVisible = Number(props.maxVisible ?? props.max_visible ?? 4);
  const gap = Number(props.gap ?? 8);
  const visible = toasts.slice(0, maxVisible);

  return (
    <div {...nodeProps(props, cn("flex flex-col", `gap-[${gap}px]`, className))}>
      {children != null
        ? children
        : visible.map((toast, index) => (
            <Toast key={itemId(toast, String(index))} toast={toast} index={index} onClose={onClose} />
          ))}
    </div>
  );
}

export function ToastRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <Card
      {...nodeProps(props, cn("bg-card border border-border rounded-md shadow-md", className))}
      paddingX={10}
      paddingY={10}
    >
      {children}
    </Card>
  );
}

export function ToastContent({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <div {...nodeProps(props, cn("flex flex-col gap-1", className))}>
      {children}
    </div>
  );
}

export function ToastHeader({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <div {...nodeProps(props, cn("flex flex-row items-center gap-3", className))}>
      {children}
    </div>
  );
}

export function ToastTitle({ title = "Toast", className, ...props }: NodeProps & { title?: string } & Record<string, unknown>) {
  return <Label {...props} text={title} weight="semibold" className={cn("text-sm font-semibold", className)} />;
}

export function ToastDescription({ description, className, ...props }: NodeProps & { description?: string } & Record<string, unknown>) {
  if (description == null || description === "") return null;
  return <LabelMuted {...props} text={description} className={cn("text-xs", className)} />;
}

export function ToastClose({ toast, title, index = 0, onClose, className, ...props }: NodeProps & { toast: Item; title: string; index?: number } & Record<string, unknown>) {
  const id = itemId(toast, String(index));
  return (
    <Button
      {...props}
      variant="ghost"
      iconOnly
      leadingIcon="x"
      size="sm"
      className={cn("h-7 w-7", className)}
      onClick={(event) => onClose?.({ ...(event as object), metadata: { item_id: id, item_label: title, item_index: index, toast } }, id)}
    />
  );
}

export function ToastAction({ children, variant = "secondary", className, ...props }: NodeProps & { variant?: string } & Record<string, unknown>) {
  return <Button {...props} variant={variant} className={className}>{children}</Button>;
}

export function Toast({ toast, index = 0, onClose, children }: { toast: Item; index?: number; onClose?: NodeProps["onClose"]; children?: unknown }) {
  const id = itemId(toast, String(index));
  const title = itemLabel(toast, "Toast");
  const description = toast.description == null ? undefined : String(toast.description);
  const intent = String(toast.intent ?? "neutral");

  return (
    <ToastRoot
      id={`toast-${id}`}
      paddingX={10}
      paddingY={10}
      className={cn(
        "bg-card border border-border rounded-md shadow-md",
        intent === "destructive" && "border-destructive",
        intent === "success" && "border-primary",
      )}
    >
      <ToastHeader>
        <ToastContent>
          <ToastTitle title={title} className={intent === "destructive" ? "text-destructive" : undefined} />
          {description != null && <ToastDescription description={description} />}
          {children}
        </ToastContent>
        <div data-slot="spacer" />
        <ToastClose toast={toast} title={title} index={index} onClose={onClose} />
      </ToastHeader>
    </ToastRoot>
  );
}
