import { CheckCircle2, Check, Copy, ExternalLink } from 'lucide-react';
import { C, FONT_MONO, FONT_SANS } from '@/lib/tokens';
import type { ReceiverRow, SuccessResult } from './useRegisterForm';

interface RegisterSuccessProps {
  success: SuccessResult;
  projectName: string;
  receivers: ReceiverRow[];
  copied: boolean;
  onCopyProjectId: () => void;
}

export function RegisterSuccess({
  success,
  projectName,
  receivers,
  copied,
  onCopyProjectId,
}: RegisterSuccessProps) {
  return (
    <div
      className="rounded-xl p-5 sm:p-8 text-center"
      style={{ background: C.panel, border: `1px solid ${C.borderLight}` }}
    >
      {/* Icon */}
      <div
        className="w-[52px] h-[52px] rounded-full flex items-center justify-center mx-auto mb-[18px]"
        style={{ background: C.greenFaint }}
      >
        <CheckCircle2 size={26} color={C.green} />
      </div>

      <h2 className="text-[18px] m-0 mb-1.5" style={{ fontFamily: FONT_MONO }}>
        Project registered
      </h2>
      <div className="text-[13px] mb-6" style={{ fontFamily: FONT_SANS, color: C.textDim }}>
        {projectName}
      </div>

      {/* Project ID card */}
      <div
        className="text-left rounded-lg p-4 mb-4"
        style={{ background: C.panelRaised, border: `1px solid ${C.border}` }}
      >
        <div
          className="text-[10px] tracking-[0.06em] mb-1.5"
          style={{ fontFamily: FONT_MONO, color: C.textFaint }}
        >
          PROJECT ID
        </div>
        <div className="flex justify-between items-center gap-2">
          <span
            className="text-xs break-all"
            style={{ fontFamily: FONT_MONO, color: C.text }}
          >
            {success.projectId}
          </span>
          <button
            onClick={onCopyProjectId}
            className="shrink-0 cursor-pointer"
            style={{ background: 'none', border: 'none', color: copied ? C.green : C.textFaint }}
            aria-label="Copy project ID"
          >
            {copied ? <Check size={14} /> : <Copy size={14} />}
          </button>
        </div>
      </div>

      {/* Receivers table */}
      <table className="w-full border-collapse mb-5 text-left">
        <thead>
          <tr>
            {['Receiver', 'Share'].map((h) => (
              <th
                key={h}
                className="text-[10px] tracking-[0.06em] px-2 py-1.5"
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
                className="px-2 py-[7px] text-xs"
                style={{
                  fontFamily: FONT_MONO,
                  color: C.text,
                  borderBottom: `1px solid ${C.border}`,
                }}
              >
                {r.address.slice(0, 6)}…{r.address.slice(-4)}
              </td>
              <td
                className="px-2 py-[7px] text-xs"
                style={{
                  fontFamily: FONT_MONO,
                  color: C.text,
                  borderBottom: `1px solid ${C.border}`,
                }}
              >
                {r.percent}%
              </td>
            </tr>
          ))}
        </tbody>
      </table>

      {/* On-chain link placeholder */}
      <div
        className="flex items-center justify-center gap-1.5 text-xs mb-1"
        style={{ fontFamily: FONT_SANS, color: C.textFaint }}
      >
        <ExternalLink size={12} />
        View on-chain — available once a testnet contract ID and get_project read path are wired
        up
      </div>
    </div>
  );
}
