import { useEffect, useState, useCallback } from "react";

let canvasKitInstance: any = null;

export function useCanvasKit() {
  const [canvasKit, setCanvasKit] = useState<any>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [memoryInfo, setMemoryInfo] = useState<{ heapSize: number; heapLimit: number } | null>(null);

  const updateMemoryInfo = useCallback(() => {
    if (!canvasKitInstance) return;
    try {
      const ck = canvasKitInstance as any;
      const memory = ck.wasmMemory || ck.memory || ck._memory || 
                    (ck.WasmMemory && ck.WasmMemory.buffer) ||
                    (ck.HEAPU8 && ck.HEAPU8.buffer);
      
      if (memory && memory.buffer) {
        setMemoryInfo({
          heapSize: memory.buffer.byteLength,
          heapLimit: memory.buffer.byteLength,
        });
      } else if (memory && memory.byteLength) {
        setMemoryInfo({
          heapSize: memory.byteLength,
          heapLimit: memory.byteLength,
        });
      } else {
        setMemoryInfo({ heapSize: 0, heapLimit: 0 });
      }
    } catch {
      setMemoryInfo({ heapSize: 0, heapLimit: 0 });
    }
  }, []);

  useEffect(() => {
    if (canvasKitInstance) {
      setCanvasKit(canvasKitInstance);
      setIsLoading(false);
      return;
    }

    const load = async () => {
      try {
        const CanvasKitInit = (await import("canvaskit-wasm")).default;
        const ck = await CanvasKitInit({
          locateFile: (file: string) => {
            return `/node_modules/canvaskit-wasm/bin/${file}`;
          }
        });
        canvasKitInstance = ck;
        setCanvasKit(ck);
      } catch (e) {
        console.error("Failed to load CanvasKit:", e);
      } finally {
        setIsLoading(false);
      }
    };

    load();
  }, []);

  useEffect(() => {
    if (import.meta.env.MODE !== "development") return;
    if (!canvasKit) return;

    const interval = setInterval(updateMemoryInfo, 5000);
    return () => clearInterval(interval);
  }, [canvasKit, updateMemoryInfo]);

  return { canvasKit, isLoading, memoryInfo, updateMemoryInfo };
}
