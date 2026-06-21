/**
 * Extracts the Tampermonkey metadata banner from main.user.ts
 * and prepends it to the bundled dist/scrapper.user.js output.
 */

const source = await Bun.file("main.user.ts").text();
const target = "dist/scrapper.user.js";
const output = await Bun.file(target).text();

const match = source.match(
  /\/\/ ==UserScript==[\s\S]*?\/\/ ==\/UserScript==/,
);

if (!match) {
  console.error("❌ No UserScript banner found in main.user.ts");
  process.exit(1);
}

const banner = match[0];

// Only prepend if not already present
if (!output.startsWith("// ==UserScript==")) {
  await Bun.write(target, banner + "\n\n" + output);
}

console.log(`✅ Banner prepended to ${target}`);
