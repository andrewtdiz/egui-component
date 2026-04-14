import { surfaceVariants } from "../lib/styles.ts";
import { cn, type NodeProps, nodeProps } from "../component-support.ts";
import { LabelDescription, LabelTitle } from "./primary/label.tsx";

export type CardProps = NodeProps & { paddingX?: number; paddingY?: number; variant?: string } & Record<string, unknown>;

export function Card({ children, className, paddingX = 16, paddingY = 16, variant = "card", ...props }: CardProps) {
  return <div data-slot="card" {...nodeProps(props, surfaceVariants({ variant, className }))} paddingX={paddingX} paddingY={paddingY}>{children}</div>;
}

export const CardRoot = Card;

export function CardHeader({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-1.5", className))}>{children}</div>;
}

export function CardTitle(props: NodeProps & { text?: string } & Record<string, unknown>) {
  return <LabelTitle {...props} />;
}

export function CardDescription(props: NodeProps & { text?: string } & Record<string, unknown>) {
  return <LabelDescription {...props} />;
}

export function CardContent({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-2", className))}>{children}</div>;
}

export function CardFooter({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-row items-center justify-end gap-2", className))}>{children}</div>;
}
