const isTauri = typeof window !== "undefined" && !!(window as any).__TAURI__;

export interface FileFilter {
  name: string;
  extensions: string[];
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
async function tauriOpenDialog(filters?: FileFilter[]): Promise<string | null> {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const dialog: any = await import(/* @vite-ignore */ "@tauri-apps/plugin-dialog");
    const selected = await dialog.open({
      multiple: false,
      filters: filters ?? [{ name: "RETAS Project", extensions: ["retas", "json"] }],
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
async function tauriSaveDialog(filters?: FileFilter[]): Promise<string | null> {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const dialog: any = await import(/* @vite-ignore */ "@tauri-apps/plugin-dialog");
    const path = await dialog.save({
      filters: filters ?? [{ name: "RETAS Project", extensions: ["retas"] }],
    });
    return path || null;
  } catch {
    const path = window.prompt("输入保存路径:");
    return path || null;
  }
}

export async function showOpenDialog(filters?: FileFilter[]): Promise<string | null> {
  if (isTauri) {
    return tauriOpenDialog(filters);
  }

  const defaultFilters = filters ?? [{ name: "RETAS Project", extensions: ["retas", "json"] }];
  const acceptExts = defaultFilters.flatMap(f => f.extensions.map(e => `.${e}`));

  try {
    const [handle] = await (window as any).showOpenFilePicker({
      types: defaultFilters.map(f => ({
        description: f.name,
        accept: { "application/octet-stream": f.extensions.map(e => `.${e}`) },
      })),
      multiple: false,
    });
    const file = await handle.getFile();
    return URL.createObjectURL(file);
  } catch {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = acceptExts.join(",");
    return new Promise((resolve) => {
      input.onchange = () => {
        const file = input.files?.[0];
        resolve(file ? URL.createObjectURL(file) : null);
      };
      input.click();
    });
  }
}

export async function showSaveDialog(filters?: FileFilter[]): Promise<string | null> {
  if (isTauri) {
    return tauriSaveDialog(filters);
  }

  const defaultFilters = filters ?? [{ name: "RETAS Project", extensions: ["retas"] }];
  const defaultExt = defaultFilters[0]?.extensions[0] ?? "retas";

  try {
    const handle = await (window as any).showSaveFilePicker({
      types: defaultFilters.map(f => ({
        description: f.name,
        accept: { "application/octet-stream": f.extensions.map(e => `.${e}`) },
      })),
      suggestedName: `export.${defaultExt}`,
    });
    return handle.name;
  } catch {
    const path = window.prompt("输入保存路径:", `export.${defaultExt}`);
    return path || null;
  }
}
