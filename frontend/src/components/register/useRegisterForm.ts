import { useState, useEffect, useRef, useMemo } from 'react';

// ─── Constants ────────────────────────────────────────────────────────────────

export const MAX_RECEIVERS = 20;
// TODO: replace with actual has_project RPC call
const TAKEN_NAMES = new Set(['cascade-core', 'stellar-cascade', 'test-project', 'gridwork']);
// TODO: replace with connected wallet address from wallet adapter
export const OWNER_ADDRESS = 'GDQP2KPQGKIHYJGXNUIYOMHARUARCA7DJT5FO2FFOOKY3B2WSQHG4W37';

// ─── Types ────────────────────────────────────────────────────────────────────

export type Availability = 'idle' | 'checking' | 'available' | 'taken';
export type Mutability = 'unlocked' | 'locked';
export type ReceiverTab = 'manual' | 'csv';

export interface ReceiverRow {
  id: string;
  address: string;
  percent: string;
  source: 'manual' | 'csv';
  csvError?: string | null;
  raw?: string;
}

export interface Validation {
  rowErrors: string[][];
  total: number;
  duplicateAddrs: Set<string>;
  hasRowErrors: boolean;
  allRowsFilled: boolean;
  totalOk: boolean;
  noDuplicates: boolean;
  countOk: boolean;
}

export interface SuccessResult {
  projectId: string;
  txHash: string;
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

function useDebounced<T>(value: T, delay: number): T {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const t = setTimeout(() => setDebounced(value), delay);
    return () => clearTimeout(t);
  }, [value, delay]);
  return debounced;
}

