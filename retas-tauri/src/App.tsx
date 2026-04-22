import { useState, useRef, useCallback, useEffect } from "react";
import { ButtonGroup, Button, Popover, Menu, MenuDivider, Checkbox } from "@blueprintjs/core";
import {
  Brush, Eraser, PenTool, PaintBucket,
  Move, ZoomIn, ZoomOut, Hand, ChevronDown,
  Undo2, Redo2, FolderOpen, Save, FilePlus
} from "lucide-react";
import { DockviewReact, DockviewReadyEvent, IDockviewPanelProps } from "dockview";
import "dockview/dist/styles/dockview.css";
import ToolBox from "./components/ToolBox";
import WorkspaceSwitcher from "./components/WorkspaceSwitcher";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useCanvasKit } from "./hooks/useCanvasKit";
import { useWorkspace } from "./hooks/useWorkspace";
import { openDocument, saveDocument, undo, redo, canUndo, canRedo } from "./api";
import { showOpenDialog, showSaveDialog } from "./utils/fileDialog";
import MemoryMonitor from "./components/MemoryMonitor";
import ShortcutHelp from "./components/ShortcutHelp";
import { ExportButton } from "./components/ExportDialog";

import LayerPanel from "./components/LayerPanel";
import ColorPanel from "./components/ColorPanel";
import AiPanel from "./components/AiPanel";
import Timeline from "./components/Timeline";
import PlaybackController from "./components/PlaybackController";
import SkiaCanvas from "./components/SkiaCanvas";
import PenToolCanvas from "./components/PenTool";
import SelectionToolCanvas from "./components/SelectionTool";

type Tool = "brush" | "eraser" | "pen" | "fill" | "select" | "move" | "zoom" | "hand";

interface ToolDef {
  id: Tool;
  icon: React.ReactNode;
  label: string;
}

const allTools: ToolDef[] = [
  { id: "select", icon: <Move size={16} />, label: "选择" },
  { id: "brush", icon: <Brush size={16} />, label: "画笔" },
  { id: "pen", icon: <PenTool size={16} />, label: "钢笔" },
  { id: "eraser", icon: <Eraser size={16} />, label: "橡皮" },
  { id: "fill", icon: <PaintBucket size={16} />, label: "填充" },
  { id: "hand", icon: <Hand size={16} />, label: "抓手" },
];

const workspaceLabels: Record<string, string> = {
  drawing: "绘画",
  animation: "动画",
  coloring: "上色",
  compositing: "合成",
};

function CanvasPanel(props: IDockviewPanelProps<{ tool?: string; zoom?: number; color?: string; brushSize?: number }>) {
  const tool = props.params.tool || "brush";
  const zoom = props.params.zoom || 100;
  const color = props.params.color || "#000000";
  const brushSize = props.params.brushSize || 2;

  switch (tool) {
    case "pen":
      return <PenToolCanvas zoom={zoom} />;
    case "select":
      return <SelectionToolCanvas zoom={zoom} />;
    default:
      return <SkiaCanvas tool={tool} zoom={zoom} color={color} brushSize={brushSize} />;
  }
}

function TimelinePanel(props: IDockviewPanelProps<{ isPlaying?: boolean; onPlayToggle?: () => void; currentFrame?: number; totalFrames?: number; fps?: number; onFrameChange?: (frame: number) => void }>) {
  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", background: "#161b22" }}>
      <Timeline 
        isPlaying={props.params.isPlaying || false} 
        onPlayToggle={props.params.onPlayToggle || (() => {})}
        currentFrame={props.params.currentFrame || 1}
        totalFrames={props.params.totalFrames || 100}
        onFrameChange={props.params.onFrameChange || (() => {})}
      />
      <PlaybackController 
        totalFrames={props.params.totalFrames || 100} 
        fps={props.params.fps || 24} 
        currentFrame={props.params.currentFrame || 1}
        onFrameChange={props.params.onFrameChange || (() => {})} 
      />
    </div>
  );
}

function AnimationPropsPanel() {
  return (
    <div style={{ padding: 12 }}>
      <div style={{ fontSize: 11, fontWeight: 600, color: "#8b949e", marginBottom: 10, textTransform: "uppercase", letterSpacing: 0.8 }}>动画属性</div>
      <div style={{ fontSize: 12, color: "#8b949e" }}>
        <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8 }}>
          <span>帧率</span><span style={{ color: "#e6edf3" }}>24 fps</span>
        </div>
        <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8 }}>
          <span>总帧数</span><span style={{ color: "#e6edf3" }}>100</span>
        </div>
        <div style={{ display: "flex", justifyContent: "space-between" }}>
          <span>当前帧</span><span style={{ color: "#e6edf3" }}>1</span>
        </div>
      </div>
    </div>
  );
}

