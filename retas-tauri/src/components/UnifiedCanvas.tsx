import { useEffect, useRef, useCallback, useState } from "react";
import { useCanvasKit } from "../hooks/useCanvasKit";
import { drawStroke, compositeLayers, DrawCommand } from "../api";
import { canvasMonitor } from "../utils/CanvasMonitor";

interface UnifiedCanvasProps {
  tool: string;
  zoom: number;
  color?: string;
  brushSize?: number;
}

const DOC_WIDTH = 1920;
const DOC_HEIGHT = 1080;

export default function UnifiedCanvas({
  tool,
  zoom: _zoom,
  color = "#000000",
  brushSize = 2,
}: UnifiedCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { canvasKit, isLoading } = useCanvasKit();

  const surfaceRef = useRef<any>(null);
  const paintRef = useRef<any>(null);
  const bgImageRef = useRef<any>(null);
  const rafRef = useRef<number>(0);

  const isDrawingRef = useRef(false);
  const pointsRef = useRef<[number, number][]>([]);
  const lastPosRef = useRef<{ x: number; y: number } | null>(null);
  const needsRefreshRef = useRef(true);
  const isRefreshingRef = useRef(false);

  const [debugInfo, setDebugInfo] = useState({
    isDrawing: false,
    points: 0,
    lastX: 0,
    lastY: 0,
    bgReady: false,
    tool,
    color,
    brushSize,
  });

  useEffect(() => {
    setDebugInfo((prev) => ({ ...prev, tool, color, brushSize }));
  }, [tool, color, brushSize]);

  const getCursor = () => {
    switch (tool) {
      case "brush":
      case "eraser":
        return "crosshair";
      case "hand":
        return "grab";
      case "zoom":
        return "zoom-in";
      case "select":
        return "default";
      default:
        return "default";
    }
  };

  useEffect(() => {
    if (!canvasKit || !canvasRef.current) return;
    const canvas = canvasRef.current;
    canvas.width = DOC_WIDTH;
    canvas.height = DOC_HEIGHT;

    const surface = canvasKit.MakeCanvasSurface(canvas);
    if (!surface) {
      console.error("Failed to create Skia surface");
      return;
    }
    surfaceRef.current = surface;
    canvasMonitor.setSurface(surface);
    canvasMonitor.start();

    const paint = new canvasKit.Paint();
    paint.setStyle(canvasKit.PaintStyle.Stroke);
    paint.setStrokeWidth(brushSize);
    paint.setColor(canvasKit.Color4f(0, 0, 0, 1.0));
    paint.setAntiAlias(true);
    paintRef.current = paint;

    needsRefreshRef.current = true;

    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      try { surface.delete(); } catch {}
      try { paint.delete(); } catch {}
      if (bgImageRef.current) {
        try { bgImageRef.current.delete(); } catch {}
      }
      canvasMonitor.stop();
    };
  }, [canvasKit, brushSize]);

  useEffect(() => {
    if (!paintRef.current || !canvasKit) return;
    const rgb = hexToRgb(color);
    if (tool === "eraser") {
      paintRef.current.setColor(canvasKit.Color4f(0, 0, 0, 0));
      paintRef.current.setBlendMode(canvasKit.BlendMode.DstOut);
      paintRef.current.setStrokeWidth(20);
    } else {
      paintRef.current.setColor(canvasKit.Color4f(rgb[0] / 255, rgb[1] / 255, rgb[2] / 255, 1.0));
      paintRef.current.setBlendMode(canvasKit.BlendMode.SrcOver);
      paintRef.current.setStrokeWidth(brushSize);
    }
  }, [color, brushSize, tool, canvasKit]);

  const renderBackground = useCallback(async () => {
    if (!canvasKit || !surfaceRef.current) return;
    try {
      const pixels = await compositeLayers();
      const pixelArray = pixels instanceof Uint8Array ? pixels : new Uint8Array(pixels);
      const info = {
        width: DOC_WIDTH,
        height: DOC_HEIGHT,
        colorType: canvasKit.ColorType.RGBA_8888,
        alphaType: canvasKit.AlphaType.Unpremul,
        colorSpace: canvasKit.ColorSpace.SRGB,
      };
      const img = canvasKit.MakeImage(info, pixelArray, DOC_WIDTH * 4);
      if (bgImageRef.current) {
        try { bgImageRef.current.delete(); } catch {}
      }
      bgImageRef.current = img;
      redraw();
      canvasMonitor.log({ type: "composite", message: "Background rendered" });
    } catch (e: any) {
      console.error("Composite failed:", e);
      canvasMonitor.log({ type: "error", message: "Composite failed", error: e.message });
    }
  }, [canvasKit]);

  const redraw = useCallback(() => {
    if (!surfaceRef.current || !canvasKit) return;
    const ctx = surfaceRef.current.getCanvas();
    ctx.clear(canvasKit.Color4f(30 / 255, 32 / 255, 38 / 255, 1.0));
    if (bgImageRef.current) {
      ctx.drawImage(bgImageRef.current, 0, 0);
    }
    surfaceRef.current.flush();
  }, [canvasKit]);

  useEffect(() => {
    if (!canvasKit) return;
    const loop = () => {
      if (needsRefreshRef.current && !isDrawingRef.current && !isRefreshingRef.current) {
        isRefreshingRef.current = true;
        needsRefreshRef.current = false;
        renderBackground().finally(() => {
          isRefreshingRef.current = false;
        });
      }
      rafRef.current = requestAnimationFrame(loop);
    };
    rafRef.current = requestAnimationFrame(loop);
    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
    };
  }, [canvasKit, renderBackground]);

  useEffect(() => {
    if (canvasKit && surfaceRef.current) {
      renderBackground();
    }
  }, [canvasKit, renderBackground]);

  const getCanvasPos = useCallback((e: React.MouseEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0 };
    const rect = canvas.getBoundingClientRect();
    const scaleX = DOC_WIDTH / rect.width;
    const scaleY = DOC_HEIGHT / rect.height;
    return {
      x: (e.clientX - rect.left) * scaleX,
      y: (e.clientY - rect.top) * scaleY,
    };
  }, []);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      if (!surfaceRef.current || !paintRef.current) return;
      if (tool === "brush" || tool === "eraser") {
        isDrawingRef.current = true;
        const pos = getCanvasPos(e);
        lastPosRef.current = pos;
        pointsRef.current = [[pos.x, pos.y]];
        canvasMonitor.log({ type: "mousedown", x: pos.x, y: pos.y, points: 1 });
        setDebugInfo((prev) => ({ ...prev, isDrawing: true, points: 1, lastX: pos.x, lastY: pos.y }));
      }
    },
    [tool, getCanvasPos]
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (!isDrawingRef.current || !lastPosRef.current || !surfaceRef.current || !paintRef.current || !canvasKit) return;
      const pos = getCanvasPos(e);
      pointsRef.current.push([pos.x, pos.y]);

      const ctx = surfaceRef.current.getCanvas();

      ctx.clear(canvasKit.Color4f(30 / 255, 32 / 255, 38 / 255, 1.0));
      if (bgImageRef.current) {
        ctx.drawImage(bgImageRef.current, 0, 0);
      }

      for (let i = 0; i < pointsRef.current.length - 1; i++) {
        const [x0, y0] = pointsRef.current[i];
        const [x1, y1] = pointsRef.current[i + 1];
        ctx.drawLine(x0, y0, x1, y1, paintRef.current);
      }

      surfaceRef.current.flush();
      lastPosRef.current = pos;

      canvasMonitor.log({
        type: "mousemove",
        x: pos.x,
        y: pos.y,
        points: pointsRef.current.length,
        paintColor: color,
        paintWidth: brushSize,
      });

      setDebugInfo((prev) => ({
        ...prev,
        points: pointsRef.current.length,
        lastX: pos.x,
        lastY: pos.y,
        bgReady: !!bgImageRef.current,
      }));
    },
    [getCanvasPos, canvasKit, color, brushSize]
  );

  const handleMouseUp = useCallback(async () => {
    if (!isDrawingRef.current) return;
    isDrawingRef.current = false;
    lastPosRef.current = null;

    canvasMonitor.log({ type: "mouseup", points: pointsRef.current.length });

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
        window.dispatchEvent(new CustomEvent("retas:state-changed"));
      } catch (e) {
        console.error("Draw failed:", e);
      }
    }

    pointsRef.current = [];
    needsRefreshRef.current = true;
    setDebugInfo((prev) => ({ ...prev, isDrawing: false, points: 0 }));
  }, [tool, color, brushSize]);

  if (isLoading) {
    return (
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          color: "#8b949e",
        }}
      >
        加载画布...
      </div>
    );
  }

  return (
    <div
      style={{
        position: "relative",
        width: "100%",
        height: "100%",
        overflow: "auto",
        background: "#161b22",
      }}
    >
      <canvas
        ref={canvasRef}
        data-testid="main-canvas"
        width={DOC_WIDTH}
        height={DOC_HEIGHT}
        style={{
          width: DOC_WIDTH,
          height: DOC_HEIGHT,
          cursor: getCursor(),
        }}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      />
      <div
        style={{
          position: "absolute",
          top: 8,
          right: 8,
          background: "rgba(0,0,0,0.7)",
          color: "#fff",
          padding: "6px 10px",
          borderRadius: 4,
          fontSize: 11,
          fontFamily: "monospace",
          lineHeight: 1.5,
          pointerEvents: "none",
          zIndex: 100,
          minWidth: 140,
        }}
      >
        <div>drawing: {debugInfo.isDrawing ? "yes" : "no"}</div>
        <div>points: {debugInfo.points}</div>
        <div>x: {debugInfo.lastX.toFixed(1)}</div>
        <div>y: {debugInfo.lastY.toFixed(1)}</div>
        <div>bg: {debugInfo.bgReady ? "ready" : "none"}</div>
        <div>tool: {debugInfo.tool}</div>
        <div>size: {debugInfo.brushSize}</div>
      </div>
    </div>
  );
}

function hexToRgb(hex: string): [number, number, number] {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? [parseInt(result[1], 16), parseInt(result[2], 16), parseInt(result[3], 16)]
    : [0, 0, 0];
}
