import { useEffect, useRef, useCallback, useState } from "react";
import { useCanvasKit } from "../hooks/useCanvasKit";

interface SelectionToolProps {
  zoom: number;
}

interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface TransformHandle {
  x: number;
  y: number;
  cursor: string;
  type: string;
}

export default function SelectionToolCanvas({ zoom }: SelectionToolProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const surfaceRef = useRef<any>(null);
  const { canvasKit, isLoading } = useCanvasKit();
  
  const [selection, setSelection] = useState<Rect | null>(null);
  const [isSelecting, setIsSelecting] = useState(false);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const [activeHandle, setActiveHandle] = useState<string | null>(null);
  const selectionRef = useRef<Rect | null>(null);
  
  // Store selection in ref for immediate access
  useEffect(() => {
    selectionRef.current = selection;
  }, [selection]);

  // Initialize canvas
  useEffect(() => {
    if (!canvasKit || !canvasRef.current) return;

    const canvas = canvasRef.current;
    canvas.width = 1920;
    canvas.height = 1080;

    const surface = canvasKit.MakeCanvasSurface(canvas);
    if (!surface) return;
    surfaceRef.current = surface;

    // Draw background
    const ctx = surface.getCanvas();
    ctx.clear(canvasKit.Color(255, 255, 255, 1));

    surface.flush();

    return () => {
      surface.delete();
    };
  }, [canvasKit]);

  // Redraw canvas with selection
  const redraw = useCallback(() => {
    if (!surfaceRef.current || !canvasKit) return;

    const ctx = surfaceRef.current.getCanvas();
    ctx.clear(canvasKit.Color(255, 255, 255, 1));

    // Draw selection
    const sel = selectionRef.current;
    if (sel) {
      // Selection rectangle
      const selPaint = new canvasKit.Paint();
      selPaint.setStyle(canvasKit.PaintStyle.Stroke);
      selPaint.setStrokeWidth(1);
      selPaint.setColor(canvasKit.Color(31, 111, 235, 1));
      
      const path = new canvasKit.Path();
      path.moveTo(sel.x, sel.y);
      path.lineTo(sel.x + sel.width, sel.y);
      path.lineTo(sel.x + sel.width, sel.y + sel.height);
      path.lineTo(sel.x, sel.y + sel.height);
      path.close();
      ctx.drawPath(path, selPaint);
      
      // Fill with semi-transparent blue
      const fillPaint = new canvasKit.Paint();
      fillPaint.setStyle(canvasKit.PaintStyle.Fill);
      fillPaint.setColor(canvasKit.Color(31, 111, 235, 0.1));
      ctx.drawPath(path, fillPaint);
      
      path.delete();
      selPaint.delete();
      fillPaint.delete();

      // Draw transform handles
      const handles = getTransformHandles(sel);
      const handlePaint = new canvasKit.Paint();
      handlePaint.setStyle(canvasKit.PaintStyle.Fill);
      handlePaint.setColor(canvasKit.Color(255, 255, 255, 1));
      
      const handleStroke = new canvasKit.Paint();
      handleStroke.setStyle(canvasKit.PaintStyle.Stroke);
      handleStroke.setStrokeWidth(1);
      handleStroke.setColor(canvasKit.Color(31, 111, 235, 1));

      handles.forEach(handle => {
        ctx.drawCircle(handle.x, handle.y, 4, handlePaint);
        ctx.drawCircle(handle.x, handle.y, 4, handleStroke);
      });

      handlePaint.delete();
      handleStroke.delete();
    }

    surfaceRef.current.flush();
  }, [canvasKit]);

  // Redraw when selection changes
  useEffect(() => {
    redraw();
  }, [selection, redraw]);

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

  const getTransformHandles = (rect: Rect): TransformHandle[] => {
    const { x, y, width, height } = rect;
    return [
      { x, y, cursor: "nw-resize", type: "nw" },
      { x: x + width / 2, y, cursor: "n-resize", type: "n" },
      { x: x + width, y, cursor: "ne-resize", type: "ne" },
      { x: x + width, y: y + height / 2, cursor: "e-resize", type: "e" },
      { x: x + width, y: y + height, cursor: "se-resize", type: "se" },
      { x: x + width / 2, y: y + height, cursor: "s-resize", type: "s" },
      { x, y: y + height, cursor: "sw-resize", type: "sw" },
      { x, y: y + height / 2, cursor: "w-resize", type: "w" },
    ];
  };

  const getHandleAtPos = (pos: { x: number; y: number }): string | null => {
    if (!selection) return null;
    const handles = getTransformHandles(selection);
    for (const handle of handles) {
      const dist = Math.sqrt((pos.x - handle.x) ** 2 + (pos.y - handle.y) ** 2);
      if (dist < 8) return handle.type;
    }
    return null;
  };

  const isInsideSelection = (pos: { x: number; y: number }): boolean => {
    if (!selection) return false;
    return (
      pos.x >= selection.x &&
      pos.x <= selection.x + selection.width &&
      pos.y >= selection.y &&
      pos.y <= selection.y + selection.height
    );
  };

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    const pos = getCanvasPos(e);
    const handle = getHandleAtPos(pos);
    
    if (handle) {
      setActiveHandle(handle);
      setDragStart(pos);
      setIsDragging(true);
    } else if (isInsideSelection(pos)) {
      setActiveHandle("move");
      setDragStart(pos);
      setIsDragging(true);
    } else {
      setIsSelecting(true);
      setDragStart(pos);
      setSelection({ x: pos.x, y: pos.y, width: 0, height: 0 });
    }
  }, [getCanvasPos, selection]);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    const pos = getCanvasPos(e);

    if (isSelecting) {
      setSelection({
        x: Math.min(dragStart.x, pos.x),
        y: Math.min(dragStart.y, pos.y),
        width: Math.abs(pos.x - dragStart.x),
        height: Math.abs(pos.y - dragStart.y),
      });
    } else if (isDragging && selection && activeHandle) {
      const dx = pos.x - dragStart.x;
      const dy = pos.y - dragStart.y;

      let newRect = { ...selection };

      switch (activeHandle) {
        case "move":
          newRect.x += dx;
          newRect.y += dy;
          break;
        case "se":
          newRect.width += dx;
          newRect.height += dy;
          break;
        case "nw":
          newRect.x += dx;
          newRect.y += dy;
          newRect.width -= dx;
          newRect.height -= dy;
          break;
        case "ne":
          newRect.y += dy;
          newRect.width += dx;
          newRect.height -= dy;
          break;
        case "sw":
          newRect.x += dx;
          newRect.width -= dx;
          newRect.height += dy;
          break;
        case "n":
          newRect.y += dy;
          newRect.height -= dy;
          break;
        case "s":
          newRect.height += dy;
          break;
        case "e":
          newRect.width += dx;
          break;
        case "w":
          newRect.x += dx;
          newRect.width -= dx;
          break;
      }

      // Prevent negative dimensions
      if (newRect.width < 0) {
        newRect.x += newRect.width;
        newRect.width = Math.abs(newRect.width);
      }
      if (newRect.height < 0) {
        newRect.y += newRect.height;
        newRect.height = Math.abs(newRect.height);
      }

      setSelection(newRect);
      setDragStart(pos);
    }
  }, [isSelecting, isDragging, dragStart, selection, activeHandle, getCanvasPos]);

  const handleMouseUp = useCallback(() => {
    if (isSelecting) {
      setIsSelecting(false);
      // If selection is too small, clear it
      if (selection && selection.width < 5 && selection.height < 5) {
        setSelection(null);
      }
    }
    if (isDragging) {
      setIsDragging(false);
      setActiveHandle(null);
    }
  }, [isSelecting, isDragging, selection]);

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
    <div style={{ position: "relative" }}>
      <canvas
        ref={canvasRef}
        style={{
          width: `${1920 * (zoom / 100)}px`,
          height: `${1080 * (zoom / 100)}px`,
          maxWidth: "100%",
          maxHeight: "100%",
          cursor: isDragging ? (activeHandle === "move" ? "move" : activeHandle ? "crosshair" : "default") : "crosshair",
        }}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      />
      <div style={{
        position: "absolute",
        bottom: 8,
        left: 8,
        background: "rgba(0,0,0,0.7)",
        color: "white",
        padding: "4px 8px",
        borderRadius: 4,
        fontSize: 12,
      }}>
        {selection 
          ? `选择: ${Math.round(selection.width)}x${Math.round(selection.height)}` 
          : "拖拽创建选区"}
      </div>
    </div>
  );
}
