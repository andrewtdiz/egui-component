import { isDisabled, type NodeProps, nodeProps, textFromChildren } from "../component-support.ts";
import { buttonVariants } from "../lib/styles.ts";

export type ButtonProps = NodeProps & {
  variant?: string;
  size?: string;
  selected?: boolean;
  label?: string;
  leadingIcon?: string;
  trailingIcon?: string;
  trailingText?: string;
  iconOnly?: boolean;
} & Record<string, unknown>;

export function buttonClassName({ variant = "secondary", size = "md", selected = false, disabled = false, iconOnly = false, className }: { variant?: string; size?: string; selected?: boolean; disabled?: boolean; iconOnly?: boolean; className?: string }) {
  return buttonVariants({ variant, size, selected, disabled, iconOnly, className });
}

export function Button({ children, className, variant = "secondary", size = "md", selected = false, iconOnly = false, label, ...props }: ButtonProps) {
  const disabled = isDisabled(props);
  const resolvedLabel = label ?? textFromChildren(children) ?? (iconOnly ? "" : undefined);
  const resolvedSize = size === "sm" || size === "md" ? size : undefined;
  return (
    <button
      data-slot="button"
      {...nodeProps(props, buttonClassName({ variant, size, selected, disabled, iconOnly, className }))}
      variant={variant}
      size={resolvedSize}
      selected={selected}
      iconOnly={iconOnly}
      label={resolvedLabel}
    >
      {children}
    </button>
  );
}
