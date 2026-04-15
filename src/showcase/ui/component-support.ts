import { cn } from "./lib/cn.ts";

export type Children = unknown;
export type Handler = (event: unknown, value?: unknown) => void;
export type Item = Record<string, unknown>;

export type NodeProps = {
  id?: string;
  nodeId?: string;
  className?: string;
  classNames?: string;
  classList?: string[];
  slotClasses?: Record<string, string>;
  slotClassNames?: Record<string, string>;
  visible?: boolean;
  enabled?: boolean;
  disabled?: boolean;
  actionId?: string;
  children?: Children;
  onClick?: Handler;
  onChange?: Handler;
  onSubmit?: Handler;
  onSelect?: Handler;
  onToggle?: Handler;
  onConfirm?: Handler;
  onCancel?: Handler;
  onOpen?: Handler;
  onClose?: Handler;
  onCommand?: Handler;
};

export function nodeProps(props: NodeProps & Record<string, unknown>, className?: string) {
  const {
    id,
    nodeId,
    className: propClassName,
    classNames,
    classList,
    slotClasses,
    slotClassNames,
    visible,
    enabled,
    disabled,
    children: _children,
    ...rest
  } = props;
  const attrs: Record<string, unknown> = { ...rest };
  const resolvedId = nodeId ?? id;
  const resolvedClassName = cn(className, propClassName, classNames);
  const resolvedSlotClasses = { ...(slotClasses ?? {}), ...(slotClassNames ?? {}) };

  if (resolvedId != null) attrs.id = resolvedId;
  if (resolvedClassName !== "") attrs.className = resolvedClassName;
  if (classList != null) attrs.classList = classList;
  if (Object.keys(resolvedSlotClasses).length > 0) attrs.slotClasses = resolvedSlotClasses;
  if (visible != null) attrs.visible = visible;
  if (disabled === true) attrs.enabled = false;
  else if (enabled != null) attrs.enabled = enabled;

  return attrs;
}

export function resolveNodeId(props: { id?: string; nodeId?: string }) {
  const value = props.nodeId ?? props.id;
  if (value == null) return undefined;
  const nodeId = String(value).trim();
  return nodeId === "" ? undefined : nodeId;
}

export function requireNodeId(props: { id?: string; nodeId?: string }, componentName: string) {
  const nodeId = resolveNodeId(props);
  if (nodeId == null) {
    throw new Error(`${componentName} requires an explicit id or nodeId.`);
  }
  return nodeId;
}

export function isDisabled(props: { disabled?: boolean; enabled?: boolean }) {
  return props.disabled === true || props.enabled === false;
}

export function boolValue(value: boolean | undefined, checked: boolean | undefined, fallback = false) {
  return value ?? checked ?? fallback;
}

export function itemId(item: Item, fallback: string) {
  const value = item.itemId ?? item.item_id ?? item.id ?? fallback;
  return String(value);
}

export function itemLabel(item: Item, fallback: string) {
  const value = item.label ?? item.name ?? item.title ?? fallback;
  return String(value);
}

export function textFromChildren(children: Children): string | undefined {
  if (children == null || children === false || children === true) return undefined;
  if (Array.isArray(children)) {
    const text = children.map(textFromChildren).filter(Boolean).join(" ");
    return text === "" ? undefined : text;
  }
  if (typeof children === "string" || typeof children === "number") {
    const text = String(children).trim();
    return text === "" ? undefined : text;
  }
  return undefined;
}

export function scopedId(props: { id?: string; nodeId?: string }, fallback: string, suffix: string) {
  return `${props.nodeId ?? props.id ?? fallback}-${suffix}`;
}

export function requiredScopedId(
  props: { id?: string; nodeId?: string },
  componentName: string,
  suffix: string,
) {
  return `${requireNodeId(props, componentName)}-${suffix}`;
}

export { cn };
