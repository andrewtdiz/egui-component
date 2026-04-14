import { useState } from "egui";
import { cn, itemId, itemLabel, type Handler, type Item, type NodeProps, nodeProps } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";
import { DropdownMenuContent, type MenuEntry } from "./dropdown-menu.tsx";

export type MenuBarProps = NodeProps & { menus?: Item[]; activeMenuId?: string; activeMenuIndex?: number } & Record<string, unknown>;

export function MenuBarRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col gap-1", className))}>{children}</div>;
}

export function MenuBarList({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <Card {...props} className={cn("bg-card border border-border rounded-md shadow-sm", className)} paddingX={4} paddingY={2}>
      <div className="flex flex-row items-center gap-px">{children}</div>
    </Card>
  );
}

export function MenuBarTrigger({ children, active = false, className, ...props }: NodeProps & { active?: boolean } & Record<string, unknown>) {
  return (
    <Button
      {...props}
      variant="ghost"
      selected={active}
      size="sm"
      className={cn("h-7 px-2", className)}
    >
      {children}
    </Button>
  );
}

export function MenuBarContent({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return (
    <Card {...nodeProps(props, cn("bg-popover border border-border rounded-md shadow-md", className))} paddingX={4} paddingY={4}>
      {children}
    </Card>
  );
}

export function MenuBar({ menus = [], activeMenuId, activeMenuIndex, className, onCommand, onOpen, onClose, ...props }: MenuBarProps) {
  const baseId = props.id ?? props.nodeId ?? "menu-bar";
  const [internalActive, setInternalActive] = useState<string | null>(null);
  const controlledId = activeMenuId ?? (activeMenuIndex != null && menus[activeMenuIndex] != null ? itemId(menus[activeMenuIndex], String(activeMenuIndex)) : null);
  const resolvedActive = controlledId ?? internalActive;
  const activeMenu = resolvedActive == null ? null : menus.find((menu, index) => itemId(menu, String(index)) === resolvedActive) ?? null;

  const toggleMenu = (id: string, event: unknown) => {
    const next = resolvedActive === id ? null : id;
    if (activeMenuId == null && activeMenuIndex == null) {
      setInternalActive(next);
    }
    if (next == null) onClose?.(event, id);
    else onOpen?.(event, id);
  };

  const closeMenu = () => {
    if (activeMenuId == null && activeMenuIndex == null) {
      setInternalActive(null);
    }
  };

  return (
    <MenuBarRoot {...props} className={className}>
      <MenuBarList id={`${baseId}-triggers`}>
        {menus.map((menu, index) => {
          const id = itemId(menu, String(index));
          const label = itemLabel(menu, id);
          const active = resolvedActive === id;
          return (
            <MenuBarTrigger
              key={id}
              id={`${baseId}-${id}-trigger`}
              actionId={String(menu.actionId ?? menu.menuId ?? menu.itemId ?? id)}
              active={active}
              onClick={(event) => toggleMenu(id, event)}
            >
              {label}
            </MenuBarTrigger>
          );
        })}
      </MenuBarList>
      {activeMenu != null && Array.isArray(activeMenu.entries) && (
        <MenuBarContent id={`${baseId}-content`}>
          <DropdownMenuContent id={`${baseId}-items`} entries={activeMenu.entries as MenuEntry[]} onCommand={onCommand as Handler | undefined} onClose={closeMenu} />
        </MenuBarContent>
      )}
    </MenuBarRoot>
  );
}
