import { Lock, Unlock, AlertTriangle } from 'lucide-react';
import { C, FONT_MONO } from '@/lib/tokens';
import type { Mutability } from './useRegisterForm';

interface StepMutabilityProps {
  mutability: Mutability;
  onToggle: () => void;
}

export function StepMutability({ mutability, onToggle }: StepMutabilityProps) {
  const isLocked = mutability === 'locked';

  return (
    <div>
      {/* Toggle row */}
      <div
        className="flex items-center justify-between gap-4 rounded-lg p-4 sm:p-[18px] mb-3.5"
        style={{ background: C.panelRaised, border: `1px solid ${C.border}` }}
      >
        <div className="flex gap-3 items-center min-w-0">
          <div className="shrink-0">
            {isLocked ? (
              <Lock size={18} color={C.gold} />
            ) : (
              <Unlock size={18} color={C.green} />
            )}
          </div>
          <div className="min-w-0">
            <div className="text-[13px] leading-snug" style={{ fontFamily: FONT_MONO, color: C.text }}>
              {isLocked
                ? 'Locked permanently after registration'
                : 'Allow future updates to this split'}
            </div>
            <div className="text-xs mt-0.5" style={{ color: C.textFaint }}>
              {isLocked
                ? 'No further changes, ever'
                : 'Default — matches current contract behavior'}
            </div>
          </div>
        </div>

        {/* Toggle switch */}
        <button
          onClick={onToggle}
          className="w-[46px] h-[26px] rounded-full relative shrink-0 cursor-pointer"
          style={{
            border: 'none',
            background: isLocked ? C.goldFaint : C.greenFaint,
            outline: `1px solid ${isLocked ? C.gold : C.green}`,
          }}
          aria-label={isLocked ? 'Unlock splits' : 'Lock splits'}
        >
          <div
            className="w-[18px] h-[18px] rounded-full absolute top-[3px] transition-[left] duration-150"
            style={{
              left: isLocked ? 24 : 4,
              background: isLocked ? C.gold : C.green,
            }}
          />
        </button>
      </div>

      {/* Irreversibility warning */}
      {isLocked && (
        <div
          className="flex gap-2.5 rounded-lg px-3.5 py-3 mb-3.5"
          style={{ background: C.redFaint, border: 'rgba(239,97,81,0.3)' }}
        >
          <AlertTriangle size={16} color={C.red} className="shrink-0 mt-px" />
          <div className="text-[12.5px] leading-relaxed" style={{ color: C.text }}>
            This cannot be undone — once locked, you will not be able to change receivers or
            percentages, even as the owner.
          </div>
        </div>
      )}

      {/* On-chain disclaimer */}
      <div
        className="flex gap-2.5 rounded-lg px-3.5 py-3"
        style={{ background: C.goldFaint, border: `1px solid rgba(245,185,66,0.25)` }}
      >
        <AlertTriangle size={16} color={C.gold} className="shrink-0 mt-px" />
        <div className="text-xs leading-relaxed" style={{ color: C.textDim }}>
          <strong style={{ color: C.gold }}>Not yet wired on-chain.</strong> The current{' '}
          <code style={{ fontFamily: FONT_MONO }}>update_splits</code> function is always
          available to the owner — there's no lock mechanism yet. This toggle is a UI placeholder
          until a <code style={{ fontFamily: FONT_MONO }}>lock_project</code> function and an{' '}
          <code style={{ fontFamily: FONT_MONO }}>is_locked</code> check ship.
        </div>
      </div>
    </div>
  );
}
