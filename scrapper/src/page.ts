import type { Profile } from "./profile";
import * as Lawyer from "./lawyer";

export function read(strict: boolean): Profile[] {
  const nodes = document.querySelectorAll(".card.border-t-blue");
  if (nodes.length === 0) {
    throw new Error("No lawyer cards found. The site layout may have changed or selectors are stale.");
  }
  const items = Array.from(nodes).map(Lawyer.parse);
  return strict ? items.filter(p => p.email) : items;
}
