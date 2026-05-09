import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const badgeVariants = cva(
  "inline-flex items-center rounded-md border px-2 py-1 text-[11px] font-medium uppercase tracking-wide",
  {
    variants: {
      variant: {
        default: "border-emerald-300 bg-emerald-50 text-emerald-700",
        secondary: "border-slate-300 bg-slate-100 text-slate-700",
        warning: "border-amber-300 bg-amber-50 text-amber-700",
        danger: "border-rose-300 bg-rose-50 text-rose-700",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

export interface BadgeProps extends React.HTMLAttributes<HTMLDivElement>, VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return <div className={cn(badgeVariants({ variant }), className)} {...props} />;
}

export { Badge, badgeVariants };
