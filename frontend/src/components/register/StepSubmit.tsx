import { Loader2 } from 'lucide-react';
import { C, FONT_MONO, FONT_SANS } from '@/lib/tokens';
import { FormButton } from './FormButton';
import { MAX_RECEIVERS } from './useRegisterForm';
import type { Mutability, Validation } from './useRegisterForm';

interface StepSubmitProps {
  projectName: string;
  receiverCount: number;
  validation: Validation;
  mutability: Mutability;
  canSubmit: boolean;
  submitting: boolean;
  onSubmit: () => void;
}

export function StepSubmit({
  projectName,
  receiverCount,
  validation,
  mutability,
  canSubmit,
  submitting,
  onSubmit,
}: StepSubmitProps) {
  const summary = [
    ['Project', projectName],
    ['Receivers', `${receiverCount} / ${MAX_RECEIVERS}`],
    ['Total share', `${validation.total}%`],
    [
      'Mutability',
      mutability === 'locked' ? 'Locked (UI only, not enforced)' : 'Unlocked',
    ],
  ] as const;

  return (
    <div>
      <div
        className="text-[13px] leading-relaxed mb-5"
        style={{ fontFamily: FONT_SANS, color: C.textDim }}
      >
        Builds the unsigned{' '}
        <code style={{ fontFamily: FONT_MONO }}>register_project</code> transaction and sends it
        to your connected wallet for signing.
      </div>

      {/* Summary card */}
      <div
        className="rounded-lg p-4 mb-5"
        style={{ background: C.panelRaised, border: `1px solid ${C.border}` }}
      >
        {summary.map(([k, v]) => (
          <div
            key={k}
            className="flex justify-between text-[12.5px] py-[5px]"
            style={{ fontFamily: FONT_MONO }}
          >
            <span style={{ color: C.textFaint }}>{k}</span>
            <span style={{ color: C.text }}>{v}</span>
          </div>
        ))}
      </div>

      <FormButton
        variant="primary"
        onClick={onSubmit}
        disabled={!canSubmit || submitting}
        style={{ width: '100%', justifyContent: 'center', padding: '13px 20px' }}
      >
        {submitting ? (
          <>
            <Loader2 size={15} style={{ animation: 'spin 1s linear infinite' }} />
            Waiting for signature…
          </>
        ) : (
          'Sign & submit'
        )}
      </FormButton>
    </div>
  );
}
