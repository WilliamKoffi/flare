import * as Memory from "./memory";
import * as Scan from "./scan";
import * as Archive from "./archive";
import * as Probe from "./probe";

export function inject(): void {
  const root = document.createElement("div");
  Object.assign(root.style, {
    position: "fixed", bottom: "20px", right: "20px",
    background: "#fff", padding: "15px", borderRadius: "8px",
    boxShadow: "0 4px 12px rgba(0,0,0,0.15)", zIndex: "9999",
    display: "flex", flexDirection: "column", gap: "10px",
    color: "#000", fontFamily: "system-ui, sans-serif", fontSize: "13px",
    maxWidth: "240px"
  });

  // --- Email filter: toggle ---
  const { container: filter, input } = toggle("Ignore Néant", true);

  const strict = () => input.checked;

  // --- Drawer toggle ---
  let open = true;
  const drawer = document.createElement("button");
  drawer.textContent = "▼ Actions";
  Object.assign(drawer.style, {
    padding: "6px", background: "#f8f9fa", color: "#333",
    border: "1px solid #dee2e6", borderRadius: "4px", cursor: "pointer",
    fontWeight: "600", fontSize: "12px"
  });

  const actions = document.createElement("div");
  Object.assign(actions.style, { display: "flex", flexDirection: "column", gap: "8px" });

  drawer.onclick = () => {
    open = !open;
    actions.style.display = open ? "flex" : "none";
    drawer.textContent = open ? "▼ Actions" : "▶ Actions";
  };

  // --- Actions ---
  const traverse = () => Scan.run(strict());

  const save = () => {
    const items = Memory.read();
    if (!items.length) return window.alert("Empty Memory.");

    const invalid = items.filter(i => !i.valid).length;
    if (invalid > 0) {
      const proceed = window.confirm(
        `Validation Warning:\n` +
        `${invalid} of ${items.length} records are incomplete (missing name or email).\n\n` +
        `Proceed with download?`
      );
      if (!proceed) return;
    }

    Archive.download(items);
  };

  const report = () => {
    const items = Memory.read();
    const valid = items.filter(i => i.valid).length;
    const emails = items.filter(i => i.email).length;
    const phones = items.filter(i => i.phone).length;
    const firms = items.filter(i => i.firm).length;
    window.alert(
      `Memory Status:\n` +
      `Total: ${items.length}\n` +
      `Valid: ${valid}\n` +
      `Invalid: ${items.length - valid}\n` +
      `──────────\n` +
      `With email: ${emails}\n` +
      `With phone: ${phones}\n` +
      `With firm: ${firms}`
    );
  };

  const verify = async () => {
    const result = await Probe.check(Memory.read());
    window.alert(result);
  };

  const reset = () => {
    const count = Memory.read().length;
    if (count > 0) {
      const proceed = window.confirm(`Clear ${count} records from memory?`);
      if (!proceed) return;
    }
    Memory.clear();
    window.alert("Memory cleared.");
  };

  // --- Assembly ---
  actions.append(
    filter,
    build("Scan All Pages", "#0d6efd", traverse),
    build("Download TOML", "#198754", save),
    build("View Status", "#0dcaf0", report),
    build("Verify Links", "#6f42c1", verify),
    build("Clear Memory", "#dc3545", reset)
  );

  root.append(drawer, actions);
  document.body.appendChild(root);
}

// --- Builders ---

function build(text: string, color: string, task: () => void): HTMLButtonElement {
  const button = document.createElement("button");
  button.textContent = text;
  button.onclick = task;
  Object.assign(button.style, {
    padding: "8px", background: color, color: "#fff",
    border: "none", borderRadius: "4px", cursor: "pointer",
    fontSize: "12px", fontWeight: "500"
  });
  return button;
}

function toggle(text: string, checked: boolean) {
  const input = document.createElement("input");
  input.type = "checkbox";
  input.checked = checked;
  Object.assign(input.style, { display: "none" });

  const track = document.createElement("span");
  Object.assign(track.style, {
    position: "relative", display: "inline-block",
    width: "36px", height: "20px", borderRadius: "10px",
    background: checked ? "#0d6efd" : "#ccc",
    transition: "background 0.2s", cursor: "pointer", flexShrink: "0"
  });

  const thumb = document.createElement("span");
  Object.assign(thumb.style, {
    position: "absolute", top: "2px",
    left: checked ? "18px" : "2px",
    width: "16px", height: "16px", borderRadius: "50%",
    background: "#fff", transition: "left 0.2s",
    boxShadow: "0 1px 3px rgba(0,0,0,0.2)"
  });
  track.appendChild(thumb);

  const label = document.createElement("span");
  label.textContent = text;
  Object.assign(label.style, { fontSize: "12px", color: "#333", userSelect: "none" });

  const container = document.createElement("label");
  Object.assign(container.style, {
    display: "flex", alignItems: "center", gap: "8px",
    cursor: "pointer", padding: "4px 0"
  });
  container.append(input, track, label);

  input.addEventListener("change", () => {
    track.style.background = input.checked ? "#0d6efd" : "#ccc";
    thumb.style.left = input.checked ? "18px" : "2px";
  });

  return { container, input };
}
