import type { Profile } from "./profile";

export interface Stats {
  added: number;
  skipped: number;
}

const key = "directory";

export function read(): Profile[] {
  try {
    const raw = localStorage.getItem(key);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

export function write(items: Profile[]): void {
  localStorage.setItem(key, JSON.stringify(items));
}

export function merge(items: Profile[]): Stats {
  const pool = read();
  const map = new Map(pool.map(p => [p.id, p]));
  let added = 0;
  let skipped = 0;
  for (const p of items) {
    if (map.has(p.id)) skipped++;
    else added++;
    map.set(p.id, p);
  }
  write(Array.from(map.values()));
  return { added, skipped };
}

export function clear(): void {
  localStorage.removeItem(key);
}
