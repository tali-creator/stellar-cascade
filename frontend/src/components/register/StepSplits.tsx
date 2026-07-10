import { useRef } from 'react';
import { X, Plus, Upload } from 'lucide-react';
import { C, FONT_MONO, FONT_SANS } from '@/lib/tokens';
import { FormButton } from './FormButton';
import { inputStyle } from './inputStyle';
import type { ReceiverRow, ReceiverTab, Validation } from './useRegisterForm';
import { MAX_RECEIVERS } from './useRegisterForm';

interface StepSplitsProps {
  receivers: ReceiverRow[];
  tab: ReceiverTab;
  onTabChange: (tab: ReceiverTab) => void;
  csvText: string;
  onCsvTextChange: (text: string) => void;
  fileInputRef: React.RefObject<HTMLInputElement | null>;
  validation: Validation;
  onUpdateRow: (id: string, patch: Partial<ReceiverRow>) => void;
  onRemoveRow: (id: string) => void;
  onAddRow: () => void;
  onApplyCsv: () => void;
  onFileUpload: (e: React.ChangeEvent<HTMLInputElement>) => void;
}

export function StepSplits({
  receivers,
  tab,
  onTabChange,
  csvText,
  onCsvTextChange,
  fileInputRef,
  validation,
  onUpdateRow,
  onRemoveRow,
  onAddRow,
  onApplyCsv,
  onFileUpload,
}: StepSplitsProps) {
  return (
    <div>
      {/* Tab bar */}
      <div className="flex gap-1.5 mb-5" style={{ borderBottom: `1px solid ${C.border}` }}>
        {(['manual', 'csv'] as ReceiverTab[]).map((t) => (
          <button
            key={t}
            onClick={() => onTabChange(t)}
            style={{
              fontFamily: FONT_MONO,
              background: 'none',
              border: 'none',
              color: tab === t ? C.green : C.textFaint,
              borderBottom: tab === t ? `2px solid ${C.green}` : '2px solid transparent',
              padding: '10px 4px',
              marginRight: 20,
              marginBottom: -1,
              cursor: 'pointer',
              fontSize: 12,
              letterSpacing: '0.04em',
              textTransform: 'uppercase',
            }}
          >
            {t === 'manual' ? 'Manual entry' : 'CSV upload'}
          </button>
        ))}
      </div>

      {tab === 'manual' && (
        <div>
          {receivers.map((r, i) => {
            const errs = validation.rowErrors[i] || [];
            const isDup =
              !!r.address &&
              validation.duplicateAddrs.has(r.address.trim().toLowerCase());
            return (
              <div key={r.id} className="mb-2.5">
                {/* On mobile: address full width, then percent + remove on same line below */}
                <div className="flex flex-col sm:flex-row gap-2 items-start">
                  <div className="w-full sm:flex-1">
                    <input
                      style={inputStyle(errs.length > 0 || isDup)}
                      placeholder="G... receiver address"
                      value={r.address}
                      onChange={(e) => onUpdateRow(r.id, { address: e.target.value })}
                    />
                  </div>
                  <div className="flex gap-2 items-center w-full sm:w-auto">
                    <div className="w-[90px] relative">
                      <input
                        style={{
                          ...inputStyle(errs.some((e) => e.includes('umeric'))),
                          paddingRight: 24,
                        }}
                        placeholder="0"
                        value={r.percent}
                        onChange={(e) => onUpdateRow(r.id, { percent: e.target.value })}
                      />
                      <span
                        className="absolute right-2.5 top-1/2 -translate-y-1/2 text-xs"
                        style={{ color: C.textFaint }}
                      >
                        %
                      </span>
                    </div>
                    <button
                      onClick={() => onRemoveRow(r.id)}
                      className="cursor-pointer p-2.5 shrink-0"
                      style={{ background: 'none', border: 'none', color: C.textFaint }}
                      title="Remove receiver"
                    >
                      <X size={15} />
                    </button>
                  </div>
                </div>
              </div>
            );
          })}

          {/* Row-level errors */}
          {receivers.some(
            (r, i) =>
              (validation.rowErrors[i] || []).length > 0 ||
              (r.address &&
                validation.duplicateAddrs.has(r.address.trim().toLowerCase()))
          ) && (
            <div className="mt-1 mb-3">
              {receivers.map((r, i) => {
                const errs = validation.rowErrors[i] || [];
                const isDup =
                  !!r.address &&
                  validation.duplicateAddrs.has(r.address.trim().toLowerCase());
                if (!errs.length && !isDup) return null;
                return (
                  <div
                    key={r.id}
                    className="text-[11px] mb-0.5"
                    style={{ fontFamily: FONT_SANS, color: C.red }}
                  >
                    Row {i + 1}:{' '}
                    {[...errs, isDup ? 'Duplicate address' : null]
                      .filter(Boolean)
                      .join(', ')}
                  </div>
                );
              })}
            </div>
          )}

          <FormButton
            variant="secondary"
            icon={Plus}
            onClick={onAddRow}
            disabled={receivers.length >= MAX_RECEIVERS}
          >
            Add receiver
          </FormButton>
        </div>
      )}

      {tab === 'csv' && (
        <div>
          <div
            className="rounded-lg p-5 text-center mb-3.5 cursor-pointer"
            style={{ border: `1px dashed ${C.borderLight}`, background: C.input }}
            onClick={() => fileInputRef.current?.click()}
          >
            <Upload size={18} color={C.textDim} className="mb-2 mx-auto" />
            <div className="text-xs" style={{ fontFamily: FONT_SANS, color: C.textDim }}>
              Click to upload a .csv, or paste rows below
            </div>
            <input
              ref={fileInputRef}
              type="file"
              accept=".csv,text/csv"
              onChange={onFileUpload}
              className="hidden"
            />
          </div>
          <textarea
            value={csvText}
            onChange={(e) => onCsvTextChange(e.target.value)}
            placeholder={'GABC...XYZ,40\nGDEF...UVW,60'}
            rows={6}
            style={{
              ...inputStyle(false),
              fontFamily: FONT_MONO,
              resize: 'vertical',
              marginBottom: 8,
            }}
          />
          <div className="text-[11px] mb-3.5" style={{ color: C.textFaint }}>
            Format: <span style={{ fontFamily: FONT_MONO }}>address,percentage</span> — one pair
            per line. Bad rows are flagged individually after parsing, not blocked as a whole
            file.
          </div>
          <FormButton variant="primary" onClick={onApplyCsv} disabled={!csvText.trim()}>
            Parse & add rows
          </FormButton>
        </div>
      )}
    </div>
  );
}
