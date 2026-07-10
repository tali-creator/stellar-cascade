import type { ButtonHTMLAttributes } from 'react';
import { C, FONT_MONO } from '@/lib/tokens';

type Variant = 'primary' | 'secondary' | 'ghost' | 'danger';

interface FormButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  icon?: React.ElementType;
}

/** Button styled with the Cascade green design tokens.
 *  Kept separate from ui/button.tsx which uses the landing-page teal palette. */
export function FormButton({
  children,
  variant = 'primary',
  disabled,
  icon: Icon,
  style,
  type = 'button',
  ...rest
}: FormButtonProps) {
  const variants: Record<Variant, React.CSSProperties> = {
    primary: {
      background: disabled ? C.borderLight : C.green,
      color: '#06120B',
      fontWeight: 600,
    },
    secondary: {
      background: 'transparent',
      color: C.text,
      border: `1px solid ${C.borderLight}`,
    },
    ghost: { background: 'transparent', color: C.textDim },
    danger: {
      background: 'transparent',
      color: C.red,
      border: `1px solid ${C.redFaint}`,
    },
  };

  return (
    <button
      type={type}
      disabled={disabled}
      className="inline-flex items-center gap-2 rounded-md text-[13px] tracking-[0.02em] px-5 py-[11px] transition-all duration-150 border border-transparent"
      style={{
        fontFamily: FONT_MONO,
        cursor: disabled ? 'not-allowed' : 'pointer',
        opacity: disabled ? 0.4 : 1,
        ...variants[variant],
        ...style,
      }}
      {...rest}
    >
      {Icon && <Icon size={14} />}
      {children}
    </button>
  );
}
