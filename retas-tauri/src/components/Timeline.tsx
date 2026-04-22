import { useState, useEffect, useRef, lazy, Suspense } from "react";
import { Button, ButtonGroup } from "@blueprintjs/core";
import {
  ClientSideRowModelModule,
  ModuleRegistry,
  type ColDef,
  type RowClassParams
} from "ag-grid-community";
import {
  Play, Pause, SkipBack, SkipForward, Plus, Trash2
} from "lucide-react";
import { getFrameInfo, setCurrentFrame, addFrame, deleteFrame, FrameInfo } from "../api";

import "ag-grid-community/styles/ag-grid.css";
import "ag-grid-community/styles/ag-theme-alpine.css";

ModuleRegistry.registerModules([ClientSideRowModelModule]);

const AgGridReact = lazy(() => import("ag-grid-react").then(m => ({ default: m.AgGridReact })));

interface TimelineProps {
  isPlaying: boolean;
  onPlayToggle: () => void;
}

interface TimelineRow {
  layerName: string;
  layerId: string;
  [key: string]: any;
}

export default function Timeline({ isPlaying, onPlayToggle }: TimelineProps) {
  const [frameInfo, setFrameInfo] = useState<FrameInfo>({ current: 1, total: 100, fps: 24 });
  const [rowData, setRowData] = useState<TimelineRow[]>([]);
  const gridRef = useRef<any>(null);

  useEffect(() => {
    loadFrameInfo();
    generateTimelineData();
  }, []);

  const loadFrameInfo = async () => {
    try {
      const info = await getFrameInfo();
      setFrameInfo(info);
    } catch (e) {
      console.error("Failed to load frame info:", e);
    }
  };

  const generateTimelineData = () => {
    const rows: TimelineRow[] = [
      { layerName: "背景", layerId: "bg" },
      { layerName: "图层 1", layerId: "layer1" },
      { layerName: "图层 2", layerId: "layer2" },
    ];
    
    // Add frame cells
    for (let i = 1; i <= 24; i++) {
      rows.forEach(row => {
        row[`frame_${i}`] = i % 3 === 0 ? "key" : "";
      });
    }
    
    setRowData(rows);
  };

  const handleFrameChange = async (frame: number) => {
    try {
      await setCurrentFrame(frame);
      setFrameInfo(prev => ({ ...prev, current: frame }));
    } catch (e) {
      console.error(e);
    }
  };

  const handleAddFrame = async () => {
    try {
      await addFrame();
      await loadFrameInfo();
    } catch (e) {
      console.error(e);
    }
  };

  const handleDeleteFrame = async () => {
    try {
      await deleteFrame();
      await loadFrameInfo();
    } catch (e) {
      console.error(e);
    }
  };

  const columnDefs: ColDef[] = [
    { 
      field: "layerName", 
      headerName: "图层", 
      width: 120, 
      pinned: "left",
      cellStyle: { background: "#2d2d30", color: "#e0e0e0" }
    },
    ...Array.from({ length: 24 }, (_, i) => ({
      field: `frame_${i + 1}`,
      headerName: `${i + 1}`,
      width: 28,
      cellStyle: (params: any) => ({
        background: params.value === "key" ? "#094771" : "#1e1e1e",
        color: "#e0e0e0",
        textAlign: "center",
        border: "1px solid #333",
      }),
    })),
  ];

  const getRowClass = (params: RowClassParams) => {
    const index = params.node.rowIndex ?? 0;
    return index % 2 === 0 ? "timeline-row-even" : "timeline-row-odd";
  };

  return (
    <div className="timeline-container">
      <ButtonGroup>
        <Button minimal icon={<SkipBack size={14} />} onClick={() => handleFrameChange(1)}>起始</Button>
        <Button minimal icon={<SkipBack size={14} />} onClick={() => handleFrameChange(Math.max(1, frameInfo.current - 1))}>上一帧</Button>
        <Button 
          minimal 
          icon={isPlaying ? <Pause size={14} /> : <Play size={14} />}
          onClick={onPlayToggle}
        >
          {isPlaying ? "暂停" : "播放"}
        </Button>
        <Button minimal icon={<SkipForward size={14} />} onClick={() => handleFrameChange(Math.min(frameInfo.total, frameInfo.current + 1))}>下一帧</Button>
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
            />
          </Suspense>
        </div>
      </div>
      
      <div style={{ display: "flex", alignItems: "center", gap: 12, whiteSpace: "nowrap" }}>
        <span style={{ fontSize: 12, color: "#888" }}>
          帧: {frameInfo.current} / {frameInfo.total} | {frameInfo.fps} 帧/秒
        </span>
        <ButtonGroup>
          <Button minimal icon={<Plus size={14} />} onClick={handleAddFrame}>增加帧</Button>
          <Button minimal icon={<Trash2 size={14} />} onClick={handleDeleteFrame}>删除帧</Button>
        </ButtonGroup>
      </div>
    </div>
  );
}
