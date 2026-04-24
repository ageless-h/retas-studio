import { useEffect, useRef, useCallback, useState } from "react";
import { useCanvasKit } from "../hooks/useCanvasKit";
import { drawStroke, compositeLayers, DrawCommand, SelectionData } from "../api";
import { canvasMonitor } from "../utils/CanvasMonitor";

export interface OnionSkinSettings {
  enabled: boolean;
  framesBefore: number;
  framesAfter: number;
  opacityBefore: number;
  opacityAfter: number;
  colorBefore: string;
  colorAfter: string;
  blendMode: "tint" | "overlay" | "difference" | "normal";
}

interface UnifiedCanvasProps {
  tool: string;
  zoom: number;
  color?: string;
  brushSize?: number;
  onionSkin?: OnionSkinSettings;
  currentFrame?: number;
  totalFrames?: number;
  selection?: SelectionData | null;
  selectionTool?: "rect" | "ellipse" | "lasso" | "magicWand";
  selectionMode?: "replace" | "add" | "subtract" | "intersect";
  onSelectionChange?: (selection: SelectionData | null) => void;
}

const DOC_WIDTH = 1920;
const DOC_HEIGHT = 1080;

export default function UnifiedCanvas({
  tool,
  zoom: _zoom,
  color = "#000000",
  brushSize = 2,
  onionSkin,
  currentFrame = 1,
  totalFrames = 100,
  selection: _selection,
  selectionTool = "rect",
  selectionMode = "replace",
  onSelectionChange,
}: UnifiedCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { canvasKit, isLoading } = useCanvasKit();

  const surfaceRef = useRef<any>(null);
  const paintRef = useRef<any>(null);
  const bgImageRef = useRef<any>(null);
  const rafRef = useRef<number>(0);
  const onionSkinCacheRef = useRef<Map<number, any>>(new Map());

  const isDrawingRef = useRef(false);
  const pointsRef = useRef<[number, number][]>([]);
  const lastPosRef = useRef<{ x: number; y: number } | null>(null);
  const needsRefreshRef = useRef(true);
  const isRefreshingRef = useRef(false);

  const isSelectingRef = useRef(false);
  const selectionStartRef = useRef<{ x: number; y: number } | null>(null);
  const selectionPointsRef = useRef<{ x: number; y: number }[]>([]);
  const [currentSelectionRect, setCurrentSelectionRect] = useState<{ x: number; y: number; width: number; height: number } | null>(null);

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
        return "crosshair";
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
  }, [canvasKit]);

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
    
    if (onionSkin?.enabled && onionSkinCacheRef.current.size > 0) {
      renderOnionSkin(ctx);
    }
    
    if (bgImageRef.current) {
      ctx.drawImage(bgImageRef.current, 0, 0);
    }
    surfaceRef.current.flush();
  }, [canvasKit, onionSkin]);

  const renderOnionSkin = useCallback((ctx: any) => {
    if (!canvasKit || !onionSkin?.enabled) return;
    
    const cache = onionSkinCacheRef.current;
    
    for (let i = 1; i <= onionSkin.framesBefore; i++) {
      const frame = currentFrame - i;
      if (frame < 1) break;
      
      const cachedImage = cache.get(frame);
      if (cachedImage) {
        const opacity = onionSkin.opacityBefore * (1 - (i - 1) / Math.max(1, onionSkin.framesBefore));
        const tintPaint = createTintPaint(onionSkin.colorBefore, opacity);
        
        ctx.save();
        ctx.drawImage(cachedImage, 0, 0);
        ctx.restore();
        
        tintPaint.delete();
      }
    }
    
    for (let i = 1; i <= onionSkin.framesAfter; i++) {
      const frame = currentFrame + i;
      if (frame > totalFrames) break;
      
      const cachedImage = cache.get(frame);
      if (cachedImage) {
        const opacity = onionSkin.opacityAfter * (1 - (i - 1) / Math.max(1, onionSkin.framesAfter));
        const tintPaint = createTintPaint(onionSkin.colorAfter, opacity);
        
        ctx.save();
        ctx.drawImage(cachedImage, 0, 0);
        ctx.restore();
        
        tintPaint.delete();
      }
    }
  }, [canvasKit, onionSkin, currentFrame, totalFrames]);

  const createTintPaint = useCallback((hexColor: string, opacity: number) => {
    if (!canvasKit) return null;
    
    const paint = new canvasKit.Paint();
    const rgb = hexToRgb(hexColor);
    paint.setColor(canvasKit.Color4f(rgb[0] / 255, rgb[1] / 255, rgb[2] / 255, opacity));
    paint.setBlendMode(canvasKit.BlendMode.Multiply);
    return paint;
  }, [canvasKit]);

  useEffect(() => {
    if (!canvasKit || !surfaceRef.current || !bgImageRef.current) return;
    
    const cache = onionSkinCacheRef.current;
    const currentImage = bgImageRef.current;
    cache.set(currentFrame, currentImage);
    
    if (cache.size > 20) {
      const oldestKey = cache.keys().next().value;
      if (oldestKey !== undefined) {
        const oldImage = cache.get(oldestKey);
        if (oldImage && oldImage !== currentImage) {
          try { oldImage.delete(); } catch {}
        }
        cache.delete(oldestKey);
      }
    }
  }, [canvasKit, currentFrame]);

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

  const drawSelectionPreview = useCallback(() => {
    if (!surfaceRef.current || !canvasKit) return;
    const ctx = surfaceRef.current.getCanvas();
    
    ctx.clear(canvasKit.Color4f(30 / 255, 32 / 255, 38 / 255, 1.0));
    if (bgImageRef.current) {
      ctx.drawImage(bgImageRef.current, 0, 0);
    }
    
    const paint = new canvasKit.Paint();
    paint.setStyle(canvasKit.PaintStyle.Stroke);
    paint.setStrokeWidth(1);
    paint.setColor(canvasKit.Color4f(0, 0.5, 1, 1));
    paint.setAntiAlias(true);
    
    if (selectionTool === "lasso" && selectionPointsRef.current.length > 1) {
      const path = new canvasKit.Path();
      path.moveTo(selectionPointsRef.current[0].x, selectionPointsRef.current[0].y);
      for (let i = 1; i < selectionPointsRef.current.length; i++) {
        path.lineTo(selectionPointsRef.current[i].x, selectionPointsRef.current[i].y);
      }
      ctx.drawPath(path, paint);
      path.delete();
    } else if (currentSelectionRect) {
      if (selectionTool === "ellipse") {
        ctx.drawOval(
          canvasKit.LTRBRect(
            currentSelectionRect.x,
            currentSelectionRect.y,
            currentSelectionRect.x + currentSelectionRect.width,
            currentSelectionRect.y + currentSelectionRect.height
          ),
          paint
        );
      } else {
        ctx.drawRect(
          canvasKit.LTRBRect(
            currentSelectionRect.x,
            currentSelectionRect.y,
            currentSelectionRect.x + currentSelectionRect.width,
            currentSelectionRect.y + currentSelectionRect.height
          ),
          paint
        );
      }
    }
    
    paint.delete();
    surfaceRef.current.flush();
  }, [canvasKit, selectionTool, currentSelectionRect]);

  const handleMouseDown = useCallback(
    (e: React.MouseEvent) => {
      if (!surfaceRef.current || !paintRef.current) return;
      
      if (tool === "select") {
        isSelectingRef.current = true;
        const pos = getCanvasPos(e);
        selectionStartRef.current = pos;
        
        if (selectionTool === "lasso") {
          selectionPointsRef.current = [{ x: pos.x, y: pos.y }];
        } else {
          setCurrentSelectionRect({ x: pos.x, y: pos.y, width: 0, height: 0 });
        }
        return;
      }
      
      if (tool === "brush" || tool === "eraser") {
        isDrawingRef.current = true;
        const pos = getCanvasPos(e);
        lastPosRef.current = pos;
        pointsRef.current = [[pos.x, pos.y]];
        canvasMonitor.log({ type: "mousedown", x: pos.x, y: pos.y, points: 1 });
        setDebugInfo((prev) => ({ ...prev, isDrawing: true, points: 1, lastX: pos.x, lastY: pos.y }));
      }
    },
    [tool, selectionTool, getCanvasPos]
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (!surfaceRef.current || !canvasKit) return;
      
      if (tool === "select" && isSelectingRef.current && selectionStartRef.current) {
        const pos = getCanvasPos(e);
        
        if (selectionTool === "lasso") {
          selectionPointsRef.current.push({ x: pos.x, y: pos.y });
          drawSelectionPreview();
        } else {
          const rect = {
            x: Math.min(selectionStartRef.current.x, pos.x),
            y: Math.min(selectionStartRef.current.y, pos.y),
            width: Math.abs(pos.x - selectionStartRef.current.x),
            height: Math.abs(pos.y - selectionStartRef.current.y),
          };
          setCurrentSelectionRect(rect);
          drawSelectionPreview();
        }
        return;
      }
      
      if (!isDrawingRef.current || !lastPosRef.current || !paintRef.current) return;
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
    [tool, selectionTool, getCanvasPos, canvasKit, color, brushSize]
  );

  const handleMouseUp = useCallback(async () => {
      if (tool === "select" && isSelectingRef.current) {
        isSelectingRef.current = false;
        
        if (selectionTool === "lasso" && selectionPointsRef.current.length > 2 && onSelectionChange) {
          const closedPoints = [...selectionPointsRef.current, selectionPointsRef.current[0]];
          onSelectionChange({
            type: "lasso",
            mode: selectionMode,
            points: closedPoints,
            feather: 0,
            tolerance: 32,
            contiguous: true,
          });
        } else if (currentSelectionRect && currentSelectionRect.width > 0 && currentSelectionRect.height > 0 && onSelectionChange) {
          onSelectionChange({
            type: selectionTool as "rect" | "ellipse",
            mode: selectionMode,
            points: [],
            rect: currentSelectionRect,
            feather: 0,
            tolerance: 32,
            contiguous: true,
          });
        }
        
        selectionStartRef.current = null;
        selectionPointsRef.current = [];
        return;
      }
      
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
    }, [tool, selectionTool, selectionMode, currentSelectionRect, onSelectionChange, color, brushSize]);

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
      {import.meta.env.MODE === "development" && (
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
      )}
    </div>
  );
}

function hexToRgb(hex: string): [number, number, number] {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? [parseInt(result[1], 16), parseInt(result[2], 16), parseInt(result[3], 16)]
    : [0, 0, 0];
}
