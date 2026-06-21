import { chromium } from 'playwright';
import filesystem from 'node:fs/promises';

export interface Lawyer {
  name: string | null;
  profile: string | null;
  firm: string | null;
  website: string | null;
  email: string | null;
  phone: string | null;
  avatar: string | null;
}

export namespace Directory {
  export async function read(start: string): Promise<Lawyer[]> {
    const browser = await chromium.launch({ headless: false });
    try {
      const page = await browser.newPage();
      const records: Lawyer[] = [];
      let address: string | null = start;

      while (address) {
        await page.goto(address, { waitUntil: 'domcontentloaded' });
        await page.waitForLoadState('networkidle').catch(() => {});

        const batch = await page.locator('div.card.border-t-blue').evaluateAll((cards) =>
          cards.map((card) => {
            const text = (selector: string) =>
              card.querySelector(selector)?.textContent?.replace(/\s+/g, ' ').trim() || null;

            const link = (selector: string) =>
              card.querySelector(selector)?.getAttribute('href') || null;

            const mail = link('a[href^="mailto:"]');
            const dial = link('a[href^="tel:"]');

            return {
              name: text('a[href*="/avocat/"] p'),
              profile: link('a[href*="/avocat/"]'),
              firm: text('a[href*="/structures/"] p'),
              website: link('a[href*="/structures/"]'),
              email: mail && mail !== 'mailto:' ? mail.replace(/^mailto:/, '') : null,
              phone:
                text('a[href^="tel:"] span') ||
                (dial && dial !== 'tel:' ? dial.replace(/^tel:/, '') : null),
              avatar: card.querySelector('img.rounded-circle')?.getAttribute('src') || null,
            };
          })
        );

        records.push(...batch);

        const next = await page
          .locator('a[rel="next"]')
          .first()
          .getAttribute('href')
          .catch(() => null);

        if (!next) break;

        address = new URL(next, page.url()).href;
        await page.waitForTimeout(250);
      }

      return records;
    } finally {
      await browser.close();
    }
  }
}

export namespace Archive {
  export async function write(records: Lawyer[], destination: string): Promise<void> {
    const content = JSON.stringify(records, null, 2);
    const parent = destination.slice(0, destination.lastIndexOf('/'));
    if (parent) await filesystem.mkdir(parent, { recursive: true });
    await filesystem.writeFile(destination, content, 'utf8');
  }
}

// Composition
const target = 'https://app.ordredesavocats-ci.net/annuaire';
const destination = process.argv[2] || '../scrapper/directory.json';

Directory.read(target)
  .then((records) => {
    console.log(`Scraped ${records.length} records -> ${destination}`);
    return Archive.write(records, destination);
  })
  .catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
