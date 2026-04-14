import { styles } from "../lib/styles.ts";
import { cn, type Handler, type NodeProps, nodeProps, scopedId } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";
import { Image } from "./primary/image.tsx";

export function ImageTileRoot({ children, selected = false, className, ...props }: NodeProps & { selected?: boolean } & Record<string, unknown>) {
  return <Card {...props} className={cn(styles.surfaceMuted, selected && "border-primary bg-accent", className)} paddingX={10} paddingY={10}>{children}</Card>;
}

export function ImageTilePlaybackButton({ playbackState = "paused", className, ...props }: NodeProps & { playbackState?: string } & Record<string, unknown>) {
  return (
    <Button
      {...props}
      id={scopedId(props, "image-tile", "playback")}
      iconOnly
      leadingIcon={playbackState === "playing" ? "pause" : "play"}
      variant="primary"
      size="sm"
      className={cn("h-7 w-7", className)}
      onClick={props.onToggle}
    />
  );
}

export function ImageTileMedia({ source = "builtin:showcase-image", playbackState, className, ...props }: NodeProps & { source?: string; playbackState?: string } & Record<string, unknown>) {
  return (
    <Card {...props} className={cn("bg-card border-border rounded-md", className)} paddingX={0} paddingY={0}>
      <div className="flex flex-col items-stretch gap-0">
        <Image id={scopedId(props, "image-tile", "image")} source={source} width={props.width ?? 144} height={props.height ?? 96} cornerRadius={6} />
        {playbackState != null && (
          <div className="flex flex-row justify-end p-1">
            <ImageTilePlaybackButton playbackState={playbackState} onToggle={props.onToggle} />
          </div>
        )}
      </div>
    </Card>
  );
}

export function ImageTileContent({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-col items-stretch gap-2", className))}>{children}</div>;
}

export function ImageTile({ children, source = "builtin:showcase-image", selected = false, playbackState, className, onClick, onToggle, ...props }: NodeProps & { selected?: boolean; playbackState?: string; onToggle?: Handler } & Record<string, unknown>) {
  return (
    <ImageTileRoot {...props} selected={selected} className={className} onClick={onClick}>
      <ImageTileContent>
        <ImageTileMedia source={source} playbackState={playbackState} width={props.width} height={props.height} onToggle={onToggle} />
        {children}
      </ImageTileContent>
    </ImageTileRoot>
  );
}
