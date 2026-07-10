import { Check } from 'lucide-react';
import { C, FONT_MONO } from '@/lib/tokens';

interface StepDotProps {
  n: number;
  label: string;
  active: boolean;
  done: boolean;
}

function StepDot({ n, label, active, done }: StepDotProps) {
  return (
    <div className="flex items-center gap-2.5 flex-1">
      <div
        className="w-7 h-7 rounded-full flex items-center justify-center shrink-0 text-xs transition-all duration-200"
        style={{
          fontFamily: FONT_MONO,
          border: `1px solid ${done || active ? C.green : C.border}`,
          background: done ? C.green : active ? C.greenFaint : 'transparent',
          color: done ? '#06120B' : active ? C.green : C.textFaint,
        }}
      >
        {done ? <Check size={14} /> : n}
      </div>
      <span
        className="text-[11px] tracking-[0.04em] uppercase hidden sm:block"
        style={{
          fontFamily: FONT_MONO,
          color: active ? C.text : done ? C.textDim : C.textFaint,
        }}
      >
        {label}
      </span>
    </div>
  );
}

interface StepIndicatorProps {
  steps: { n: number; label: string }[];
  currentStep: number;
}

export function StepIndicator({ steps, currentStep }: StepIndicatorProps) {
  return (
    <div className="flex mb-9 gap-1">
      {steps.map((s) => (
        <StepDot
          key={s.n}
          n={s.n}
          label={s.label}
          active={currentStep === s.n}
          done={currentStep > s.n}
        />
      ))}
    </div>
  );
}
