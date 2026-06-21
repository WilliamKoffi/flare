import * as Memory from "./memory";
import * as Page from "./page";

interface Progress {
  pages: number;
  added: number;
  skipped: number;
}

export function run(strict: boolean): void {
  traverse(strict, { pages: 0, added: 0, skipped: 0 });
}

function traverse(strict: boolean, progress: Progress): void {
  try {
    const items = Page.read(strict);
    const stats = Memory.merge(items);
    progress.pages++;
    progress.added += stats.added;
    progress.skipped += stats.skipped;
  } catch (error) {
    window.alert(
      `Extraction error on page ${progress.pages + 1}:\n` +
      `${(error as Error).message}\n\n` +
      `Scanned ${progress.pages} pages before failure.`
    );
    return;
  }

  const next = document.querySelector<HTMLAnchorElement>('.pagination .next a, a[rel="next"]');
  if (next) {
    next.click();
    setTimeout(() => traverse(strict, progress), 2000);
  } else {
    window.alert(
      `Scan complete.\n` +
      `Pages scanned: ${progress.pages}\n` +
      `Total records: ${Memory.read().length}\n` +
      `New: ${progress.added}\n` +
      `Duplicates skipped: ${progress.skipped}`
    );
  }
}
