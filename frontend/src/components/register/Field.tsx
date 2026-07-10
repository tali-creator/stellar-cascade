import { AlertTriangle } from 'lucide-react';
import { C, FONT_MONO, FONT_SANS } from '@/lib/tokens';

interface FieldProps {
  label: string;
  hint?: string;
  error?: string;
  children: React.ReactNode;
}

export function Field({ label, hint, error, children }: FieldProps) {
  return (
    <div className="mb-5">
      <label
        className="block text-[11px] tracking-wider uppercase mb-2"
        style={{ fontFamily: FONT_MONO, color: C.textDim }}
      >
        {label}
      </label>
      {children}
      {hint && !error && (
        <div className="text-xs mt-1.5" style={{ fontFamily: FONT_SANS, color: C.textFaint }}>
          {hint}
        </div>
      )}
      {error && (
        <div
          className="flex items-center gap-1 text-xs mt-1.5"
          style={{ fontFamily: FONT_SANS, color: C.red }}
        >
          <AlertTriangle size={12} />
          {error}
        </div>
      )}
    </div>
  );
}
