import { useEffect, useRef, useCallback, useState } from "react";
import { useCanvasKit } from "../hooks/useCanvasKit";
import { drawStroke, DrawCommand } from "../api";

interface SkiaCanvasProps {
  tool: string;
  zoom: number;
  color?: string;
  brushSize?: number;
}

export default function SkiaCanvas({ tool, zoom, color = "#000000", brushSize = 2 }: SkiaCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const surfaceRef = useRef<any>(null);
  const { canvasKit, isLoading } = useCanvasKit();
  const isDrawingRef = useRef(false);
  const pointsRef = useRef<[number, number][]>([]);
  const lastPosRef = useRef<{ x: number; y: number } | null>(null);
  const paintRef = useRef<any>(null);
  const [canvasSize, setCanvasSize] = useState({ width: 1920, height: 1080 });

  // Listen to container resize
  useEffect(() => {
    if (!containerRef.current) return;

    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width, height } = entry.contentRect;
        const targetWidth = Math.max(width, 800);
        const targetHeight = Math.max(height, 450);
        setCanvasSize({ width: Math.floor(targetWidth), height: Math.floor(targetHeight) });
      }
    });

    resizeObserver.observe(containerRef.current);
    return () => resizeObserver.disconnect();
  }, []);

  useEffect(() => {
    if (!canvasKit || !canvasRef.current) return;

    const canvas = canvasRef.current;
    canvas.width = canvasSize.width;
    canvas.height = canvasSize.height;

    const surface = canvasKit.MakeCanvasSurface(canvas);
    if (!surface) {
      console.error("Failed to create Skia surface");
      return;
    }
    surfaceRef.current = surface;

    // Create paint for drawing
    const paint = new canvasKit.Paint();
    paint.setStyle(canvasKit.PaintStyle.Stroke);
    paint.setStrokeWidth(brushSize);
    paint.setColor(canvasKit.Color(0, 0, 0, 1));
    paint.setAntiAlias(true);
    paintRef.current = paint;

    const ctx = surface.getCanvas();
    ctx.clear(canvasKit.Color(30, 32, 38, 1));

    const gridPaint = new canvasKit.Paint();
    gridPaint.setColor(canvasKit.Color(45, 48, 56, 1));
    gridPaint.setStrokeWidth(1);

    for (let x = 0; x < canvasSize.width; x += 40) {
      ctx.drawLine(x, 0, x, canvasSize.height, gridPaint);
    }
    for (let y = 0; y < canvasSize.height; y += 40) {
      ctx.drawLine(0, y, canvasSize.width, y, gridPaint);
    }

    gridPaint.delete();

    surface.flush();

    return () => {
      surface.delete();
      paint.delete();
    };
  }, [canvasKit, canvasSize.width, canvasSize.height]);

  // Update paint when color or brush size changes
  useEffect(() => {
    if (!paintRef.current || !canvasKit) return;

    const rgb = hexToRgb(color);
    if (tool === "eraser") {
      paintRef.current.setColor(canvasKit.Color(255, 255, 255, 1));
      paintRef.current.setStrokeWidth(20);
    } else {
      paintRef.current.setColor(canvasKit.Color(rgb[0], rgb[1], rgb[2], 1));
      paintRef.current.setStrokeWidth(brushSize);
    }
  }, [color, brushSize, tool, canvasKit]);

  const getCanvasPos = useCallback((e: React.MouseEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0 };
    const rect = canvas.getBoundingClientRect();
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;
    return {
      x: (e.clientX - rect.left) * scaleX,
      y: (e.clientY - rect.top) * scaleY,
    };
  }, []);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (!surfaceRef.current || !paintRef.current) return;
    if (tool === "brush" || tool === "eraser") {
      isDrawingRef.current = true;
      const pos = getCanvasPos(e);
      lastPosRef.current = pos;
      pointsRef.current = [[pos.x, pos.y]];
    }
  }, [tool, getCanvasPos]);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    if (!isDrawingRef.current || !lastPosRef.current || !surfaceRef.current || !paintRef.current) return;

    const pos = getCanvasPos(e);
    const ctx = surfaceRef.current.getCanvas();
    
    ctx.drawLine(
      lastPosRef.current.x, 
      lastPosRef.current.y, 
      pos.x, 
      pos.y, 
      paintRef.current
    );
    
    surfaceRef.current.flush();
    pointsRef.current.push([pos.x, pos.y]);
    lastPosRef.current = pos;
  }, [getCanvasPos]);

  const handleMouseUp = useCallback(async () => {
    if (!isDrawingRef.current) return;
    isDrawingRef.current = false;
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
        const result = await drawStroke(command);
        console.log("Draw result:", result);
      } catch (e) {
        console.error("Draw failed:", e);
      }
    }
    
    pointsRef.current = [];
  }, [tool, color, brushSize]);

  if (isLoading) {
    return (
      <div style={{ 
        width: "100%", 
        height: "100%", 
        display: "flex", 
        alignItems: "center", 
        justifyContent: "center",
        color: "#8b949e"
      }}>
        加载 CanvasKit...
      </div>
    );
  }

  return (
    <div 
      ref={containerRef}
      style={{ 
        position: "relative", 
        width: "100%", 
        height: "100%",
        overflow: "auto"
      }}
    >
      <canvas
        ref={canvasRef}
        style={{
          width: `${canvasSize.width * (zoom / 100)}px`,
          height: `${canvasSize.height * (zoom / 100)}px`,
          maxWidth: "100%",
          maxHeight: "100%",
          cursor: tool === "brush" ? "crosshair" : "default",
        }}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      />
    </div>
  );
}

function hexToRgb(hex: string): [number, number, number] {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result 
    ? [parseInt(result[1], 16), parseInt(result[2], 16), parseInt(result[3], 16)]
    : [0, 0, 0];
}