function BlendModesPanel() {
  return (
    <div style={{ padding: 12 }}>
      <div style={{ fontSize: 11, fontWeight: 600, color: "#8b949e", marginBottom: 10, textTransform: "uppercase", letterSpacing: 0.8 }}>混合模式</div>
      <div style={{ fontSize: 12, color: "#8b949e" }}>正常 / 正片叠底 / 滤色 / 叠加</div>
    </div>
  );
}

const components = {
  canvas: CanvasPanel,
  layerPanel: LayerPanel,
  colorPanel: (props: IDockviewPanelProps<{ color?: string; onColorChange?: (c: string) => void; brushSize?: number; onBrushSizeChange?: (s: number) => void }>) => (
    <ColorPanel
      color={props.params.color || "#000000"}
      onColorChange={props.params.onColorChange || (() => {})}
      brushSize={props.params.brushSize || 2}
      onBrushSizeChange={props.params.onBrushSizeChange || (() => {})}
    />
  ),
  aiPanel: AiPanel,
  timeline: TimelinePanel,
  animationProps: AnimationPropsPanel,
  blendModes: BlendModesPanel,
};

function App() {
  useKeyboardShortcuts();
  const { memoryInfo } = useCanvasKit();
  const { currentWorkspace } = useWorkspace();
  const apiRef = useRef<any>(null);

  const [currentTool, setCurrentTool] = useState<Tool>("brush");
  const [zoom, setZoom] = useState(100);
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentFrame, setCurrentFrame] = useState(1);
  const [totalFrames, _setTotalFrames] = useState(100);
  const [fps, _setFps] = useState(24);
  const [brushColor, setBrushColor] = useState("#000000");
  const [brushSize, setBrushSize] = useState(2);
  const [visibleToolIds, setVisibleToolIds] = useState<Set<Tool>>(new Set(allTools.map(t => t.id)));
  const [undoAvailable, setUndoAvailable] = useState(false);
  const [redoAvailable, setRedoAvailable] = useState(false);

  const visibleTools = allTools.filter(t => visibleToolIds.has(t.id));

  const refreshHistoryState = useCallback(async () => {
    try {
      const [u, r] = await Promise.all([canUndo(), canRedo()]);
      setUndoAvailable(u);
      setRedoAvailable(r);
    } catch {
      setUndoAvailable(false);
      setRedoAvailable(false);
    }
  }, []);

  useEffect(() => {
    const handler = () => refreshHistoryState();
    window.addEventListener("retas:state-changed", handler);
    refreshHistoryState();
    return () => window.removeEventListener("retas:state-changed", handler);
  }, [refreshHistoryState]);

  useEffect(() => {
    if (!isPlaying) return;
    const interval = setInterval(() => {
      setCurrentFrame(prev => {
        const next = prev >= totalFrames ? 1 : prev + 1;
        return next;
      });
    }, 1000 / fps);
    return () => clearInterval(interval);
  }, [isPlaying, fps, totalFrames]);

  const toggleToolVisibility = (toolId: Tool) => {
    setVisibleToolIds(prev => {
      const next = new Set(prev);
      if (next.has(toolId)) {
        next.delete(toolId);
        if (currentTool === toolId) {
          const remaining = allTools.filter(t => next.has(t.id));
          if (remaining.length > 0) setCurrentTool(remaining[0].id);
        }
      } else {
        next.add(toolId);
      }
      return next;
    });
  };

  const onReady = useCallback((event: DockviewReadyEvent) => {
    const api = event.api;
    apiRef.current = api;

    const canvasPanel = api.addPanel({
      id: "canvas",
      component: "canvas",
      title: "画布",
      params: { tool: currentTool, zoom, color: brushColor, brushSize },
    });

    api.addPanel({
      id: "layers",
      component: "layerPanel",
      title: "图层",
      position: { referencePanel: "canvas", direction: "left" },
    });
    api.getPanel("layers")?.api.setSize({ width: 200 });

    api.addPanel({
      id: "timeline",
      component: "timeline",
      title: "时间轴",
      position: { referencePanel: "canvas", direction: "below" },
      params: { 
        isPlaying, 
        onPlayToggle: () => setIsPlaying(p => !p),
        currentFrame,
        totalFrames,
        fps,
        onFrameChange: (frame: number) => setCurrentFrame(frame)
      },
    });
    api.getPanel("timeline")?.api.setSize({ height: 180 });

    canvasPanel.api.setActive();

    const workspace = currentWorkspace;
    if (workspace === "drawing") {
      api.addPanel({
        id: "color",
        component: "colorPanel",
        title: "颜色",
        position: { referencePanel: "canvas", direction: "right" },
        params: { color: brushColor, onColorChange: setBrushColor, brushSize, onBrushSizeChange: setBrushSize },
      });
      api.addPanel({
        id: "ai",
        component: "aiPanel",
        title: "AI",
        position: { referencePanel: "color", direction: "below" },
      });
    } else if (workspace === "animation") {
      api.addPanel({
        id: "animationProps",
        component: "animationProps",
        title: "动画属性",
        position: { referencePanel: "canvas", direction: "right" },
      });
      api.addPanel({
        id: "ai",
        component: "aiPanel",
        title: "AI",
        position: { referencePanel: "animationProps", direction: "below" },
      });
    } else if (workspace === "coloring") {
      api.addPanel({
        id: "color",
        component: "colorPanel",
        title: "颜色",
        position: { referencePanel: "canvas", direction: "right" },
        params: { color: brushColor, onColorChange: setBrushColor, brushSize, onBrushSizeChange: setBrushSize },
      });
      api.addPanel({
        id: "layers2",
        component: "layerPanel",
        title: "图层",
        position: { referencePanel: "color", direction: "below" },
      });
    } else if (workspace === "compositing") {
      api.addPanel({
        id: "blendModes",
        component: "blendModes",
        title: "混合模式",
        position: { referencePanel: "canvas", direction: "right" },
      });
      api.addPanel({
        id: "ai",
        component: "aiPanel",
        title: "AI",
        position: { referencePanel: "blendModes", direction: "below" },
      });
    }
  }, []);

  const updateCanvas = useCallback(() => {
    if (apiRef.current) {
      const panel = apiRef.current.getPanel("canvas");
      if (panel) {
        panel.api.updateParameters({ tool: currentTool, zoom, color: brushColor, brushSize });
      }
    }
  }, [currentTool, zoom, brushColor, brushSize]);

  useEffect(() => {
    updateCanvas();
  }, [updateCanvas]);

  useEffect(() => {
    if (!apiRef.current) return;
    const api = apiRef.current;

    const rightPanelIds = ["color", "ai", "animationProps", "blendModes"];
    rightPanelIds.forEach(id => {
      const panel = api.getPanel(id);
      if (panel) {
        try { panel.api.close(); } catch {}
      }
    });

    if (currentWorkspace === "drawing") {
      api.addPanel({
        id: "color",
        component: "colorPanel",
        title: "颜色",
        position: { referencePanel: "canvas", direction: "right" },
        params: { color: brushColor, onColorChange: setBrushColor, brushSize, onBrushSizeChange: setBrushSize },
      });
      api.addPanel({
        id: "ai",
        component: "aiPanel",
        title: "AI",
        position: { referencePanel: "color", direction: "below" },
      });
    } else if (currentWorkspace === "animation") {
      api.addPanel({
        id: "animationProps",
        component: "animationProps",
        title: "动画属性",
        position: { referencePanel: "canvas", direction: "right" },
      });
      api.addPanel({
        id: "ai",
        component: "aiPanel",
        title: "AI",
        position: { referencePanel: "animationProps", direction: "below" },
      });
    } else if (currentWorkspace === "coloring") {
      api.addPanel({
        id: "color",
        component: "colorPanel",
        title: "颜色",
        position: { referencePanel: "canvas", direction: "right" },
        params: { color: brushColor, onColorChange: setBrushColor, brushSize, onBrushSizeChange: setBrushSize },
      });
      api.addPanel({
        id: "ai",
        component: "aiPanel",
        title: "AI",
        position: { referencePanel: "color", direction: "below" },
      });
    } else if (currentWorkspace === "compositing") {
      api.addPanel({
        id: "blendModes",
        component: "blendModes",
        title: "混合模式",
        position: { referencePanel: "canvas", direction: "right" },
      });
      api.addPanel({
        id: "ai",
        component: "aiPanel",
        title: "AI",
        position: { referencePanel: "blendModes", direction: "below" },
      });
    }
  }, [currentWorkspace]);

  const handleUndo = async () => {
    try {
      const didUndo = await undo();
      if (didUndo) {
        window.dispatchEvent(new CustomEvent("retas:state-changed"));
      }
    } catch (e) {
      console.error("Undo failed:", e);
    }
  };

  const handleRedo = async () => {
    try {
      const didRedo = await redo();
      if (didRedo) {
        window.dispatchEvent(new CustomEvent("retas:state-changed"));
      }
    } catch (e) {
      console.error("Redo failed:", e);
    }
  };

  const handleOpen = async () => {
    try {
      const path = await showOpenDialog();
      if (path) {
        const result = await openDocument(path);
        console.log("Opened:", result);
        window.dispatchEvent(new CustomEvent("retas:state-changed"));
      }
    } catch (e) {
      console.error("Open failed:", e);
    }
  };

  const handleSave = async () => {
    try {
      const path = await showSaveDialog();
      if (path) {
        await saveDocument(path);
        console.log("Saved to:", path);
      }
    } catch (e) {
      console.error("Save failed:", e);
    }
  };

  const toolVisibilityMenu = (
    <Menu>
      <MenuDivider title="显示工具" />
      {allTools.map(tool => (
        <div key={tool.id} style={{ padding: "5px 12px", display: "flex", alignItems: "center", gap: 6 }}>
          <Checkbox checked={visibleToolIds.has(tool.id)} onChange={() => toggleToolVisibility(tool.id)} />
          <span style={{ display: "flex", alignItems: "center", gap: 6, marginLeft: 4 }}>
            {tool.icon}
            <span style={{ fontSize: 12 }}>{tool.label}</span>
          </span>
        </div>
      ))}
    </Menu>
  );

  return (
    <div className="app-container">
      <div className="toolbar" style={{ height: 40, borderBottom: "1px solid #2d3139", flexShrink: 0 }}>
        <ButtonGroup minimal>
          <Button small icon={<FilePlus size={14} />}>新建</Button>
          <Button small icon={<FolderOpen size={14} />} onClick={handleOpen}>打开</Button>
          <Button small icon={<Save size={14} />} onClick={handleSave}>保存</Button>
        </ButtonGroup>

        <div style={{ width: 1, height: 20, background: "#2d3139", margin: "0 6px" }} />

        <ButtonGroup minimal>
          <Button
            small
            icon={<Undo2 size={14} />}
            onClick={handleUndo}
            disabled={!undoAvailable}
            title="撤销 (Ctrl+Z)"
          />
          <Button
            small
            icon={<Redo2 size={14} />}
            onClick={handleRedo}
            disabled={!redoAvailable}
            title="重做 (Ctrl+Y)"
          />
        </ButtonGroup>

        <div style={{ width: 1, height: 20, background: "#2d3139", margin: "0 6px" }} />

        <ButtonGroup minimal>
          {visibleTools.map((t) => (
            <Button
              key={t.id}
              small
              icon={<span style={{ display: "flex" }}>{t.icon}</span>}
              active={currentTool === t.id}
              onClick={() => setCurrentTool(t.id)}
              title={t.label}
            />
          ))}
          <Popover content={toolVisibilityMenu} position="bottom">
            <Button small minimal icon={<span><ChevronDown size={12} /></span>} title="工具显示选项" />
          </Popover>
        </ButtonGroup>

        <div style={{ flex: 1 }} />

        <ButtonGroup minimal>
          <Button small icon={<ZoomOut size={12} />} onClick={() => setZoom(z => Math.max(10, z - 10))} />
          <span style={{ padding: "0 6px", fontSize: 11, display: "flex", alignItems: "center", color: "#8b949e" }}>
            {zoom}%
          </span>
          <Button small icon={<ZoomIn size={12} />} onClick={() => setZoom(z => Math.min(500, z + 10))} />
        </ButtonGroup>

        <span style={{ marginLeft: 12, fontSize: 11, color: "#8b949e" }}>
          {allTools.find(t => t.id === currentTool)?.label}
        </span>

        <div style={{ marginLeft: 6 }}><ShortcutHelp /></div>
        <div style={{ marginLeft: 4 }}><ExportButton /></div>
      </div>

      <WorkspaceSwitcher />

      <div style={{ flex: 1, overflow: "hidden", position: "relative" }}>
        <div style={{ display: "flex", height: "100%" }}>
          <ToolBox currentTool={currentTool} onToolChange={(t) => setCurrentTool(t as Tool)} />
          <div style={{ flex: 1, position: "relative" }}>
            <div style={{ height: "100%" }}>
              <DockviewReact
                components={components}
                onReady={onReady}
                className="dockview-theme-dark"
              />
            </div>
          </div>
        </div>
      </div>

      <div className="status-bar" style={{ borderTop: "1px solid #2d3139", flexShrink: 0 }}>
        <span>RETAS Studio v1.0</span>
        <div style={{ flex: 1 }} />
        <span>1920 x 1080 | 24fps | {workspaceLabels[currentWorkspace] || currentWorkspace}</span>
      </div>

      {import.meta.env.MODE === "development" && (
        <MemoryMonitor memoryInfo={memoryInfo} currentTool={allTools.find((t: ToolDef) => t.id === currentTool)?.label} />
      )}
    </div>
  );
}

export default App;
