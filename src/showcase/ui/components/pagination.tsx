import { cn, eventWithValue, type Handler, type NodeProps, nodeProps, requireNodeId } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Label } from "./primary/label.tsx";

type PaginationItem = number | "ellipsis";

function paginationItems(currentPage: number, pageCount: number, siblingCount: number): PaginationItem[] {
  if (pageCount <= 0) return [];
  const current = Math.min(Math.max(Math.round(currentPage || 1), 1), pageCount);
  const visiblePageSlots = siblingCount * 2 + 5;
  if (pageCount <= visiblePageSlots) return Array.from({ length: pageCount }, (_, index) => index + 1);

  const leftSibling = Math.max(current - siblingCount, 1);
  const rightSibling = Math.min(current + siblingCount, pageCount);
  const showLeftEllipsis = leftSibling > 2;
  const showRightEllipsis = rightSibling < pageCount - 1;

  if (!showLeftEllipsis && showRightEllipsis) {
    const lastLeftPage = Math.min(siblingCount * 2 + 2, pageCount - 1);
    return [...Array.from({ length: lastLeftPage }, (_, index) => index + 1), "ellipsis", pageCount];
  }
  if (showLeftEllipsis && !showRightEllipsis) {
    const startPage = Math.max(pageCount - (siblingCount * 2 + 1), 2);
    return [1, "ellipsis", ...Array.from({ length: pageCount - startPage + 1 }, (_, index) => startPage + index)];
  }
  if (showLeftEllipsis && showRightEllipsis) {
    return [1, "ellipsis", ...Array.from({ length: rightSibling - leftSibling + 1 }, (_, index) => leftSibling + index), "ellipsis", pageCount];
  }
  return Array.from({ length: pageCount }, (_, index) => index + 1);
}

export function PaginationRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-row flex-wrap items-center gap-1", className))}>{children}</div>;
}

export function PaginationPrevious({ currentPage, baseId = "pagination", onSelect }: { currentPage: number; baseId?: string; onSelect?: Handler<number> }) {
  if (currentPage <= 1) return null;
  return <Button id={`${baseId}-previous`} size="sm" variant="ghost" leadingIcon="chevron-left" className="shrink-0" onClick={(event) => onSelect?.(eventWithValue(event, currentPage - 1, { page: currentPage - 1 }), currentPage - 1)}>Previous</Button>;
}

export function PaginationNext({ currentPage, pageCount, baseId = "pagination", onSelect }: { currentPage: number; pageCount: number; baseId?: string; onSelect?: Handler<number> }) {
  if (currentPage >= pageCount) return null;
  return <Button id={`${baseId}-next`} size="sm" variant="ghost" trailingIcon="chevron-right" className="shrink-0" onClick={(event) => onSelect?.(eventWithValue(event, currentPage + 1, { page: currentPage + 1 }), currentPage + 1)}>Next</Button>;
}

export function PaginationEllipsis({ index, baseId = "pagination" }: { index: number; baseId?: string }) {
  return <Label key={`ellipsis-${index}`} id={`${baseId}-ellipsis-${index}`} text="..." tone="muted" className="px-2 shrink-0 text-sm text-muted-foreground" />;
}

export function PaginationPage({ page, currentPage, baseId = "pagination", onSelect }: { page: number; currentPage: number; baseId?: string; onSelect?: Handler<number> }) {
  return (
    <Button
      id={`${baseId}-page-${page}`}
      size="sm"
      variant={page === currentPage ? "secondary" : "ghost"}
      selected={page === currentPage}
      width={32}
      className="h-8 shrink-0"
      onClick={(event) => onSelect?.(eventWithValue(event, page, { page }), page)}
    >
      {String(page)}
    </Button>
  );
}

export function PaginationContent({ currentPage = 1, pageCount = 1, siblingCount = 1, baseId = "pagination", onSelect }: { currentPage?: number; pageCount?: number; siblingCount?: number; baseId?: string; onSelect?: Handler<number> }) {
  const current = Math.min(Math.max(Math.round(currentPage || 1), 1), Math.max(pageCount, 1));
  const pages = paginationItems(current, Math.max(pageCount, 0), Math.max(siblingCount, 0));

  return (
    <>
      <PaginationPrevious currentPage={current} baseId={baseId} onSelect={onSelect} />
      {pages.map((page, index) => (page === "ellipsis" ? <PaginationEllipsis key={`ellipsis-${index}`} index={index} baseId={baseId} /> : <PaginationPage key={page} page={page} currentPage={current} baseId={baseId} onSelect={onSelect} />))}
      <PaginationNext currentPage={current} pageCount={pageCount} baseId={baseId} onSelect={onSelect} />
    </>
  );
}

export function Pagination({ currentPage = 1, pageCount = 1, siblingCount = 1, className, onSelect, ...props }: NodeProps & { currentPage?: number; pageCount?: number; siblingCount?: number; onSelect?: Handler<number> } & Record<string, unknown>) {
  const baseId = requireNodeId(props, "Pagination");
  return (
    <PaginationRoot {...props} className={className}>
      <PaginationContent currentPage={currentPage} pageCount={pageCount} siblingCount={siblingCount} baseId={baseId} onSelect={onSelect} />
    </PaginationRoot>
  );
}
