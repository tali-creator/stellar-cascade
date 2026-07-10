import { C, FONT_MONO } from '@/lib/tokens';

/** Returns an inline style object for text inputs/textareas.
 *  Border color is dynamic (error state), so it cannot be a static Tailwind class. */
export const inputStyle = (hasError: boolean): React.CSSProperties => ({
  width: '100%',
  background: C.input,
  border: `1px solid ${hasError ? C.red : C.border}`,
  borderRadius: 6,
  padding: '10px 12px',
  color: C.text,
  fontFamily: FONT_MONO,
  fontSize: 13,
  outline: 'none',
  boxSizing: 'border-box',
});
