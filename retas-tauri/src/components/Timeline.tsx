import { useState, useEffect, useRef, lazy, Suspense, useCallback } from "react";
import { Button, ButtonGroup } from "@blueprintjs/core";
import {
  ClientSideRowModelModule,
  ModuleRegistry,
  type ColDef,
  type RowClassParams,
  type CellClickedEvent,
} from "ag-grid-community";
import {
  Play, Pause, SkipBack, SkipForward, Plus, Trash2, Copy, ClipboardPaste
} from "lucide-react";
import {
  getFrameInfo, setCurrentFrame, addFrame, deleteFrame,
  getXSheetData, getLayers, toggleKeyframe, copyFrame,
  FrameInfo, XSheetCell, LayerInfo,
} from "../api";

import "ag-grid-community/styles/ag-grid.css";
import "ag-grid-community/styles/ag-theme-alpine.css";

ModuleRegistry.registerModules([ClientSideRowModelModule]);

const AgGridReact = lazy(() => import("ag-grid-react").then(m => ({ default: m.AgGridReact })));

interface TimelineProps {
  isPlaying: boolean;
  onPlayToggle: () => void;
  currentFrame: number;
  totalFrames: number;
  onFrameChange: (frame: number) => void;
}

interface TimelineRow {
  layerName: string;
  layerId: string;
  [key: string]: string;
}

