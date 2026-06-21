import type { Profile } from "./profile";

export function parse(node: Element): Profile {
  const name = node.querySelector('a[href*="/avocat/"] p')?.getAttribute("data-bs-title") || "";
  const email = node.querySelector('a[href^="mailto:"] p')?.getAttribute("data-bs-title") || "";
  const digits = node.querySelector('a[href^="tel:"] span')?.textContent || "";
  const image = node.querySelector("img.rounded-circle")?.getAttribute("src") || "";
  const path = node.querySelector('a[href*="/avocat/"]')?.getAttribute("href") || "";
  const firm = node.querySelector('a[href*="/structures/"] p')?.getAttribute("data-bs-title") || "";
  const office = node.querySelector('a[href*="/structures/"]')?.getAttribute("href") || "";

  const id = path.split("/").pop() || name.replace(/\s+/g, "");
  const valid = Boolean(name && email && email !== "Néant");

  return {
    id: id.trim(),
    name: name.trim(),
    email: email === "Néant" ? "" : email.trim(),
    phone: digits.replace(/\D/g, "").slice(-10),
    image: image.trim(),
    link: path.trim() ? new URL(path.trim(), window.location.origin).href : "",
    firm: firm.trim(),
    office: office.trim(),
    valid
  };
}
