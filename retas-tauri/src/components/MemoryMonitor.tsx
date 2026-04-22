import { useEffect, useState, useRef } from "react";
import { Card, Tag } from "@blueprintjs/core";

interface MemoryMonitorProps {
  memoryInfo: { heapSize: number; heapLimit: number } | null;
  currentTool?: string;
}

function formatBytes(bytes: number): string {
  if (!bytes || bytes === 0) return "--";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

export default function MemoryMonitor({ memoryInfo, currentTool }: MemoryMonitorProps) {
  const [jsHeap, setJsHeap] = useState(0);
  const [fps, setFps] = useState(0);
  const frameCount = useRef(0);
  const lastTime = useRef(Date.now());

  useEffect(() => {
    if ("memory" in performance) {
      const interval = setInterval(() => {
        const mem = (performance as any).memory;
        if (mem) {
          setJsHeap(mem.usedJSHeapSize);
        }
      }, 5000);
      return () => clearInterval(interval);
    }
  }, []);

  useEffect(() => {
    let rafId: number;
    const loop = () => {
      frameCount.current++;
      const now = Date.now();
      if (now - lastTime.current >= 1000) {
        setFps(frameCount.current);
        frameCount.current = 0;
        lastTime.current = now;
      }
      rafId = requestAnimationFrame(loop);
    };
    rafId = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(rafId);
  }, []);

  return (
    <Card style={{
      position: "fixed",
      bottom: 8,
      left: 8,
      zIndex: 1000,
      padding: 8,
      background: "#161b22",
      border: "1px solid #30363d",
      fontSize: 11,
    }}>
      <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
        <Tag minimal intent="primary" style={{ fontSize: 10, justifyContent: "space-between" }}>
          <span>CanvasKit</span>
          <span>{memoryInfo && memoryInfo.heapSize > 0 ? formatBytes(memoryInfo.heapSize) : "--"}</span>
        </Tag>
        <Tag minimal intent="success" style={{ fontSize: 10, justifyContent: "space-between" }}>
          <span>JS Heap</span>
          <span>{jsHeap > 0 ? formatBytes(jsHeap) : "--"}</span>
        </Tag>
        <Tag minimal intent="warning" style={{ fontSize: 10, justifyContent: "space-between" }}>
          <span>FPS</span>
          <span>{fps}</span>
        </Tag>
        {currentTool && (
          <Tag minimal style={{ fontSize: 10, justifyContent: "space-between" }}>
            <span>工具</span>
            <span>{currentTool}</span>
          </Tag>
        )}
      </div>
    </Card>
  );
}
