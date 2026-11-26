// Canonical bytes are retained; padding is never counted as another report.
export function reportsAt(pointer, length, embedded) {
  if (length === 0 || pointer.isNull()) return [];
  if (length > 4096) return [];
  const buffer = pointer.readByteArray(length);
  if (buffer === null) throw new Error("Report buffer was unreadable");
  const bytes = new Uint8Array(buffer);
  const reports = [];
  for (let offset = 0; offset < (embedded ? length : 1); offset++) {
    const id = bytes[offset];
    const size = id === 0x31 ? 11 : id === 0x32 ? 41 : 0;
    if (size === 0 || offset + size > length) continue;
    if (id === 0x31 && (bytes[offset + 1] !== 1 || bytes[offset + 6] !== 1))
      continue;
    const hex = Array.from(bytes.subarray(offset, offset + size), (byte) =>
      byte.toString(16).padStart(2, "0"),
    ).join("");
    reports.push({ offset, report_id: id, hex, buffer_length: length });
    offset += size - 1;
  }
  return reports;
}
