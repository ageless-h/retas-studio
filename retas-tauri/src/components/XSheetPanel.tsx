import { useState, useRef, useCallback, useMemo, useEffect } from "react";
import { LayerInfo } from "../api";

interface XSheetPanelProps {
  layers: LayerInfo[];
  currentFrame: number;
  totalFrames: number;
  keyframes: Map<string, Set<number>>;
  onFrameSelect: (frame: number) => void;
  onKeyframeToggle: (layerId: string, frame: number) => void;
  onFrameInsert: (frame: number) => void;
  onFrameDelete: (frame: number) => void;
  onFrameCopy?: (frame: number) => void;
}

const CELL_WIDTH = 32;
const CELL_HEIGHT = 24;
const LAYER_COLUMN_WIDTH = 120;
const HEADER_HEIGHT = 28;
const OVERSCAN = 10;

const COLORS = {
  background: "#1a1a1e",
  backgroundAlt: "#1e1e22",
  backgroundAlt2: "#222226",
  headerBg: "#252529",
  headerText: "#8b949e",
  cellBorder: "#2d2d32",
  cellText: "#c9d1d9",
  currentFrameBorder: "#1f6feb",
  currentFrameGlow: "rgba(31, 111, 235, 0.3)",
  keyframe: "#3fb950",
  keyframeGlow: "rgba(63, 185, 80, 0.4)",
  selectedCell: "#388bfd",
  hoverCell: "#30363d",
  layerNameActive: "#58a6ff",
  rowEven: "#1a1a1e",
  rowOdd: "#16161a",
  contextMenu: "#21262d",
};

interface ContextMenuState {
  isOpen: boolean;
  frame: number;
  x: number;
  y: number;
}

