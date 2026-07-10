import { Check, AlertTriangle } from 'lucide-react';
import { C, FONT_MONO } from '@/lib/tokens';

interface SummaryChipProps {
  ok: boolean;
  label: string;
}

export function SummaryChip({ ok, label }: SummaryChipProps) {
  return (
    <div
      className="flex items-center gap-1.5 text-[11.5px] px-3 py-1.5 rounded-full"
      style={{
        fontFamily: FONT_MONO,
        border: `1px solid ${ok ? 'rgba(52,211,153,0.3)' : 'rgba(239,97,81,0.3)'}`,
        background: ok ? C.greenFaint : C.redFaint,
        color: ok ? C.green : C.red,
      }}
    >
      {ok ? <Check size={12} /> : <AlertTriangle size={12} />}
      {label}
    </div>
  );
}
