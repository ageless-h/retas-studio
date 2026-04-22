import { useRef, useEffect, useCallback, useState } from "react";
import { drawStroke, DrawCommand } from "../api";

interface CanvasProps {
  tool: string;
  zoom: number;
  color?: string;
  brushSize?: number;
}

export default function Canvas({ tool, zoom, color = "#000000", brushSize = 2 }: CanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const isDrawingRef = useRef(false);
  const pointsRef = useRef<[number, number][]>([]);
  const lastPosRef = useRef<{ x: number; y: number } | null>(null);
  const [isDrawing, setIsDrawing] = useState(false);

  const getCanvasPos = useCallback((e: React.MouseEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0 };
    const rect = canvas.getBoundingClientRect();
    return {
      x: (e.clientX - rect.left) / (zoom / 100),
      y: (e.clientY - rect.top) / (zoom / 100),
    };
  }, [zoom]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    canvas.width = 1920;
    canvas.height = 1080;
    
    // White background
    ctx.fillStyle = "white";
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    
    // Grid
    ctx.strokeStyle = "#eee";
    ctx.lineWidth = 1;
    for (let x = 0; x < canvas.width; x += 50) {
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x, canvas.height);
      ctx.stroke();
    }
    for (let y = 0; y < canvas.height; y += 50) {
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(canvas.width, y);
      ctx.stroke();
    }
  }, []);

  const hexToRgb = (hex: string): [number, number, number] => {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result 
      ? [parseInt(result[1], 16), parseInt(result[2], 16), parseInt(result[3], 16)]
      : [0, 0, 0];
  };

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (tool === "brush" || tool === "eraser") {
      isDrawingRef.current = true;
      setIsDrawing(true);
      const pos = getCanvasPos(e);
      lastPosRef.current = pos;
      pointsRef.current = [[pos.x, pos.y]];
    }
  }, [tool, getCanvasPos]);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (!isDrawingRef.current || !lastPosRef.current) return;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const pos = getCanvasPos(e);
    
    // Local drawing
    ctx.beginPath();
    ctx.moveTo(lastPosRef.current.x, lastPosRef.current.y);
    ctx.lineTo(pos.x, pos.y);
    ctx.strokeStyle = tool === "eraser" ? "white" : color;
    ctx.lineWidth = tool === "eraser" ? 20 : brushSize;
    ctx.lineCap = "round";
    ctx.stroke();

    pointsRef.current.push([pos.x, pos.y]);
    lastPosRef.current = pos;
  }, [tool, color, brushSize, getCanvasPos]);

  const handleMouseUp = useCallback(async () => {
    if (!isDrawingRef.current) return;
    isDrawingRef.current = false;
    setIsDrawing(false);
    lastPosRef.current = null;

    // Send draw command to Rust backend
    if (pointsRef.current.length > 1) {
      try {
        const command: DrawCommand = {
          tool,
          points: pointsRef.current,
          color: hexToRgb(color),
          size: brushSize,
          layerId: "current",
        };
        await drawStroke(command);
      } catch (e) {
        console.error("Draw failed:", e);
      }
    }
    
    pointsRef.current = [];
  }, [tool, color, brushSize]);

  return (
    <div style={{ position: "relative" }}>
      <canvas
        ref={canvasRef}
        style={{
          width: `${1920 * (zoom / 100)}px`,
          height: `${1080 * (zoom / 100)}px`,
          maxWidth: "100%",
          maxHeight: "100%",
          cursor: tool === "brush" ? "crosshair" : "default",
        }}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      />
      {isDrawing && (
        <div style={{
          position: "absolute",
          bottom: 8,
          right: 8,
          background: "rgba(0,0,0,0.7)",
          color: "white",
          padding: "4px 8px",
          borderRadius: 4,
          fontSize: 12,
        }}>
          绘制中...
        </div>
      )}
    </div>
  );
}
