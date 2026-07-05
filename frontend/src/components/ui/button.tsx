

import Link from "next/link";
import type { AnchorHTMLAttributes, ButtonHTMLAttributes } from "react";
import { cn } from "@/lib/utils"

type Variant = "primary" | "secondary" | "ghost";
type Size = "md" | "lg";

const base =
  "inline-flex min-h-11 items-center justify-center gap-2 rounded-lg font-medium " +
  "transition-colors duration-200 focus-visible:outline-none focus-visible:ring-2 " +
  "focus-visible:ring-[#2DD4BF] focus-visible:ring-offset-2 focus-visible:ring-offset-[#0A0E17] " +
  "disabled:cursor-not-allowed disabled:opacity-50 motion-reduce:transition-none";

const variants: Record<Variant, string> = {
  primary:
    "bg-[#14B8A6] text-[#0A0E17] hover:bg-[#0D9488] active:bg-[#0B7A6F]",
  secondary:
    "border border-[#29394D] bg-transparent text-[#F1F5F9] hover:border-[#3A4C63] hover:bg-[#111827]",
  ghost:
    "bg-transparent text-[#8B98AC] hover:text-[#F1F5F9]",
};

const sizes: Record<Size, string> = {
  md: "px-5 py-2.5 text-sm",
  lg: "px-6 py-3 text-base",
};

interface ButtonOwnProps {
  variant?: Variant;
  size?: Size;
  className?: string;
}

/** Renders a real <button> for in-page actions. */
export function Button({
  variant = "primary",
  size = "md",
  className,
  ...props
}: ButtonOwnProps & ButtonHTMLAttributes<HTMLButtonElement>) {
  return (
    <button
      className={cn(base, variants[variant], sizes[size], className)}
      {...props}
    />
  );
}

/** Renders a styled <Link> for navigational CTAs (e.g. "Read the Docs"). */
export function LinkButton({
  variant = "primary",
  size = "md",
  className,
  href,
  ...props
}: ButtonOwnProps &
  AnchorHTMLAttributes<HTMLAnchorElement> & { href: string }) {
  return (
    <Link
      href={href}
      className={cn(base, variants[variant], sizes[size], className)}
      {...props}
    />
  );
}
