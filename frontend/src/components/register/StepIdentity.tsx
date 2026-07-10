import { Check, X, Loader2 } from 'lucide-react';
import { C, FONT_MONO, FONT_SANS } from '@/lib/tokens';
import { Field } from './Field';
import { inputStyle } from './inputStyle';
import type { Availability } from './useRegisterForm';

interface StepIdentityProps {
  projectName: string;
  onProjectNameChange: (name: string) => void;
  projectId: string;
  availability: Availability;
  ownerAddress: string;
}

export function StepIdentity({
  projectName,
  onProjectNameChange,
  projectId,
  availability,
  ownerAddress,
}: StepIdentityProps) {
  return (
    <div>
      <Field
        label="Project name"
        hint="Human-readable identifier, e.g. cascade-core. Hashed client-side (SHA-256) into the on-chain project ID."
      >
        <div className="relative">
          <input
            style={inputStyle(availability === 'taken')}
            value={projectName}
            onChange={(e) => onProjectNameChange(e.target.value)}
            placeholder="my-open-source-project"
          />
          <div className="absolute right-3 top-1/2 -translate-y-1/2">
            {availability === 'checking' && (
              <Loader2
                size={15}
                color={C.textFaint}
                style={{ animation: 'spin 1s linear infinite' }}
              />
            )}
            {availability === 'available' && <Check size={15} color={C.green} />}
            {availability === 'taken' && <X size={15} color={C.red} />}
          </div>
        </div>
        {availability === 'available' && (
          <div className="text-xs mt-1.5" style={{ fontFamily: FONT_SANS, color: C.green }}>
            Available
          </div>
        )}
        {availability === 'taken' && (
          <div className="text-xs mt-1.5" style={{ fontFamily: FONT_SANS, color: C.red }}>
            Already taken — try another name
          </div>
        )}
      </Field>

      {projectId && (
        <Field label="Project ID (derived)">
          <div
            className="text-xs rounded-md px-3 py-2.5 break-all"
            style={{
              fontFamily: FONT_MONO,
              color: C.textDim,
              background: C.input,
              border: `1px solid ${C.border}`,
            }}
          >
            {projectId}
          </div>
        </Field>
      )}

      <Field label="Owner">
        <div
          className="flex justify-between items-center rounded-md px-3 py-2.5"
          style={{
            fontFamily: FONT_MONO,
            fontSize: 12,
            color: C.textDim,
            background: C.input,
            border: `1px solid ${C.border}`,
          }}
        >
          <span className="break-all">{ownerAddress}</span>
          <span
            className="text-[10px] tracking-wider shrink-0 ml-2.5"
            style={{ color: C.greenDim }}
          >
            CONNECTED WALLET
          </span>
        </div>
      </Field>
    </div>
  );
}
