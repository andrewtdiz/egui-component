import { cn, type NodeProps, nodeProps } from "../component-support.ts";

export function Slider({ value = 0, className, ...props }: NodeProps & Record<string, unknown>) {
  return <input data-slot="slider" type="range" {...nodeProps(props, className)} value={value} />;
}

export function NumberInput({ value = 0, className, ...props }: NodeProps & Record<string, unknown>) {
  return <input data-slot="number-input" type="number" {...nodeProps(props, className)} value={value} />;
}
