const isTauri = typeof window !== "undefined" && !!(window as any).__TAURI__;

// eslint-disable-next-line @typescript-eslint/no-explicit-any
async function tauriOpenDialog(): Promise<string | null> {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const dialog: any = await import(/* @vite-ignore */ "@tauri-apps/plugin-dialog");
    const selected = await dialog.open({
      multiple: false,
      filters: [{ name: "RETAS Project", extensions: ["retas", "json"] }],
    });
    if (selected && typeof selected === "string") {
      return selected;
    }
    return null;
  } catch {
    const path = window.prompt("输入文件路径:");
    return path || null;
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
async function tauriSaveDialog(): Promise<string | null> {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const dialog: any = await import(/* @vite-ignore */ "@tauri-apps/plugin-dialog");
    const path = await dialog.save({
      filters: [{ name: "RETAS Project", extensions: ["retas"] }],
    });
    return path || null;
  } catch {
    const path = window.prompt("输入保存路径:");
    return path || null;
  }
}

export async function showOpenDialog(): Promise<string | null> {
  if (isTauri) {
    return tauriOpenDialog();
  }

  try {
    const [handle] = await (window as any).showOpenFilePicker({
      types: [{ description: "RETAS Project", accept: { "application/json": [".retas", ".json"] } }],
      multiple: false,
    });
    const file = await handle.getFile();
    return URL.createObjectURL(file);
  } catch {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".retas,.json";
    return new Promise((resolve) => {
      input.onchange = () => {
        const file = input.files?.[0];
        resolve(file ? URL.createObjectURL(file) : null);
      };
      input.click();
    });
  }
}

export async function showSaveDialog(): Promise<string | null> {
  if (isTauri) {
    return tauriSaveDialog();
  }

  try {
    const handle = await (window as any).showSaveFilePicker({
      types: [{ description: "RETAS Project", accept: { "application/json": [".retas"] } }],
      suggestedName: "project.retas",
    });
    return handle.name;
  } catch {
    const path = window.prompt("输入保存路径:", "project.retas");
    return path || null;
  }
}
