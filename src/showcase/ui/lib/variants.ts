import { cn, type ClassValue } from "./cn.ts";

type VariantSelection = Record<string, string | boolean | number | null | undefined>;
type VariantConfig = {
  variants?: Record<string, Record<string, ClassValue>>;
  defaultVariants?: VariantSelection;
  compoundVariants?: Array<VariantSelection & { className?: ClassValue; class?: ClassValue }>;
};

export function cva(base: ClassValue, config: VariantConfig = {}) {
  return (selection: VariantSelection & { className?: ClassValue; class?: ClassValue } = {}) => {
    const out: ClassValue[] = [base];
    const variants = config.variants ?? {};
    const defaults = config.defaultVariants ?? {};

    for (const [name, values] of Object.entries(variants)) {
      const selected = selection[name] ?? defaults[name];
      if (selected == null) continue;
      out.push(values[String(selected)]);
    }

    for (const compound of config.compoundVariants ?? []) {
      const classValue = compound.className ?? compound.class;
      const matches = Object.entries(compound).every(([name, value]) => {
        if (name === "className" || name === "class") return true;
        return (selection[name] ?? defaults[name]) === value;
      });
      if (matches) out.push(classValue);
    }

    out.push(selection.className, selection.class);
    return cn(out);
  };
}
