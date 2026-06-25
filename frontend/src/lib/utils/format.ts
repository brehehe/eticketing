// ============================================================
// Format utilities
// ============================================================

export function currency(value: number, locale = 'id-ID', currency = 'IDR'): string {
  const formatted = new Intl.NumberFormat(locale, {
    style: 'currency',
    currency,
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(value);
  // Replace non-breaking spaces (U+00A0 and U+202F) and other spaces with normal space
  return formatted.replace(/[\u00A0\u202F\s]/g, ' ');
}

export function number(value: number): string {
  return new Intl.NumberFormat('id-ID').format(value);
}

export function date(value: string | Date, opts?: Intl.DateTimeFormatOptions): string {
  const d = typeof value === 'string' ? new Date(value) : value;
  return new Intl.DateTimeFormat('id-ID', opts ?? {
    day: '2-digit', month: 'short', year: 'numeric'
  }).format(d);
}

export function datetime(value: string | Date): string {
  const d = typeof value === 'string' ? new Date(value) : value;
  return new Intl.DateTimeFormat('id-ID', {
    day: '2-digit', month: 'short', year: 'numeric',
    hour: '2-digit', minute: '2-digit'
  }).format(d);
}

export function time(value: string | Date): string {
  const d = typeof value === 'string' ? new Date(value) : value;
  return new Intl.DateTimeFormat('id-ID', { hour: '2-digit', minute: '2-digit' }).format(d);
}

export function relativeTime(value: string | Date): string {
  const d = typeof value === 'string' ? new Date(value) : value;
  const diff = Date.now() - d.getTime();
  const sec  = Math.floor(diff / 1000);
  const min  = Math.floor(sec / 60);
  const hr   = Math.floor(min / 60);
  const day  = Math.floor(hr  / 24);
  if (sec < 60)  return 'Baru saja';
  if (min < 60)  return `${min} menit lalu`;
  if (hr  < 24)  return `${hr} jam lalu`;
  if (day < 7)   return `${day} hari lalu`;
  return date(d);
}

export function initials(name: string): string {
  return name
    .split(' ')
    .map(w => w[0])
    .slice(0, 2)
    .join('')
    .toUpperCase();
}

export function slugify(str: string): string {
  return str
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/(^-|-$)/g, '');
}

export function truncate(str: string, max: number): string {
  return str.length > max ? str.slice(0, max) + '...' : str;
}

export function pct(value: number, total: number, decimals = 1): string {
  if (total === 0) return '0%';
  return ((value / total) * 100).toFixed(decimals) + '%';
}
