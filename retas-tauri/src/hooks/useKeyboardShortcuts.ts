import { useEffect, useCallback } from "react";
import { undo, redo, clearSelection } from "../api";

export interface ShortcutConfig {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  action: () => void;
  description: string;
}

export function useKeyboardShortcuts(
  onToolChange?: (tool: string) => void,
  onBrushSizeChange?: (delta: number) => void,
  onExport?: () => void,
) {
  const handleKeyDown = useCallback(async (e: KeyboardEvent) => {
    const isInput = (e.target as HTMLElement).tagName === "INPUT" || 
                    (e.target as HTMLElement).tagName === "TEXTAREA";
    if (isInput) return;

    // Ctrl+Z — Undo
    if ((e.ctrlKey || e.metaKey) && e.key === "z" && !e.shiftKey) {
      e.preventDefault();
      try {
        const didUndo = await undo();
        if (didUndo) {
          window.dispatchEvent(new CustomEvent("retas:state-changed"));
        }
      } catch (err) {
        console.error("Undo failed:", err);
      }
      return;
    }

    // Ctrl+Shift+Z / Ctrl+Y — Redo
    if ((e.ctrlKey || e.metaKey) && (e.key === "y" || (e.key === "z" && e.shiftKey))) {
      e.preventDefault();
      try {
        const didRedo = await redo();
        if (didRedo) {
          window.dispatchEvent(new CustomEvent("retas:state-changed"));
        }
      } catch (err) {
        console.error("Redo failed:", err);
      }
      return;
    }

    // Ctrl+E — Export
    if ((e.ctrlKey || e.metaKey) && e.key === "e") {
      e.preventDefault();
      onExport?.();
      return;
    }

    // Delete / Backspace — Clear selection
    if (e.key === "Delete" || e.key === "Backspace") {
      e.preventDefault();
      try {
        await clearSelection();
        window.dispatchEvent(new CustomEvent("retas:state-changed"));
      } catch (err) {
        console.error("Clear selection failed:", err);
      }
      return;
    }

    if (e.ctrlKey || e.metaKey || e.altKey) return;

    switch (e.key.toLowerCase()) {
      case "b":
        onToolChange?.("brush");
        break;
      case "e":
        onToolChange?.("eraser");
        break;
      case "p":
        onToolChange?.("pen");
        break;
      case "g":
        onToolChange?.("fill");
        break;
      case "v":
        onToolChange?.("select");
        break;
      case "m":
        onToolChange?.("move");
        break;
      case "h":
        onToolChange?.("hand");
        break;
      case "z":
        onToolChange?.("zoom");
        break;
      case "[":
        onBrushSizeChange?.(-1);
        break;
      case "]":
        onBrushSizeChange?.(1);
        break;
    }
  }, [onToolChange, onBrushSizeChange, onExport]);

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);
}

export const DEFAULT_SHORTCUTS: ShortcutConfig[] = [
  { key: "b", action: () => {}, description: "画笔工具" },
  { key: "e", action: () => {}, description: "橡皮擦工具" },
  { key: "p", action: () => {}, description: "钢笔工具" },
  { key: "g", action: () => {}, description: "填充工具" },
  { key: "v", action: () => {}, description: "选择工具" },
  { key: "m", action: () => {}, description: "移动工具" },
  { key: "h", action: () => {}, description: "抓手工具" },
  { key: "[", action: () => {}, description: "减小笔刷大小" },
  { key: "]", action: () => {}, description: "增大笔刷大小" },
  { key: "z", ctrl: true, action: () => {}, description: "撤销" },
  { key: "y", ctrl: true, action: () => {}, description: "重做" },
  { key: "z", ctrl: true, shift: true, action: () => {}, description: "重做" },
  { key: "e", ctrl: true, action: () => {}, description: "导出" },
  { key: "Delete", action: () => {}, description: "清除选区" },
];
