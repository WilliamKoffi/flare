import type { Profile } from "./profile";

export async function check(items: Profile[]): Promise<string> {
  if (!items.length) return "Empty Memory.";

  let stale = 0;
  let checked = 0;
  const broken: string[] = [];

  for (const item of items) {
    const urls = [item.link, item.image].filter(Boolean);
    for (const url of urls) {
      try {
        const response = await fetch(url, { method: "HEAD", mode: "no-cors" });
        checked++;
        if (!response.ok && response.type !== "opaque") {
          stale++;
          broken.push(`${item.name}: ${url}`);
        }
      } catch {
        stale++;
        broken.push(`${item.name}: ${url}`);
      }
    }
  }

  const summary = broken.length
    ? `\n\nBroken:\n${broken.slice(0, 10).join("\n")}` +
      (broken.length > 10 ? `\n...and ${broken.length - 10} more` : "")
    : "";

  return (
    `Link Verification:\n` +
    `Checked: ${checked}\n` +
    `Stale/broken: ${stale}` +
    summary
  );
}