async function sha256Hex(text: string): Promise<string> {
  const enc = new TextEncoder().encode(text);
  const buf = await crypto.subtle.digest('SHA-256', enc);
  return Array.from(new Uint8Array(buf))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

export function isValidStellarAddress(addr: string): boolean {
  return /^G[A-Z2-7]{55}$/.test((addr || '').trim());
}

export function pctToBps(pct: string): number | null {
  const n = parseFloat(pct);
  if (Number.isNaN(n)) return null;
  return Math.round(n * 100);
}

export function newRow(address = '', percent = ''): ReceiverRow {
  return { id: crypto.randomUUID(), address, percent, source: 'manual' };
}

export function parseCsv(raw: string): ReceiverRow[] {
  const lines = raw
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter(Boolean);
  return lines.map((line) => {
    const parts = line.split(',').map((p) => p.trim());
    const [address = '', percent = ''] = parts;
    const errors: string[] = [];
    if (!isValidStellarAddress(address)) errors.push('Invalid Stellar address');
    if (percent === '' || Number.isNaN(parseFloat(percent)))
      errors.push('Non-numeric percentage');
    return {
      id: crypto.randomUUID(),
      address,
      percent,
      source: 'csv',
      csvError: errors[0] || null,
      raw: line,
    };
  });
}

// ─── Hook ─────────────────────────────────────────────────────────────────────

export function useRegisterForm() {
  // ── Step navigation ──────────────────────────────────────────────────────
  const [step, setStep] = useState(1);

  // ── Step 1 — identity ────────────────────────────────────────────────────
  const [projectName, setProjectName] = useState('');
  const debouncedName = useDebounced(projectName, 400);
  const [projectId, setProjectId] = useState('');
  const [availability, setAvailability] = useState<Availability>('idle');

  useEffect(() => {
    if (!debouncedName.trim()) {
      setAvailability('idle');
      setProjectId('');
      return;
    }
    let cancelled = false;
    setAvailability('checking');
    (async () => {
      const hex = await sha256Hex(debouncedName.trim());
      await new Promise((r) => setTimeout(r, 350)); // simulate has_project RPC latency
      if (cancelled) return;
      setProjectId('0x' + hex);
      setAvailability(
        TAKEN_NAMES.has(debouncedName.trim().toLowerCase()) ? 'taken' : 'available'
      );
    })();
    return () => {
      cancelled = true;
    };
  }, [debouncedName]);

  // ── Step 2 — splits ───────────────────────────────────────────────────────
  const [receivers, setReceivers] = useState<ReceiverRow[]>([newRow(), newRow()]);
  const [tab, setTab] = useState<ReceiverTab>('manual');
  const [csvText, setCsvText] = useState('');
  const fileInputRef = useRef<HTMLInputElement>(null);

  const updateRow = (id: string, patch: Partial<ReceiverRow>) =>
    setReceivers((rows) =>
      rows.map((r) => (r.id === id ? { ...r, ...patch, csvError: null } : r))
    );
  const removeRow = (id: string) =>
    setReceivers((rows) => rows.filter((r) => r.id !== id));
  const addRow = () =>
    setReceivers((rows) =>
      rows.length >= MAX_RECEIVERS ? rows : [...rows, newRow()]
    );

  const applyCsv = () => {
    const parsed = parseCsv(csvText);
    if (!parsed.length) return;
    setReceivers((rows) => {
      const room = MAX_RECEIVERS - rows.length;
      return [...rows, ...parsed.slice(0, Math.max(room, 0))];
    });
    setTab('manual');
  };

  const onFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => setCsvText(String(reader.result || ''));
    reader.readAsText(file);
  };

  // ── Derived validation ────────────────────────────────────────────────────
  const validation = useMemo<Validation>(() => {
    const rowErrors = receivers.map((r) => {
      const errs: string[] = [];
      if (r.csvError) {
        errs.push(r.csvError);
      } else {
        if (r.address && !isValidStellarAddress(r.address))
          errs.push('Invalid Stellar address');
        if (r.percent !== '' && Number.isNaN(parseFloat(r.percent)))
          errs.push('Non-numeric percentage');
      }
      return errs;
    });
    const total = receivers.reduce(
      (sum, r) => sum + (parseFloat(r.percent) || 0),
      0
    );
    const addrCounts: Record<string, number> = {};
    receivers.forEach((r) => {
      const a = r.address.trim().toLowerCase();
      if (a) addrCounts[a] = (addrCounts[a] || 0) + 1;
    });
    const duplicateAddrs = new Set(
      Object.keys(addrCounts).filter((a) => addrCounts[a] > 1)
    );
    const hasRowErrors = rowErrors.some((e) => e.length > 0);
    const allRowsFilled =
      receivers.length > 0 &&
      receivers.every((r) => r.address.trim() && r.percent !== '');
    const totalOk = Math.abs(total - 100) < 0.001;
    const noDuplicates = duplicateAddrs.size === 0;
    const countOk = receivers.length > 0 && receivers.length <= MAX_RECEIVERS;
    return {
      rowErrors,
      total,
      duplicateAddrs,
      hasRowErrors,
      allRowsFilled,
      totalOk,
      noDuplicates,
      countOk,
    };
  }, [receivers]);

  // ── Step 4 — mutability ───────────────────────────────────────────────────
  const [mutability, setMutability] = useState<Mutability>('unlocked');

  // ── Step 5 — submit ───────────────────────────────────────────────────────
  const [submitting, setSubmitting] = useState(false);
  const [success, setSuccess] = useState<SuccessResult | null>(null);
  const [copied, setCopied] = useState(false);

  const canSubmit =
    availability === 'available' &&
    validation.totalOk &&
    validation.noDuplicates &&
    validation.countOk &&
    validation.allRowsFilled &&
    !validation.hasRowErrors;

  const handleSubmit = async () => {
    setSubmitting(true);
    await new Promise((r) => setTimeout(r, 1400)); // simulate wallet sign + relay
    setSubmitting(false);
    setSuccess({
      projectId,
      txHash:
        '0x' +
        Array.from(crypto.getRandomValues(new Uint8Array(32)))
          .map((b) => b.toString(16).padStart(2, '0'))
          .join(''),
    });
  };

  const copyProjectId = () => {
    navigator.clipboard?.writeText(projectId);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  // ── Step gate guards ──────────────────────────────────────────────────────
  const canAdvanceFrom1 = availability === 'available';
  const canAdvanceFrom2 = validation.allRowsFilled && !validation.hasRowErrors;
  const canAdvanceFrom3 =
    validation.totalOk && validation.noDuplicates && validation.countOk;

  const goNext = () => setStep((s) => Math.min(5, s + 1));
  const goBack = () => setStep((s) => Math.max(1, s - 1));

  const canAdvance =
    (step === 1 && canAdvanceFrom1) ||
    (step === 2 && canAdvanceFrom2) ||
    (step === 3 && canAdvanceFrom3) ||
    step === 4;

  return {
    // navigation
    step,
    goNext,
    goBack,
    canAdvance,
    // step 1
    projectName,
    setProjectName,
    projectId,
    availability,
    // step 2
    receivers,
    tab,
    setTab,
    csvText,
    setCsvText,
    fileInputRef,
    updateRow,
    removeRow,
    addRow,
    applyCsv,
    onFileUpload,
    // validation
    validation,
    // step 4
    mutability,
    setMutability,
    // step 5
    submitting,
    canSubmit,
    handleSubmit,
    // success
    success,
    copied,
    copyProjectId,
  };
}
