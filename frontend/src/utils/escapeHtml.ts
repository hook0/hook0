/** Escape HTML special characters to prevent XSS in tooltip content. */
const escapeDiv = document.createElement('div');
export function escapeHtml(str: string): string {
  escapeDiv.textContent = str;
  return escapeDiv.innerHTML;
}
