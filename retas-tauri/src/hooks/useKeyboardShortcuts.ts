import { useEffect } from "react";
import { undo, redo } from "../api";

export function useKeyboardShortcuts() {
  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
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
      }

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
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);
}
