export function formatBody(body: string | undefined | null): string | null {
  if (body == null || body === '') return null;
  try {
    return JSON.stringify(JSON.parse(body), null, 4);
  } catch {
    return body;
  }
}