export default function XSheetPanel({
  layers,
  currentFrame,
  totalFrames,
  keyframes,
  onFrameSelect,
  onKeyframeToggle,
  onFrameInsert,
  onFrameDelete,
  onFrameCopy,
}: XSheetPanelProps) {
  const [hoveredCell, setHoveredCell] = useState<{ layerId: string; frame: number } | null>(null);
  const [selectedCell, setSelectedCell] = useState<{ layerId: string; frame: number } | null>(null);
  const [columnWidth, setColumnWidth] = useState(CELL_WIDTH);
  const [isResizing, setIsResizing] = useState(false);
  const [scrollOffset, setScrollOffset] = useState({ x: 0, y: 0 });
  const [contextMenu, setContextMenu] = useState<ContextMenuState>({ isOpen: false, frame: 0, x: 0, y: 0 });
  
  const containerRef = useRef<HTMLDivElement>(null);
  const headerRef = useRef<HTMLDivElement>(null);
  const bodyRef = useRef<HTMLDivElement>(null);
  const layerColumnRef = useRef<HTMLDivElement>(null);
  const resizeRef = useRef<number>(0);
  const startXRef = useRef<number>(0);

  const visibleRange = useMemo(() => {
    if (!containerRef.current) {
      return { startFrame: 1, endFrame: Math.min(50, totalFrames), startLayer: 0, endLayer: layers.length };
    }
    
    const containerWidth = containerRef.current.clientWidth - LAYER_COLUMN_WIDTH;
    const containerHeight = containerRef.current.clientHeight - HEADER_HEIGHT;
    
    const startFrame = Math.max(1, Math.floor(scrollOffset.x / columnWidth) - OVERSCAN + 1);
    const endFrame = Math.min(totalFrames, Math.ceil((scrollOffset.x + containerWidth) / columnWidth) + OVERSCAN);
    
    const startLayer = Math.max(0, Math.floor(scrollOffset.y / CELL_HEIGHT) - OVERSCAN);
    const endLayer = Math.min(layers.length - 1, Math.ceil((scrollOffset.y + containerHeight) / CELL_HEIGHT) + OVERSCAN);
    
    return { startFrame, endFrame, startLayer, endLayer };
  }, [scrollOffset, totalFrames, layers.length, columnWidth]);

  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    const target = e.target as HTMLDivElement;
    setScrollOffset({ x: target.scrollLeft, y: target.scrollTop });
    if (headerRef.current && target !== headerRef.current) {
      headerRef.current.scrollLeft = target.scrollLeft;
    }
    if (layerColumnRef.current && target !== layerColumnRef.current) {
      layerColumnRef.current.scrollTop = target.scrollTop;
    }
  }, []);

  const handleCellClick = useCallback((layerId: string, frame: number, e: React.MouseEvent) => {
    e.stopPropagation();
    setSelectedCell({ layerId, frame });
    onFrameSelect(frame);
  }, [onFrameSelect]);

  const handleCellDoubleClick = useCallback((layerId: string, frame: number, e: React.MouseEvent) => {
    e.stopPropagation();
    onKeyframeToggle(layerId, frame);
  }, [onKeyframeToggle]);

  const handleContextMenu = useCallback((e: React.MouseEvent, frame: number) => {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({ isOpen: true, frame, x: e.clientX, y: e.clientY });
  }, []);

  const closeContextMenu = useCallback(() => {
    setContextMenu(prev => ({ ...prev, isOpen: false }));
  }, []);

  const handleResizeStart = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
    startXRef.current = e.clientX;
    resizeRef.current = columnWidth;
    
    const handleMouseMove = (moveEvent: MouseEvent) => {
      const delta = moveEvent.clientX - startXRef.current;
      const newWidth = Math.max(24, Math.min(60, resizeRef.current + delta));
      setColumnWidth(newWidth);
    };
    
    const handleMouseUp = () => {
      setIsResizing(false);
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };
    
    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
  }, [columnWidth]);

  const isKeyframe = useCallback((layerId: string, frame: number): boolean => {
    return keyframes.get(layerId)?.has(frame) ?? false;
  }, [keyframes]);

  const renderHeader = useMemo(() => {
    const frames: JSX.Element[] = [];
    const { startFrame, endFrame } = visibleRange;
    
    for (let f = startFrame; f <= endFrame; f++) {
      const isCurrentFrame = f === currentFrame;
      const isSelected = selectedCell?.frame === f;
      
      frames.push(
        <div
          key={`header-${f}`}
          data-testid={`xsheet-header-${f}`}
          style={{
            width: columnWidth,
            minWidth: columnWidth,
            height: HEADER_HEIGHT,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            background: isCurrentFrame ? COLORS.currentFrameGlow : COLORS.headerBg,
            color: isCurrentFrame ? COLORS.currentFrameBorder : COLORS.headerText,
            fontWeight: isCurrentFrame ? 600 : 400,
            fontSize: 11,
            borderRight: `1px solid ${COLORS.cellBorder}`,
            borderBottom: `2px solid ${isCurrentFrame ? COLORS.currentFrameBorder : COLORS.cellBorder}`,
            boxSizing: "border-box",
            position: "relative",
            cursor: "pointer",
            transition: "all 0.1s ease",
          }}
          onClick={(e) => {
            e.stopPropagation();
            onFrameSelect(f);
          }}
          onContextMenu={(e) => handleContextMenu(e, f)}
        >
          {f}
          {isSelected && (
            <div style={{
              position: "absolute",
              bottom: 0,
              left: 0,
              right: 0,
              height: 2,
              background: COLORS.selectedCell,
            }} />
          )}
        </div>
      );
    }
    
    return frames;
  }, [visibleRange, currentFrame, columnWidth, selectedCell, onFrameSelect, handleContextMenu]);

  const renderLayerColumn = useMemo(() => {
    const { startLayer, endLayer } = visibleRange;
    const layerItems: JSX.Element[] = [];
    
    for (let i = startLayer; i <= endLayer && i < layers.length; i++) {
      const layer = layers[i];
      const isSelected = selectedCell?.layerId === layer.id;
      const isEven = i % 2 === 0;
      
      layerItems.push(
        <div
          key={`layer-${layer.id}`}
          data-testid={`xsheet-layer-${layer.id}`}
          style={{
            width: LAYER_COLUMN_WIDTH,
            height: CELL_HEIGHT,
            display: "flex",
            alignItems: "center",
            padding: "0 8px",
            background: isSelected ? COLORS.currentFrameGlow : (isEven ? COLORS.rowEven : COLORS.rowOdd),
            color: isSelected ? COLORS.layerNameActive : COLORS.cellText,
            fontSize: 11,
            fontWeight: isSelected ? 500 : 400,
            borderRight: `1px solid ${COLORS.cellBorder}`,
            borderBottom: `1px solid ${COLORS.cellBorder}`,
            boxSizing: "border-box",
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
            transition: "background 0.1s ease",
          }}
          onClick={(e) => {
            e.stopPropagation();
            setSelectedCell({ layerId: layer.id, frame: currentFrame });
          }}
        >
          {!layer.visible && <span style={{ opacity: 0.5, marginRight: 4 }}>👁</span>}
          {layer.locked && <span style={{ opacity: 0.5, marginRight: 4 }}>🔒</span>}
          <span style={{ opacity: layer.opacity < 1 ? 0.6 : 1 }}>{layer.name}</span>
        </div>
      );
    }
    
    return layerItems;
  }, [visibleRange, layers, selectedCell, currentFrame]);

  const renderGrid = useMemo(() => {
    const { startFrame, endFrame, startLayer, endLayer } = visibleRange;
    const rows: JSX.Element[] = [];
    
    for (let i = startLayer; i <= endLayer && i < layers.length; i++) {
      const layer = layers[i];
      const isEvenRow = i % 2 === 0;
      const cells: JSX.Element[] = [];
      
      for (let f = startFrame; f <= endFrame; f++) {
        const isKey = isKeyframe(layer.id, f);
        const isCurrent = f === currentFrame;
        const isHovered = hoveredCell?.layerId === layer.id && hoveredCell?.frame === f;
        const isSelected = selectedCell?.layerId === layer.id && selectedCell?.frame === f;
        
        cells.push(
          <div
            key={`cell-${layer.id}-${f}`}
            data-testid={`xsheet-cell-${layer.id}-${f}`}
            style={{
              width: columnWidth,
              minWidth: columnWidth,
              height: CELL_HEIGHT,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              background: isCurrent
                ? COLORS.currentFrameGlow
                : isHovered
                  ? COLORS.hoverCell
                  : (isEvenRow ? COLORS.rowEven : COLORS.rowOdd),
              borderRight: `1px solid ${COLORS.cellBorder}`,
              borderBottom: `1px solid ${COLORS.cellBorder}`,
              borderLeft: isCurrent ? `2px solid ${COLORS.currentFrameBorder}` : "none",
              boxSizing: "border-box",
              position: "relative",
              cursor: "pointer",
              transition: "background 0.08s ease",
            }}
            onMouseEnter={() => setHoveredCell({ layerId: layer.id, frame: f })}
            onMouseLeave={() => setHoveredCell(null)}
            onClick={(e) => handleCellClick(layer.id, f, e)}
            onDoubleClick={(e) => handleCellDoubleClick(layer.id, f, e)}
            onContextMenu={(e) => handleContextMenu(e, f)}
          >
            {isKey && (
              <div
                data-testid={`xsheet-keyframe-${layer.id}-${f}`}
                style={{
                  width: Math.max(8, columnWidth - 16),
                  height: Math.max(8, CELL_HEIGHT - 8),
                  borderRadius: 3,
                  background: COLORS.keyframe,
                  boxShadow: `0 0 6px ${COLORS.keyframeGlow}`,
                  transition: "transform 0.1s ease",
                  transform: isSelected ? "scale(1.1)" : "scale(1)",
                }}
              />
            )}
            {isSelected && !isKey && (
              <div style={{
                width: columnWidth - 8,
                height: CELL_HEIGHT - 6,
                border: `1px solid ${COLORS.selectedCell}`,
                borderRadius: 2,
                boxSizing: "border-box",
              }} />
            )}
          </div>
        );
      }
      
      rows.push(
        <div
          key={`row-${layer.id}`}
          style={{
            display: "flex",
            height: CELL_HEIGHT,
          }}
        >
          {cells}
        </div>
      );
    }
    
    return rows;
  }, [
    visibleRange, 
    layers, 
    columnWidth, 
    currentFrame, 
    hoveredCell, 
    selectedCell, 
    isKeyframe,
    handleCellClick,
    handleCellDoubleClick,
    handleContextMenu,
  ]);

  const totalWidth = totalFrames * columnWidth;
  const totalHeight = layers.length * CELL_HEIGHT;

  useEffect(() => {
    const handleClickOutside = () => {
      if (contextMenu.isOpen) {
        closeContextMenu();
      }
    };
    
    if (contextMenu.isOpen) {
      document.addEventListener("click", handleClickOutside);
      document.addEventListener("contextmenu", handleClickOutside);
    }
    
    return () => {
      document.removeEventListener("click", handleClickOutside);
      document.removeEventListener("contextmenu", handleClickOutside);
    };
  }, [contextMenu.isOpen, closeContextMenu]);

  return (
    <div
      ref={containerRef}
      data-testid="xsheet-panel"
      style={{
        display: "flex",
        flexDirection: "column",
        width: "100%",
        height: "100%",
        background: COLORS.background,
        overflow: "hidden",
        userSelect: "none",
        fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
      }}
    >
      <div
        style={{
          display: "flex",
          height: HEADER_HEIGHT,
          background: COLORS.headerBg,
          borderBottom: `1px solid ${COLORS.cellBorder}`,
        }}
      >
        <div
          style={{
            width: LAYER_COLUMN_WIDTH,
            height: HEADER_HEIGHT,
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "0 8px",
            background: COLORS.headerBg,
            color: COLORS.headerText,
            fontSize: 10,
            fontWeight: 600,
            textTransform: "uppercase",
            letterSpacing: 1,
            borderRight: `1px solid ${COLORS.cellBorder}`,
            boxSizing: "border-box",
          }}
        >
          <span>图层</span>
          <div
            data-testid="xsheet-resize-handle"
            style={{
              width: 8,
              height: "100%",
              cursor: "col-resize",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
            onMouseDown={handleResizeStart}
          >
            <div style={{
              width: 2,
              height: 16,
              background: isResizing ? COLORS.selectedCell : COLORS.cellBorder,
              borderRadius: 1,
            }} />
          </div>
        </div>
        
        <div
          ref={headerRef}
          style={{
            flex: 1,
            overflow: "hidden",
            display: "flex",
          }}
        >
          <div
            style={{
              display: "flex",
              width: totalWidth,
              position: "relative",
            }}
          >
            <div style={{ width: (visibleRange.startFrame - 1) * columnWidth }} />
            {renderHeader}
          </div>
        </div>
      </div>

      <div
        style={{
          display: "flex",
          flex: 1,
          overflow: "hidden",
        }}
      >
        <div
          ref={layerColumnRef}
          style={{
            width: LAYER_COLUMN_WIDTH,
            overflow: "hidden",
            background: COLORS.background,
            borderRight: `1px solid ${COLORS.cellBorder}`,
          }}
        >
          <div
            style={{
              height: totalHeight,
              position: "relative",
            }}
          >
            <div style={{ height: visibleRange.startLayer * CELL_HEIGHT }} />
            {renderLayerColumn}
          </div>
        </div>

        <div
          ref={bodyRef}
          onScroll={handleScroll}
          style={{
            flex: 1,
            overflow: "auto",
            background: COLORS.background,
          }}
        >
          <div
            style={{
              width: totalWidth,
              height: totalHeight,
              position: "relative",
            }}
          >
            <div style={{ 
              width: (visibleRange.startFrame - 1) * columnWidth, 
              height: visibleRange.startLayer * CELL_HEIGHT 
            }} />
            <div style={{ 
              position: "absolute", 
              left: (visibleRange.startFrame - 1) * columnWidth,
              top: visibleRange.startLayer * CELL_HEIGHT,
            }}>
              {renderGrid}
            </div>
          </div>
        </div>
      </div>

      {contextMenu.isOpen && (
        <div
          data-testid="xsheet-context-menu"
          style={{
            position: "fixed",
            left: contextMenu.x,
            top: contextMenu.y,
            zIndex: 1000,
            background: COLORS.contextMenu,
            borderRadius: 6,
            boxShadow: "0 4px 12px rgba(0, 0, 0, 0.4)",
            border: `1px solid ${COLORS.cellBorder}`,
            padding: "4px 0",
            minWidth: 120,
          }}
          onClick={(e) => e.stopPropagation()}
        >
          <div
            data-testid="context-insert-frame"
            style={{
              padding: "6px 12px",
              cursor: "pointer",
              fontSize: 12,
              color: COLORS.cellText,
              display: "flex",
              alignItems: "center",
              gap: 8,
            }}
            onClick={() => {
              onFrameInsert(contextMenu.frame);
              closeContextMenu();
            }}
            onMouseEnter={(e) => e.currentTarget.style.background = COLORS.hoverCell}
            onMouseLeave={(e) => e.currentTarget.style.background = "transparent"}
          >
            ➕ 插入帧
          </div>
          <div
            data-testid="context-delete-frame"
            style={{
              padding: "6px 12px",
              cursor: "pointer",
              fontSize: 12,
              color: COLORS.cellText,
              display: "flex",
              alignItems: "center",
              gap: 8,
            }}
            onClick={() => {
              onFrameDelete(contextMenu.frame);
              closeContextMenu();
            }}
            onMouseEnter={(e) => e.currentTarget.style.background = COLORS.hoverCell}
            onMouseLeave={(e) => e.currentTarget.style.background = "transparent"}
          >
            🗑️ 删除帧
          </div>
          {onFrameCopy && (
            <div
              data-testid="context-copy-frame"
              style={{
                padding: "6px 12px",
                cursor: "pointer",
                fontSize: 12,
                color: COLORS.cellText,
                display: "flex",
                alignItems: "center",
                gap: 8,
              }}
              onClick={() => {
                onFrameCopy(contextMenu.frame);
                closeContextMenu();
              }}
              onMouseEnter={(e) => e.currentTarget.style.background = COLORS.hoverCell}
              onMouseLeave={(e) => e.currentTarget.style.background = "transparent"}
            >
              📋 复制帧
            </div>
          )}
        </div>
      )}
    </div>
  );
}
