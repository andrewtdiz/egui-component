import { cva } from "./variants.ts";

export const styles = {
  surface: "bg-card border border-border rounded-lg",
  surfaceMuted: "bg-muted border border-border rounded-md",
  title: "text-lg font-semibold text-foreground",
  subtitle: "text-sm text-muted-foreground",
  label: "text-sm font-medium text-foreground",
  muted: "text-sm text-muted-foreground",
  badge: "text-xs font-medium text-primary",
  destructive: "text-sm font-semibold text-destructive",
  actionLabel: "font-medium",
} as const;

export const buttonVariants = cva("font-medium", {
  variants: {
    variant: {
      primary: "text-primary-foreground",
      secondary: "text-secondary-foreground",
      ghost: "text-foreground",
      link: "text-primary",
    },
    size: {
      sm: "",
      md: "",
      lg: "",
    },
    selected: {
      true: "font-semibold text-primary",
      false: "",
    },
    disabled: {
      true: "text-muted-foreground opacity-50",
      false: "",
    },
    iconOnly: {
      true: "p-0",
      false: "",
    },
  },
  defaultVariants: {
    variant: "secondary",
    size: "md",
    selected: false,
    disabled: false,
    iconOnly: false,
  },
});

export const labelVariants = cva("text-sm", {
  variants: {
    tone: {
      primary: "text-foreground",
      secondary: "text-secondary-foreground",
      muted: "text-muted-foreground",
      destructive: "text-destructive",
    },
    weight: {
      regular: "font-normal",
      medium: "font-medium",
      semibold: "font-semibold",
      bold: "font-bold",
    },
    size: {
      xs: "text-xs",
      sm: "text-sm",
      base: "text-base",
      lg: "text-lg",
      xl: "text-xl",
    },
  },
  defaultVariants: {
    tone: "primary",
    weight: "medium",
    size: "sm",
  },
});

export const surfaceVariants = cva("border border-border", {
  variants: {
    variant: {
      card: "bg-card rounded-lg",
      muted: "bg-muted rounded-md",
      plain: "bg-transparent border-transparent rounded-md",
    },
  },
  defaultVariants: { variant: "card" },
});

export const controlVariants = cva("text-sm text-foreground", {
  variants: {
    disabled: {
      true: "text-muted-foreground opacity-50",
      false: "",
    },
  },
  defaultVariants: { disabled: false },
});

export const componentPreviewStyles = {
  shell: "bg-card border border-border rounded-lg",
  nested: "bg-muted border border-border rounded-md",
  caption: "text-xs text-muted-foreground",
} as const;