export default function Timeline({ isPlaying, onPlayToggle, currentFrame, totalFrames, onFrameChange }: TimelineProps) {
  const [frameInfo, setFrameInfo] = useState<FrameInfo>({ current: currentFrame, total: totalFrames, fps: 24 });
  const [rowData, setRowData] = useState<TimelineRow[]>([]);
  const [displayFrames, setDisplayFrames] = useState(24);
  const [copiedFrame, setCopiedFrame] = useState<{ layerId: string; frame: number } | null>(null);
  const gridRef = useRef<any>(null);

  const loadData = useCallback(async () => {
    try {
      const [info, layers, xsheet] = await Promise.all([
        getFrameInfo(),
        getLayers(),
        getXSheetData(),
      ]);

      setFrameInfo(info);
      const frameCount = Math.max(info.total, 24);
      setDisplayFrames(frameCount);

      // Build a lookup: layerId -> Set<frame>
      const keyframeMap = new Map<string, Set<number>>();
      for (const cell of xsheet) {
        if (cell.hasKeyframe) {
          if (!keyframeMap.has(cell.layerId)) {
            keyframeMap.set(cell.layerId, new Set());
          }
          keyframeMap.get(cell.layerId)!.add(cell.frame);
        }
      }

      // Build rows from real layer data
      const rows: TimelineRow[] = layers.map((layer: LayerInfo) => {
        const row: TimelineRow = {
          layerName: layer.name,
          layerId: layer.id,
        };
        const layerKeys = keyframeMap.get(layer.id);
        for (let f = 1; f <= frameCount; f++) {
          row[`frame_${f}`] = layerKeys?.has(f) ? "key" : "";
        }
        return row;
      });

      setRowData(rows);
    } catch (e) {
      console.error("Failed to load timeline data:", e);
    }
  }, []);

  useEffect(() => {
    loadData();
  }, [loadData]);

  // Auto-refresh on state changes
  useEffect(() => {
    const handler = () => loadData();
    window.addEventListener("retas:state-changed", handler);
    return () => window.removeEventListener("retas:state-changed", handler);
  }, [loadData]);

  useEffect(() => {
    setFrameInfo(prev => ({
      ...prev,
      current: currentFrame,
      total: totalFrames,
    }));
  }, [currentFrame, totalFrames]);

  const handleFrameChange = async (frame: number) => {
    try {
      await setCurrentFrame(frame);
      setFrameInfo(prev => ({ ...prev, current: frame }));
      onFrameChange(frame);
      window.dispatchEvent(new CustomEvent("retas:state-changed"));
    } catch (e) {
      console.error(e);
    }
  };

  const handleAddFrame = async () => {
    try {
      await addFrame();
      await loadData();
    } catch (e) {
      console.error(e);
    }
  };

  const handleDeleteFrame = async () => {
    try {
      await deleteFrame();
      await loadData();
    } catch (e) {
      console.error(e);
    }
  };

  const handleCellDoubleClick = async (event: CellClickedEvent) => {
    const field = event.colDef.field;
    if (!field || !field.startsWith("frame_")) return;
    const frame = parseInt(field.replace("frame_", ""), 10);
    const layerId = event.data?.layerId;
    if (!layerId || isNaN(frame)) return;

    try {
      await toggleKeyframe(layerId, frame);
      await loadData();
    } catch (e) {
      console.error("Toggle keyframe failed:", e);
    }
  };

  const handleCopyFrame = () => {
    // Copy current frame for the first layer (or active layer context)
    if (rowData.length > 0) {
      setCopiedFrame({ layerId: rowData[0].layerId, frame: frameInfo.current });
    }
  };

  const handlePasteFrame = async () => {
    if (!copiedFrame) return;
    try {
      await copyFrame(copiedFrame.layerId, copiedFrame.frame, frameInfo.current);
      await loadData();
      window.dispatchEvent(new CustomEvent("retas:state-changed"));
    } catch (e) {
      console.error("Paste frame failed:", e);
    }
  };

  const columnDefs: ColDef[] = [
    {
      field: "layerName",
      headerName: "图层",
      width: 120,
      pinned: "left",
      cellStyle: { background: "#2d2d30", color: "#e0e0e0" },
    },
    ...Array.from({ length: displayFrames }, (_, i) => {
      const frameNum = i + 1;
      const isCurrent = frameNum === currentFrame;
      return {
        field: `frame_${frameNum}`,
        headerName: `${frameNum}`,
        width: 28,
        cellStyle: (params: any) => ({
          background: isCurrent
            ? "rgba(31, 111, 235, 0.25)"
            : params.value === "key"
              ? "#094771"
              : "#1e1e1e",
          color: "#e0e0e0",
          textAlign: "center" as const,
          border: "1px solid #333",
        }),
      };
    }),
  ];

  const getRowClass = (params: RowClassParams) => {
    const index = params.node.rowIndex ?? 0;
    return index % 2 === 0 ? "timeline-row-even" : "timeline-row-odd";
  };

  return (
    <div className="timeline-container">
      <ButtonGroup>
        <Button minimal data-testid="frame-first" icon={<SkipBack size={14} />} onClick={() => handleFrameChange(1)}>起始</Button>
        <Button minimal data-testid="frame-prev" icon={<SkipBack size={14} />} onClick={() => handleFrameChange(Math.max(1, frameInfo.current - 1))}>上一帧</Button>
        <Button
          minimal
          data-testid="playback-toggle"
          icon={isPlaying ? <Pause size={14} /> : <Play size={14} />}
          onClick={onPlayToggle}
        >
          {isPlaying ? "暂停" : "播放"}
        </Button>
        <Button minimal data-testid="frame-next" icon={<SkipForward size={14} />} onClick={() => handleFrameChange(Math.min(frameInfo.total, frameInfo.current + 1))}>下一帧</Button>
      </ButtonGroup>

      <div style={{ flex: 1, overflow: "hidden" }}>
        <div className="ag-theme-alpine-dark" style={{ height: 80, width: "100%" }}>
          <Suspense fallback={<div style={{ color: "#888", fontSize: 12 }}>加载中...</div>}>
            <AgGridReact
              ref={gridRef}
              rowData={rowData}
              columnDefs={columnDefs}
              modules={[ClientSideRowModelModule]}
              getRowClass={getRowClass}
              headerHeight={24}
              rowHeight={28}
              suppressCellFocus={true}
              domLayout="autoHeight"
              onCellDoubleClicked={handleCellDoubleClick}
            />
          </Suspense>
        </div>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 12, whiteSpace: "nowrap" }}>
        <span data-testid="frame-counter" style={{ fontSize: 12, color: "#888" }}>
          帧: {frameInfo.current} / {frameInfo.total} | {frameInfo.fps} 帧/秒
        </span>
        <ButtonGroup>
          <Button minimal data-testid="frame-add" icon={<Plus size={14} />} onClick={handleAddFrame}>增加帧</Button>
          <Button minimal data-testid="frame-delete" icon={<Trash2 size={14} />} onClick={handleDeleteFrame}>删除帧</Button>
          <Button minimal data-testid="frame-copy" icon={<Copy size={14} />} onClick={handleCopyFrame} title="复制当前帧">复制</Button>
          <Button minimal data-testid="frame-paste" icon={<ClipboardPaste size={14} />} onClick={handlePasteFrame} disabled={!copiedFrame} title="粘贴帧到当前位置">粘贴</Button>
        </ButtonGroup>
      </div>
    </div>
  );
}
