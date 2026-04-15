import { type NodeProps } from "../../component-support.ts";
import { Image } from "./image.tsx";

type TwemojiProps = NodeProps & { emoji?: string; size?: number; width?: number; height?: number } & Record<string, unknown>;

export function Twemoji({ emoji = "🙂", size = 18, width, height, className, ...props }: TwemojiProps) {
  return <Image {...props} className={className} source={`builtin:twemoji:${emoji}`} width={width ?? size} height={height ?? size} />;
}
