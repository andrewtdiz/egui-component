import { styles } from "../lib/styles.ts";
import { cn, type NodeProps, nodeProps, scopedId } from "../component-support.ts";
import { Button } from "./button.tsx";
import { Card } from "./card.tsx";
import { Progress } from "./primary/progress.tsx";

export type AudioPlaybackProps = NodeProps & { playbackState?: string } & Record<string, unknown>;

export function AudioPlayback({ children, playbackState = "paused", className, onToggle, ...props }: AudioPlaybackProps) {
  const playing = playbackState === "playing" || playbackState === "Playing";
  const value = typeof props.value === "number" ? props.value : playing ? 0.66 : 0.28;

  return (
    <AudioPlaybackRoot {...props} className={className}>
      <AudioPlaybackControls>
        <AudioPlaybackButton id={scopedId(props, "audio-playback", "playback")} playing={playing} onClick={onToggle ?? props.onClick} />
        <AudioPlaybackScrubber id={scopedId(props, "audio-playback", "waveform")} value={value} width={props.width ?? 188} />
        {children}
      </AudioPlaybackControls>
    </AudioPlaybackRoot>
  );
}

export function AudioPlaybackRoot({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <Card {...props} className={cn(styles.surfaceMuted, "rounded-md", className)} paddingX={props.paddingX ?? 8} paddingY={props.paddingY ?? 6}>{children}</Card>;
}

export function AudioPlaybackControls({ children, className, ...props }: NodeProps & Record<string, unknown>) {
  return <div {...nodeProps(props, cn("flex flex-row items-center gap-2.5", className))}>{children}</div>;
}

export function AudioPlaybackButton({ playing = false, className, ...props }: NodeProps & { playing?: boolean } & Record<string, unknown>) {
  return <Button {...props} className={className} iconOnly leadingIcon={playing ? "pause" : "play"} variant="ghost" selected={playing} />;
}

export function AudioPlaybackScrubber({ value = 0, className, ...props }: NodeProps & Record<string, unknown>) {
  return <Progress {...props} value={value} height={props.height ?? 10} className={className} />;
}
