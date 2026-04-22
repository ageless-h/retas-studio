import { useEffect, useRef, useCallback, useState } from "react";
import { useCanvasKit } from "../hooks/useCanvasKit";

interface PenToolProps {
  zoom: number;
}

interface PathPoint {
  x: number;
  y: number;
  cp1x?: number;
  cp1y?: number;
  cp2x?: number;
  cp2y?: number;
}

export default function PenToolCanvas({ zoom }: PenToolProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const surfaceRef = useRef<any>(null);
  const { canvasKit, isLoading } = useCanvasKit();
  const [paths, setPaths] = useState<PathPoint[][]>([]);
  const [currentPath, setCurrentPath] = useState<PathPoint[]>([]);
  const [isDrawing, setIsDrawing] = useState(false);

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

  // Redraw all paths
  useEffect(() => {
    if (!surfaceRef.current || !canvasKit) return;

    const ctx = surfaceRef.current.getCanvas();
    ctx.clear(canvasKit.Color(255, 255, 255, 1));

    // Draw completed paths
    const pathPaint = new canvasKit.Paint();
    pathPaint.setStyle(canvasKit.PaintStyle.Stroke);
    pathPaint.setStrokeWidth(2);
    pathPaint.setColor(canvasKit.Color(0, 0, 0, 1));
    pathPaint.setAntiAlias(true);

    paths.forEach(path => {
      if (path.length < 2) return;
      const skPath = new canvasKit.Path();
      skPath.moveTo(path[0].x, path[0].y);
      
      for (let i = 1; i < path.length; i++) {
        const p = path[i];
        if (p.cp1x !== undefined && p.cp1y !== undefined) {
          skPath.cubicTo(p.cp1x, p.cp1y, p.x, p.y, p.x, p.y);
        } else {
          skPath.lineTo(p.x, p.y);
        }
      }
      
      ctx.drawPath(skPath, pathPaint);
      skPath.delete();
    });

    // Draw current path
    if (currentPath.length > 0) {
      const skPath = new canvasKit.Path();
      skPath.moveTo(currentPath[0].x, currentPath[0].y);
      
      for (let i = 1; i < currentPath.length; i++) {
        skPath.lineTo(currentPath[i].x, currentPath[i].y);
      }
      
      const currentPaint = new canvasKit.Paint();
      currentPaint.setStyle(canvasKit.PaintStyle.Stroke);
      currentPaint.setStrokeWidth(2);
      currentPaint.setColor(canvasKit.Color(31, 111, 235, 1));
      currentPaint.setAntiAlias(true);
      
      ctx.drawPath(skPath, currentPaint);
      skPath.delete();
      currentPaint.delete();
    }

    // Draw anchor points
    const pointPaint = new canvasKit.Paint();
    pointPaint.setStyle(canvasKit.PaintStyle.Fill);
    pointPaint.setColor(canvasKit.Color(255, 255, 255, 1));
    
    const pointStroke = new canvasKit.Paint();
    pointStroke.setStyle(canvasKit.PaintStyle.Stroke);
    pointStroke.setStrokeWidth(2);
    pointStroke.setColor(canvasKit.Color(31, 111, 235, 1));

    [...paths, currentPath].forEach(path => {
      path.forEach(p => {
        ctx.drawCircle(p.x, p.y, 4, pointPaint);
        ctx.drawCircle(p.x, p.y, 4, pointStroke);
      });
    });

    surfaceRef.current.flush();
    pathPaint.delete();
    pointPaint.delete();
    pointStroke.delete();
  }, [paths, currentPath, canvasKit]);

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

  const handleClick = useCallback((e: React.MouseEvent) => {
    const pos = getCanvasPos(e);
    
    if (!isDrawing) {
      // Start new path
      setCurrentPath([{ x: pos.x, y: pos.y }]);
      setIsDrawing(true);
    } else {
      // Add point to current path
      setCurrentPath(prev => [...prev, { x: pos.x, y: pos.y }]);
    }
  }, [getCanvasPos, isDrawing]);

  const handleDoubleClick = useCallback(() => {
    if (currentPath.length > 1) {
      setPaths(prev => [...prev, currentPath]);
      setCurrentPath([]);
      setIsDrawing(false);
    }
  }, [currentPath]);

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
          cursor: "crosshair",
        }}
        onClick={handleClick}
        onDoubleClick={handleDoubleClick}
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
        {isDrawing 
          ? `绘制中: ${currentPath.length} 个点 (双击完成)` 
          : "单击开始绘制路径"}
      </div>
    </div>
  );
}
