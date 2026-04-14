import { cn, type NodeProps, nodeProps, requireNodeId } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";
import { Label } from "./primary/label.tsx";
import { Separator } from "./primary/separator.tsx";

export type SidebarProps = NodeProps & {
  open?: boolean;
  title?: string;
  side?: string;
} & Record<string, unknown>;

export function SidebarRoot({
  children,
  side = "left",
  className,
  ...props
}: NodeProps & { side?: string } & Record<string, unknown>) {
  return (
    <Card
      {...nodeProps(
        props,
        cn(
          "bg-sidebar border-sidebar-border rounded-lg shadow-lg",
          side === "right" && "ml-auto",
          className,
        ),
      )}
      width={props.width ?? 280}
      paddingX={12}
      paddingY={12}
    >
      {children}
    </Card>
  );
}

export function SidebarHeader({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <div {...nodeProps(props, cn("flex flex-row items-center gap-3", className))}>
      {children}
    </div>
  );
}

export function SidebarTitle({ text = "Sidebar", className, ...props }: NodeProps & { text?: string } & Record<string, unknown>) {
  return (
    <Label
      {...props}
      text={text}
      weight="semibold"
      size="base"
      className={cn("text-sidebar-foreground text-base", className)}
    />
  );
}

export function SidebarClose({
  className,
  onClose,
  ...props
}: NodeProps & Record<string, unknown>) {
  return (
    <Button
      {...props}
      variant="ghost"
      iconOnly
      leadingIcon="x"
      size="sm"
      className={cn("h-7 w-7", className)}
      onClick={onClose}
    />
  );
}

export function SidebarBody({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <div {...nodeProps(props, cn("flex flex-col gap-2", className))}>
      {children}
    </div>
  );
}

export function SidebarFooter({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <div {...nodeProps(props, cn("flex flex-row items-center justify-end gap-2", className))}>
      {children}
    </div>
  );
}

export function SidebarContent({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <SidebarBody {...props} className={className}>{children}</SidebarBody>;
}

export function Sidebar({
  children,
  open = false,
  title = "Sidebar",
  side = "left",
  className,
  onClose,
  ...props
}: SidebarProps) {
  if (!open) return null;
  const baseId = requireNodeId(props, "Sidebar");
  return (
    <SidebarRoot {...props} side={side} className={className}>
      <div className="flex flex-col items-stretch gap-2.5">
        {title != null && title !== "" && (
          <div className="flex flex-col items-stretch gap-2.5">
            <SidebarHeader id={`${baseId}-header`}>
              <SidebarTitle text={title} />
              <div data-slot="spacer" />
              <SidebarClose id={`${baseId}-close`} onClose={onClose} />
            </SidebarHeader>
            <Separator id={`${baseId}-separator`} className="bg-sidebar-border" />
          </div>
        )}
        <SidebarBody id={`${baseId}-content`}>{children}</SidebarBody>
      </div>
    </SidebarRoot>
  );
}
