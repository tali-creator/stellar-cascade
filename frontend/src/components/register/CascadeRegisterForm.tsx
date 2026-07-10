'use client';

import { ChevronLeft, ChevronRight } from 'lucide-react';
import { C, FONT_MONO, FONT_SANS } from '@/lib/tokens';
import { useRegisterForm, OWNER_ADDRESS } from './useRegisterForm';
import { StepIndicator } from './StepIndicator';
import { StepIdentity } from './StepIdentity';
import { StepSplits } from './StepSplits';
import { StepValidate } from './StepValidate';
import { StepMutability } from './StepMutability';
import { StepSubmit } from './StepSubmit';
import { RegisterSuccess } from './RegisterSuccess';
import { SummaryChip } from './SummaryChip';
import { FormButton } from './FormButton';

const STEPS = [
  { n: 1, label: 'Identity' },
  { n: 2, label: 'Splits' },
  { n: 3, label: 'Validate' },
  { n: 4, label: 'Mutability' },
  { n: 5, label: 'Submit' },
];

export default function CascadeRegisterForm() {
  const form = useRegisterForm();

  return (
    <div
      className="flex-1 flex items-center justify-center min-h-screen px-4 py-10 sm:px-6 lg:px-8"
      style={{ background: C.bg, color: C.text, fontFamily: FONT_SANS }}
    >
      <style>{`
        @import url('https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&family=IBM+Plex+Sans:wght@400;500;600&display=swap');
        input::placeholder, textarea::placeholder { color: #4E655A; }
        input:focus, textarea:focus { border-color: #34D399 !important; }
        @keyframes spin { to { transform: rotate(360deg); } }
      `}</style>

      <div className="w-full max-w-[760px]">
        {/* Page header */}
        <div className="mb-6 sm:mb-8">
          <div
            className="text-[11px] tracking-[0.08em] mb-1.5"
            style={{ fontFamily: FONT_MONO, color: C.green }}
          >
            CASCADE · SPLITS REGISTRY
          </div>
          <h1
            className="text-xl sm:text-[22px] font-semibold m-0"
            style={{ fontFamily: FONT_MONO, color: C.text }}
          >
            Register a project
          </h1>
        </div>

        {!form.success ? (
          <>
            <StepIndicator steps={STEPS} currentStep={form.step} />

            <div
              className="rounded-xl p-4 sm:p-6 lg:p-7"
              style={{ background: C.panel, border: `1px solid ${C.border}` }}
            >
              {form.step === 1 && (
                <StepIdentity
                  projectName={form.projectName}
                  onProjectNameChange={form.setProjectName}
                  projectId={form.projectId}
                  availability={form.availability}
                  ownerAddress={OWNER_ADDRESS}
                />
              )}

              {form.step === 2 && (
                <StepSplits
                  receivers={form.receivers}
                  tab={form.tab}
                  onTabChange={form.setTab}
                  csvText={form.csvText}
                  onCsvTextChange={form.setCsvText}
                  fileInputRef={form.fileInputRef}
                  validation={form.validation}
                  onUpdateRow={form.updateRow}
                  onRemoveRow={form.removeRow}
                  onAddRow={form.addRow}
                  onApplyCsv={form.applyCsv}
                  onFileUpload={form.onFileUpload}
                />
              )}

              {form.step === 3 && <StepValidate receivers={form.receivers} />}

              {form.step === 4 && (
                <StepMutability
                  mutability={form.mutability}
                  onToggle={() =>
                    form.setMutability((m) => (m === 'locked' ? 'unlocked' : 'locked'))
                  }
                />
              )}

              {form.step === 5 && (
                <StepSubmit
                  projectName={form.projectName}
                  receiverCount={form.receivers.length}
                  validation={form.validation}
                  mutability={form.mutability}
                  canSubmit={form.canSubmit}
                  submitting={form.submitting}
                  onSubmit={form.handleSubmit}
                />
              )}

              {/* Step navigation */}
              <div
                className="flex justify-between mt-6 sm:mt-7 pt-4 sm:pt-5"
                style={{ borderTop: `1px solid ${C.border}` }}
              >
                <FormButton
                  variant="ghost"
                  icon={ChevronLeft}
                  onClick={form.goBack}
                  disabled={form.step === 1}
                >
                  Back
                </FormButton>

                {form.step < 5 && (
                  <FormButton
                    variant="primary"
                    onClick={form.goNext}
                    disabled={!form.canAdvance}
                    style={{ flexDirection: 'row-reverse' }}
                    icon={ChevronRight}
                  >
                    Continue
                  </FormButton>
                )}
              </div>
            </div>

            {/* Persistent validation chips */}
            {form.step >= 2 && form.step <= 4 && (
              <div className="mt-4 flex gap-2 sm:gap-3 flex-wrap">
                <SummaryChip
                  ok={form.validation.totalOk}
                  label={
                    form.validation.totalOk
                      ? '100% ✓'
                      : `${form.validation.total}% — need ${(100 - form.validation.total).toFixed(0)}% more`
                  }
                />
                <SummaryChip
                  ok={form.validation.noDuplicates}
                  label={
                    form.validation.noDuplicates
                      ? 'No duplicates'
                      : `${form.validation.duplicateAddrs.size} duplicate address(es)`
                  }
                />
                <SummaryChip
                  ok={form.validation.countOk}
                  label={`${form.receivers.length} / 20 receivers`}
                />
              </div>
            )}
          </>
        ) : (
          <RegisterSuccess
            success={form.success}
            projectName={form.projectName}
            receivers={form.receivers}
            copied={form.copied}
            onCopyProjectId={form.copyProjectId}
          />
        )}
      </div>
    </div>
  );
}
