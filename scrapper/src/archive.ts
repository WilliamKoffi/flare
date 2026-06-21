import type { Profile } from "./profile";

const VERSION = "2.0";

function escape(text: string): string {
  return text
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\n/g, '\\n')
    .replace(/\r/g, '\\r')
    .replace(/\t/g, '\\t');
}

export function format(items: Profile[]): string {
  const sorted = [...items].sort((a, b) => a.name.localeCompare(b.name));

  const valid = sorted.filter(i => i.valid).length;
  const header =
    `[metadata]\n` +
    `source = "${escape(window.location.href)}"\n` +
    `timestamp = "${new Date().toISOString()}"\n` +
    `version = "${VERSION}"\n` +
    `total = ${sorted.length}\n` +
    `valid = ${valid}\n\n`;

  const body = sorted.map(item =>
    `[[lawyer]]\n` +
    `id = "${escape(item.id)}"\n` +
    `name = "${escape(item.name)}"\n` +
    `email = "${escape(item.email)}"\n` +
    `phone = "${escape(item.phone)}"\n` +
    `image = "${escape(item.image)}"\n` +
    `link = "${escape(item.link)}"\n` +
    `firm = "${escape(item.firm)}"\n` +
    `office = "${escape(item.office)}"\n` +
    `valid = ${item.valid}\n`
  ).join("\n");

  return header + body;
}

export function download(items: Profile[]): void {
  const blob = new Blob([format(items)], { type: "text/plain" });
  const anchor = document.createElement("a");
  anchor.href = URL.createObjectURL(blob);
  anchor.download = "directory.toml";
  anchor.click();
  URL.revokeObjectURL(anchor.href);
}
