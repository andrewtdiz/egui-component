import { styles } from "../lib/styles.ts";
import { cn, type NodeProps, nodeProps } from "../component-support.ts";
import { Button, type ButtonProps } from "./button.tsx";
import { Card } from "./card.tsx";

export function ToolbarRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <Card
      {...nodeProps(props, cn(styles.surface, "rounded-lg shadow-sm", className))}
      paddingX={props.paddingX ?? 8}
      paddingY={props.paddingY ?? 6}
    >
      {children}
    </Card>
  );
}

export function ToolbarContent({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <div {...nodeProps(props, cn("flex flex-row items-center gap-1", className))}>
      {children}
    </div>
  );
}

export function ToolbarButton({ className, variant = "ghost", size = "sm", ...props }: ButtonProps) {
  return <Button {...props} variant={variant} size={size} className={className} />;
}

export function Toolbar({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <ToolbarRoot {...props} className={className}>
      <ToolbarContent>{children}</ToolbarContent>
    </ToolbarRoot>
  );
}
