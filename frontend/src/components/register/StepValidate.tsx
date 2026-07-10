import { C, FONT_MONO, FONT_SANS } from '@/lib/tokens';
import { pctToBps } from './useRegisterForm';
import type { ReceiverRow } from './useRegisterForm';

interface StepValidateProps {
  receivers: ReceiverRow[];
}

export function StepValidate({ receivers }: StepValidateProps) {
  return (
    <div>
      <div className="text-xs mb-4" style={{ fontFamily: FONT_SANS, color: C.textDim }}>
        Review before locking in the split table.
      </div>
      <table className="w-full border-collapse mb-5">
        <thead>
          <tr>
            {['Receiver', 'Share', 'Basis points'].map((h) => (
              <th
                key={h}
                className="text-left text-[10px] tracking-[0.06em] uppercase px-2 py-1.5"
                style={{
                  fontFamily: FONT_MONO,
                  color: C.textFaint,
                  borderBottom: `1px solid ${C.border}`,
                }}
              >
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {receivers.map((r) => (
            <tr key={r.id}>
              <td
                className="p-2 text-xs"
                style={{
                  fontFamily: FONT_MONO,
                  color: C.text,
                  borderBottom: `1px solid ${C.border}`,
                }}
              >
                {r.address ? `${r.address.slice(0, 6)}…${r.address.slice(-4)}` : '—'}
              </td>
              <td
                className="p-2 text-xs"
                style={{
                  fontFamily: FONT_MONO,
                  color: C.text,
                  borderBottom: `1px solid ${C.border}`,
                }}
              >
                {r.percent || 0}%
              </td>
              <td
                className="p-2 text-xs"
                style={{
                  fontFamily: FONT_MONO,
                  color: C.textDim,
                  borderBottom: `1px solid ${C.border}`,
                }}
              >
                {pctToBps(r.percent) ?? '—'}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
