/**
 * Watch mode: rebuilds dist/scrapper.user.js on every save of main.user.ts.
 * Uses bun's native file watcher + bundler.
 *
 * Run with: bun --watch scripts/watcher.ts
 * (bun --watch restarts this script on changes to imported files)
 */

import { $ } from "bun";

console.log("👀 Building and watching main.user.ts ...\n");

await $`bun build main.user.ts --outfile=dist/scrapper.user.js --format=iife --no-splitting`;
await $`bun scripts/banner.ts`;
