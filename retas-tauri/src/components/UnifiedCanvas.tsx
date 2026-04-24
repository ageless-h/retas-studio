import { useEffect, useRef, useCallback, useState } from "react";
import { useCanvasKit } from "../hooks/useCanvasKit";
import { applyStrokePixels, compositeLayers, floodFillLayer, SelectionData, pickColor, moveLayerPixels } from "../api";
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
  showGrid?: boolean;
  showGuides?: boolean;
  selection?: SelectionData | null;
  selectionTool?: "rect" | "ellipse" | "lasso" | "magicWand";
  selectionMode?: "replace" | "add" | "subtract" | "intersect";
  onSelectionChange?: (selection: SelectionData | null) => void;
  onColorPick?: (color: string) => void;
  activeLayerId?: string;
}

const DOC_WIDTH = 1920;
const DOC_HEIGHT = 1080;

export default function UnifiedCanvas({
  tool,
  zoom: externalZoom,
  color = "#000000",
  brushSize = 2,
  onionSkin,
  currentFrame = 1,
  totalFrames = 100,
  showGrid = false,
  showGuides = false,
  selection: _selection,
  selectionTool = "rect",
  selectionMode = "replace",
  onSelectionChange,
  onColorPick,
  activeLayerId,
}: UnifiedCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { canvasKit, isLoading } = useCanvasKit();

  const surfaceRef = useRef<any>(null);
  const paintRef = useRef<any>(null);
  const bgImageRef = useRef<any>(null);
  const rafRef = useRef<number>(0);
  const onionSkinCacheRef = useRef<Map<number, any>>(new Map());
  const strokeSurfaceRef = useRef<any>(null);
  const strokePaintRef = useRef<any>(null);

  const isDrawingRef = useRef(false);
  const pointsRef = useRef<[number, number, number][]>([]); // [x, y, pressure]
  const lastPosRef = useRef<{ x: number; y: number } | null>(null);
  const maxPressureRef = useRef(1.0);
  const needsRefreshRef = useRef(true);
  const isRefreshingRef = useRef(false);

  // Zoom & pan state
  const zoomRef = useRef(1.0);
  const panRef = useRef({ x: 0, y: 0 });
  const isPanningRef = useRef(false);
  const panStartRef = useRef({ x: 0, y: 0, panX: 0, panY: 0 });
  const spaceHeldRef = useRef(false);
  const [viewZoom, setViewZoom] = useState(1.0);

  const isSelectingRef = useRef(false);
  const selectionStartRef = useRef<{ x: number; y: number } | null>(null);
  const selectionPointsRef = useRef<{ x: number; y: number }[]>([]);
  const [currentSelectionRect, setCurrentSelectionRect] = useState<{ x: number; y: number; width: number; height: number } | null>(null);

  const isMovingRef = useRef(false);
  const moveStartRef = useRef<{ x: number; y: number } | null>(null);

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

  // Sync external zoom (from toolbar +/- buttons) to internal zoom
  useEffect(() => {
    if (externalZoom !== undefined) {
      const z = externalZoom / 100;
      zoomRef.current = z;
      setViewZoom(z);
    }
  }, [externalZoom]);

  // Space key tracking for pan mode
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.code === "Space" && !e.repeat) {
        e.preventDefault();
        spaceHeldRef.current = true;
      }
    };
    const handleKeyUp = (e: KeyboardEvent) => {
      if (e.code === "Space") {
        spaceHeldRef.current = false;
        isPanningRef.current = false;
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
    };
  }, []);

  const getCursor = () => {
    if (spaceHeldRef.current || tool === "hand") {
      return isPanningRef.current ? "grabbing" : "grab";
    }
    switch (tool) {
      case "brush":
      case "eraser":
        return "crosshair";
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

    const strokeCanvas = document.createElement('canvas');
    strokeCanvas.width = DOC_WIDTH;
    strokeCanvas.height = DOC_HEIGHT;
    const strokeSurface = canvasKit.MakeCanvasSurface(strokeCanvas);
    if (strokeSurface) {
      strokeSurfaceRef.current = strokeSurface;
      const strokePaint = new canvasKit.Paint();
      strokePaint.setStyle(canvasKit.PaintStyle.Stroke);
      strokePaint.setAntiAlias(true);
      strokePaintRef.current = strokePaint;
    }

    needsRefreshRef.current = true;

    return () => {
      if (rafRef.current) cancelAnimationFrame(rafRef.current);
      try { surface.delete(); } catch {}
      try { paint.delete(); } catch {}
      if (bgImageRef.current) {
        try { bgImageRef.current.delete(); } catch {}
      }
      if (strokeSurfaceRef.current) {
        try { strokeSurfaceRef.current.delete(); } catch {}
      }
      if (strokePaintRef.current) {
        try { strokePaintRef.current.delete(); } catch {}
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
    
    if (strokePaintRef.current) {
      if (tool === "eraser") {
        strokePaintRef.current.setColor(canvasKit.Color4f(0, 0, 0, 0));
        strokePaintRef.current.setBlendMode(canvasKit.BlendMode.DstOut);
        strokePaintRef.current.setStrokeWidth(20);
      } else {
        strokePaintRef.current.setColor(canvasKit.Color4f(rgb[0] / 255, rgb[1] / 255, rgb[2] / 255, 1.0));
        strokePaintRef.current.setBlendMode(canvasKit.BlendMode.SrcOver);
        strokePaintRef.current.setStrokeWidth(brushSize);
      }
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
    
    // Grid overlay
    if (showGrid) {
      const gridPaint = new canvasKit.Paint();
      gridPaint.setStyle(canvasKit.PaintStyle.Stroke);
      gridPaint.setStrokeWidth(0.5);
      gridPaint.setColor(canvasKit.Color4f(1, 1, 1, 0.08));
      const gridSize = 64;
      for (let x = gridSize; x < DOC_WIDTH; x += gridSize) {
        ctx.drawLine(x, 0, x, DOC_HEIGHT, gridPaint);
      }
      for (let y = gridSize; y < DOC_HEIGHT; y += gridSize) {
        ctx.drawLine(0, y, DOC_WIDTH, y, gridPaint);
      }
      gridPaint.delete();
    }
    
    // Center guides
    if (showGuides) {
      const guidePaint = new canvasKit.Paint();
      guidePaint.setStyle(canvasKit.PaintStyle.Stroke);
      guidePaint.setStrokeWidth(1);
      guidePaint.setColor(canvasKit.Color4f(0, 1, 1, 0.4));
      // Horizontal center
      ctx.drawLine(0, DOC_HEIGHT / 2, DOC_WIDTH, DOC_HEIGHT / 2, guidePaint);
      // Vertical center
      ctx.drawLine(DOC_WIDTH / 2, 0, DOC_WIDTH / 2, DOC_HEIGHT, guidePaint);
      // Rule of thirds
      guidePaint.setColor(canvasKit.Color4f(1, 0.5, 0, 0.2));
      ctx.drawLine(DOC_WIDTH / 3, 0, DOC_WIDTH / 3, DOC_HEIGHT, guidePaint);
      ctx.drawLine(DOC_WIDTH * 2 / 3, 0, DOC_WIDTH * 2 / 3, DOC_HEIGHT, guidePaint);
      ctx.drawLine(0, DOC_HEIGHT / 3, DOC_WIDTH, DOC_HEIGHT / 3, guidePaint);
      ctx.drawLine(0, DOC_HEIGHT * 2 / 3, DOC_WIDTH, DOC_HEIGHT * 2 / 3, guidePaint);
      guidePaint.delete();
    }
    
    surfaceRef.current.flush();
  }, [canvasKit, onionSkin, showGrid, showGuides]);

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
        if (tintPaint) {
          ctx.drawImage(cachedImage, 0, 0, tintPaint);
        } else {
          ctx.drawImage(cachedImage, 0, 0);
        }
        ctx.restore();
        
        if (tintPaint) tintPaint.delete();
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
        if (tintPaint) {
          ctx.drawImage(cachedImage, 0, 0, tintPaint);
        } else {
          ctx.drawImage(cachedImage, 0, 0);
        }
        ctx.restore();
        
        if (tintPaint) tintPaint.delete();
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

  const getCanvasPos = useCallback((e: React.PointerEvent | React.MouseEvent) => {
    const canvas = canvasRef.current;
    if (!canvas) return { x: 0, y: 0, pressure: 0.5 };
    const rect = canvas.getBoundingClientRect();
    // The canvas element is scaled by CSS transform, so rect already accounts for visual size.
    // Map client coords → document coords by inversing the CSS transform.
    const zoom = zoomRef.current;
    const pan = panRef.current;
    const scaleX = DOC_WIDTH / (DOC_WIDTH * zoom);
    const scaleY = DOC_HEIGHT / (DOC_HEIGHT * zoom);
    const clientX = e.clientX - rect.left;
    const clientY = e.clientY - rect.top;
    // rect.width = DOC_WIDTH * zoom, so scaleX = DOC_WIDTH / rect.width
    const docX = (clientX / rect.width) * DOC_WIDTH;
    const docY = (clientY / rect.height) * DOC_HEIGHT;
    const pressure = "pressure" in e ? (e as React.PointerEvent).pressure || 0.5 : 0.5;
    return { x: docX, y: docY, pressure };
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

  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const newZoom = Math.max(0.1, Math.min(10, zoomRef.current * delta));
    
    // Zoom toward cursor position
    const container = canvasRef.current?.parentElement;
    if (container) {
      const rect = container.getBoundingClientRect();
      const cx = e.clientX - rect.left;
      const cy = e.clientY - rect.top;
      // Adjust pan so the point under the cursor stays fixed
      const scale = newZoom / zoomRef.current;
      panRef.current = {
        x: cx - (cx - panRef.current.x) * scale,
        y: cy - (cy - panRef.current.y) * scale,
      };
    }
    
    zoomRef.current = newZoom;
    setViewZoom(newZoom);
  }, []);

  const handleMouseDown = useCallback(
    (e: React.PointerEvent) => {
      if (!surfaceRef.current || !paintRef.current) return;
      
      // Pan mode: space+drag or hand tool
      if (spaceHeldRef.current || tool === "hand") {
        isPanningRef.current = true;
        panStartRef.current = {
          x: e.clientX,
          y: e.clientY,
          panX: panRef.current.x,
          panY: panRef.current.y,
        };
        return;
      }
      
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
      
      if (tool === "fill") {
        const pos = getCanvasPos(e);
        // Parse hex color to RGBA tuple
        const hex = color.replace("#", "");
        const r = parseInt(hex.substring(0, 2), 16) || 0;
        const g = parseInt(hex.substring(2, 4), 16) || 0;
        const b = parseInt(hex.substring(4, 6), 16) || 0;
        
        floodFillLayer(Math.floor(pos.x), Math.floor(pos.y), [r, g, b, 255], 32)
          .then(() => {
            window.dispatchEvent(new CustomEvent("retas:state-changed"));
          })
          .catch((err) => console.error("Fill failed:", err));
        return;
      }
      
      if (tool === "eyedropper") {
        const pos = getCanvasPos(e);
        const px = Math.floor(pos.x);
        const py = Math.floor(pos.y);
        if (activeLayerId && onColorPick) {
          pickColor(px, py, activeLayerId, (currentFrame || 1) - 1)
            .then(([r, g, b, _a]) => {
              const hex = "#" + [r, g, b].map(c => c.toString(16).padStart(2, "0")).join("");
              onColorPick(hex);
            })
            .catch((err) => console.error("Eyedropper failed:", err));
        }
        return;
      }
      
      if (tool === "move") {
        isMovingRef.current = true;
        const pos = getCanvasPos(e);
        moveStartRef.current = { x: pos.x, y: pos.y };
        return;
      }
      
      if (tool === "brush" || tool === "eraser") {
        isDrawingRef.current = true;
        const pos = getCanvasPos(e);
        lastPosRef.current = pos;
        pointsRef.current = [[pos.x, pos.y, pos.pressure]];
        maxPressureRef.current = pos.pressure;
        
        if (strokeSurfaceRef.current && canvasKit) {
          const strokeCtx = strokeSurfaceRef.current.getCanvas();
          strokeCtx.clear(canvasKit.Color4f(0, 0, 0, 0));
          strokeSurfaceRef.current.flush();
        }
        
        canvasMonitor.log({ type: "mousedown", x: pos.x, y: pos.y, points: 1 });
        setDebugInfo((prev) => ({ ...prev, isDrawing: true, points: 1, lastX: pos.x, lastY: pos.y }));
      }
    },
    [tool, selectionTool, getCanvasPos]
  );

  const handleMouseMove = useCallback(
    (e: React.PointerEvent) => {
      if (!surfaceRef.current || !canvasKit) return;
      
      // Pan dragging
      if (isPanningRef.current) {
        panRef.current = {
          x: panStartRef.current.panX + (e.clientX - panStartRef.current.x),
          y: panStartRef.current.panY + (e.clientY - panStartRef.current.y),
        };
        // Force re-render for CSS transform update
        setViewZoom(zoomRef.current);
        return;
      }
      
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
      
      if (tool === "move" && isMovingRef.current && moveStartRef.current && activeLayerId) {
        const pos = getCanvasPos(e);
        const dx = Math.round(pos.x - moveStartRef.current.x);
        const dy = Math.round(pos.y - moveStartRef.current.y);
        if (Math.abs(dx) >= 1 || Math.abs(dy) >= 1) {
          moveStartRef.current = { x: pos.x, y: pos.y };
          moveLayerPixels(activeLayerId, dx, dy)
            .then(() => {
              window.dispatchEvent(new CustomEvent("retas:state-changed"));
            })
            .catch((err) => console.error("Move failed:", err));
        }
        return;
      }
      
      if (!isDrawingRef.current || !lastPosRef.current || !paintRef.current) return;
      const pos = getCanvasPos(e);
      
      const lastPos = lastPosRef.current;
      pointsRef.current.push([pos.x, pos.y, pos.pressure]);
      if (pos.pressure > maxPressureRef.current) maxPressureRef.current = pos.pressure;

      if (strokeSurfaceRef.current && strokePaintRef.current) {
        const strokeCtx = strokeSurfaceRef.current.getCanvas();
        // Modulate stroke width by pressure (0.3–1.0 range to avoid invisible strokes)
        const pressureScale = 0.3 + 0.7 * pos.pressure;
        strokePaintRef.current.setStrokeWidth(brushSize * pressureScale);
        strokeCtx.drawLine(lastPos.x, lastPos.y, pos.x, pos.y, strokePaintRef.current);
        strokeSurfaceRef.current.flush();
      }

      const ctx = surfaceRef.current.getCanvas();
      ctx.clear(canvasKit.Color4f(30 / 255, 32 / 255, 38 / 255, 1.0));
      
      if (bgImageRef.current) {
        ctx.drawImage(bgImageRef.current, 0, 0);
      }
      
      if (strokeSurfaceRef.current) {
        const strokeImage = strokeSurfaceRef.current.makeImageSnapshot();
        if (strokeImage) {
          ctx.drawImage(strokeImage, 0, 0);
          strokeImage.delete();
        }
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
      // Stop panning
      if (isPanningRef.current) {
        isPanningRef.current = false;
        return;
      }
      
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
      
      if (isMovingRef.current && moveStartRef.current) {
        isMovingRef.current = false;
        // Move is committed on mouse up (the visual offset was a preview)
        moveStartRef.current = null;
        return;
      }
      
      if (!isDrawingRef.current) return;
      isDrawingRef.current = false;
      lastPosRef.current = null;

      canvasMonitor.log({ type: "mouseup", points: pointsRef.current.length });

      if (pointsRef.current.length > 1) {
        try {
          if (!strokeSurfaceRef.current || !canvasKit) {
            console.warn("strokeSurface not initialized");
            return;
          }
          
          const strokeImage = strokeSurfaceRef.current.makeImageSnapshot();
          if (strokeImage) {
            const pixels = strokeImage.readPixels(0, 0, {
              width: DOC_WIDTH,
              height: DOC_HEIGHT,
              colorType: canvasKit.ColorType.RGBA_8888,
              alphaType: canvasKit.AlphaType.Unpremul,
              colorSpace: canvasKit.ColorSpace.SRGB,
            });
            
            if (pixels) {
              const fullPixels = new Uint8Array(pixels);
              const sparsePixels: { x: number; y: number; r: number; g: number; b: number; a: number }[] = [];
              
              // Compute stroke bounding box from tracked points + brush radius padding
              // to avoid scanning the entire frame (#19 performance fix)
              const pts = pointsRef.current;
              let minX = DOC_WIDTH, minY = DOC_HEIGHT, maxX = 0, maxY = 0;
              for (const [px, py] of pts) {
                if (px < minX) minX = px;
                if (py < minY) minY = py;
                if (px > maxX) maxX = px;
                if (py > maxY) maxY = py;
              }
              const pad = Math.ceil(brushSize * maxPressureRef.current) + 2;
              const scanX0 = Math.max(0, Math.floor(minX) - pad);
              const scanY0 = Math.max(0, Math.floor(minY) - pad);
              const scanX1 = Math.min(DOC_WIDTH, Math.ceil(maxX) + pad);
              const scanY1 = Math.min(DOC_HEIGHT, Math.ceil(maxY) + pad);
              
              for (let y = scanY0; y < scanY1; y++) {
                for (let x = scanX0; x < scanX1; x++) {
                  const idx = (y * DOC_WIDTH + x) * 4;
                  const alpha = fullPixels[idx + 3];
                  if (alpha > 0) {
                    sparsePixels.push({
                      x,
                      y,
                      r: fullPixels[idx],
                      g: fullPixels[idx + 1],
                      b: fullPixels[idx + 2],
                      a: alpha,
                    });
                  }
                }
              }
              
              await applyStrokePixels(sparsePixels);
            }
            strokeImage.delete();
          }
          window.dispatchEvent(new CustomEvent("retas:state-changed"));
        } catch (e) {
          console.error("Draw failed:", e);
        }
      }

      pointsRef.current = [];
      needsRefreshRef.current = true;
      setDebugInfo((prev) => ({ ...prev, isDrawing: false, points: 0 }));
    }, [tool, selectionTool, selectionMode, currentSelectionRect, onSelectionChange, color, brushSize, canvasKit]);

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
        overflow: "hidden",
        background: "#161b22",
      }}
      onWheel={handleWheel}
    >
      <div
        style={{
          position: "absolute",
          transform: `translate(${panRef.current.x}px, ${panRef.current.y}px) scale(${viewZoom})`,
          transformOrigin: "0 0",
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
            touchAction: "none",
          }}
          onPointerDown={handleMouseDown}
          onPointerMove={handleMouseMove}
          onPointerUp={handleMouseUp}
          onPointerLeave={handleMouseUp}
        />
      </div>
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
          <div>zoom: {(viewZoom * 100).toFixed(0)}%</div>
        </div>
      )}
      {/* Top ruler */}
      {showGuides && (
        <div style={{
          position: "absolute", top: 0, left: 20, right: 0, height: 20,
          background: "#0d1117", borderBottom: "1px solid #30363d",
          overflow: "hidden", pointerEvents: "none", zIndex: 50,
        }}>
          <svg width="100%" height="20" style={{ display: "block" }}>
            {(() => {
              const marks: JSX.Element[] = [];
              const step = viewZoom >= 2 ? 50 : viewZoom >= 0.5 ? 100 : 200;
              for (let px = 0; px <= DOC_WIDTH; px += step) {
                const screenX = px * viewZoom + panRef.current.x - 20;
                if (screenX < -50 || screenX > 3000) continue;
                marks.push(
                  <g key={px}>
                    <line x1={screenX} y1={14} x2={screenX} y2={20} stroke="#484f58" strokeWidth={1} />
                    <text x={screenX + 2} y={12} fill="#8b949e" fontSize={9} fontFamily="monospace">{px}</text>
                  </g>
                );
              }
              return marks;
            })()}
          </svg>
        </div>
      )}
      {/* Left ruler */}
      {showGuides && (
        <div style={{
          position: "absolute", top: 20, left: 0, bottom: 0, width: 20,
          background: "#0d1117", borderRight: "1px solid #30363d",
          overflow: "hidden", pointerEvents: "none", zIndex: 50,
        }}>
          <svg width="20" height="100%" style={{ display: "block" }}>
            {(() => {
              const marks: JSX.Element[] = [];
              const step = viewZoom >= 2 ? 50 : viewZoom >= 0.5 ? 100 : 200;
              for (let py = 0; py <= DOC_HEIGHT; py += step) {
                const screenY = py * viewZoom + panRef.current.y - 20;
                if (screenY < -50 || screenY > 3000) continue;
                marks.push(
                  <g key={py}>
                    <line x1={14} y1={screenY} x2={20} y2={screenY} stroke="#484f58" strokeWidth={1} />
                    <text x={2} y={screenY - 2} fill="#8b949e" fontSize={8} fontFamily="monospace"
                      transform={`rotate(-90, 2, ${screenY - 2})`}>{py}</text>
                  </g>
                );
              }
              return marks;
            })()}
          </svg>
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
