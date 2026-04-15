import { type NodeProps, nodeProps, textFromChildren } from "../../component-support.ts";

export function Kbd({ children, text, className, ...props }: NodeProps & { text?: string } & Record<string, unknown>) {
  const resolvedText = text ?? textFromChildren(children) ?? "Ctrl";
  return <kbd data-slot="kbd" {...nodeProps(props, className)} text={resolvedText} />;
}
