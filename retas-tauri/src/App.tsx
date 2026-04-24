import { useState, useRef, useCallback, useEffect } from "react";
import { ButtonGroup, Button, Popover, Menu, MenuDivider, Checkbox } from "@blueprintjs/core";
import {
  Brush, Eraser, PenTool, PaintBucket,
  Move, ZoomIn, ZoomOut, Hand, ChevronDown,
  Undo2, Redo2, FolderOpen, Save, FilePlus,
  Grid3X3, Ruler, Pipette
} from "lucide-react";
import { DockviewReact, DockviewReadyEvent, IDockviewPanelProps } from "dockview";
import "dockview/dist/styles/dockview.css";
import ToolBox from "./components/ToolBox";
import WorkspaceSwitcher from "./components/WorkspaceSwitcher";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { useCanvasKit } from "./hooks/useCanvasKit";
import { useWorkspace } from "./hooks/useWorkspace";
import {
  openDocument, saveDocument, undo, redo, canUndo, canRedo,
  getLayers, getXSheetData, toggleKeyframe, insertFrames, deleteFrames,
  getFrameInfo, setLayerBlendMode, LayerInfo,
} from "./api";
import { showOpenDialog, showSaveDialog } from "./utils/fileDialog";
import MemoryMonitor from "./components/MemoryMonitor";
import ShortcutHelp from "./components/ShortcutHelp";
import { ExportButton } from "./components/ExportDialog";
import NewDocumentDialog from "./components/NewDocumentDialog";

import LayerPanel from "./components/LayerPanel";
import ColorPanel from "./components/ColorPanel";
import Timeline from "./components/Timeline";
import PlaybackController from "./components/PlaybackController";
import UnifiedCanvas from "./components/UnifiedCanvas";
import { OnionSkinPanel, OnionSkinSettings } from "./components/OnionSkinPanel";
import SelectionToolPanel, { SelectionData } from "./components/SelectionToolPanel";
import XSheetPanel from "./components/XSheetPanel";
import EffectsPanel from "./components/EffectsPanel";
import RenderQueuePanel from "./components/RenderQueuePanel";
import LightTablePanel from "./components/LightTablePanel";
import MotionCheckPanel from "./components/MotionCheckPanel";
import CutSystemPanel from "./components/CutSystemPanel";
import PerspectiveGuidePanel from "./components/PerspectiveGuidePanel";

type Tool = "brush" | "eraser" | "pen" | "fill" | "select" | "move" | "zoom" | "hand" | "eyedropper";

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
  { id: "eyedropper", icon: <Pipette size={16} />, label: "吸管" },
  { id: "hand", icon: <Hand size={16} />, label: "抓手" },
];

const workspaceLabels: Record<string, string> = {
  drawing: "绘画",
  animation: "动画",
  coloring: "上色",
  compositing: "合成",
};

function CanvasPanel(props: IDockviewPanelProps<{ 
  tool?: string; 
  zoom?: number; 
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
}>) {
  const tool = props.params.tool || "brush";
  const zoom = props.params.zoom || 100;
  const color = props.params.color || "#000000";
  const brushSize = props.params.brushSize || 2;
  const onionSkin = props.params.onionSkin;
  const currentFrame = props.params.currentFrame || 1;
  const totalFrames = props.params.totalFrames || 100;
  const showGrid = props.params.showGrid || false;
  const showGuides = props.params.showGuides || false;
  const selection = props.params.selection;
  const selectionTool = props.params.selectionTool || "rect";
  const selectionMode = props.params.selectionMode || "replace";
  const onSelectionChange = props.params.onSelectionChange;
  const onColorPick = props.params.onColorPick;
  const activeLayerId = props.params.activeLayerId;

  return <UnifiedCanvas 
    tool={tool} 
    zoom={zoom} 
    color={color} 
    brushSize={brushSize} 
    onionSkin={onionSkin}
    currentFrame={currentFrame}
    totalFrames={totalFrames}
    showGrid={showGrid}
    showGuides={showGuides}
    selection={selection}
    selectionTool={selectionTool}
    selectionMode={selectionMode}
    onSelectionChange={onSelectionChange}
    onColorPick={onColorPick}
    activeLayerId={activeLayerId}
  />;
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

function AnimationPropsPanel(props: IDockviewPanelProps<{
  currentFrame?: number;
  totalFrames?: number;
  fps?: number;
}>) {
  const [frameInfo, setFrameInfo] = useState<{ current: number; total: number; fps: number }>({
    current: props.params.currentFrame || 1,
    total: props.params.totalFrames || 100,
    fps: props.params.fps || 24,
  });

  useEffect(() => {
    const handler = async () => {
      try {
        const info = await getFrameInfo();
        setFrameInfo(info);
      } catch {}
    };
    handler();
    window.addEventListener("retas:state-changed", handler);
    return () => window.removeEventListener("retas:state-changed", handler);
  }, []);

  useEffect(() => {
    setFrameInfo(prev => ({
      ...prev,
      current: props.params.currentFrame || prev.current,
      total: props.params.totalFrames || prev.total,
      fps: props.params.fps || prev.fps,
    }));
  }, [props.params.currentFrame, props.params.totalFrames, props.params.fps]);

  return (
    <div style={{ padding: 12 }}>
      <div style={{ fontSize: 11, fontWeight: 600, color: "#8b949e", marginBottom: 10, textTransform: "uppercase", letterSpacing: 0.8 }}>动画属性</div>
      <div style={{ fontSize: 12, color: "#8b949e" }}>
        <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8 }}>
          <span>帧率</span><span style={{ color: "#e6edf3" }}>{frameInfo.fps} fps</span>
        </div>
        <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8 }}>
          <span>总帧数</span><span style={{ color: "#e6edf3" }}>{frameInfo.total}</span>
        </div>
        <div style={{ display: "flex", justifyContent: "space-between" }}>
          <span>当前帧</span><span style={{ color: "#e6edf3" }}>{frameInfo.current}</span>
        </div>
      </div>
    </div>
  );
}
    </div>
  );
}

const BLEND_MODES = [
  { value: "normal", label: "正常" },
  { value: "multiply", label: "正片叠底" },
  { value: "screen", label: "滤色" },
  { value: "overlay", label: "叠加" },
  { value: "darken", label: "变暗" },
  { value: "lighten", label: "变亮" },
  { value: "color_dodge", label: "颜色减淡" },
  { value: "color_burn", label: "颜色加深" },
  { value: "hard_light", label: "强光" },
  { value: "soft_light", label: "柔光" },
  { value: "difference", label: "差值" },
  { value: "exclusion", label: "排除" },
  { value: "hue", label: "色相" },
  { value: "saturation", label: "饱和度" },
  { value: "color", label: "颜色" },
  { value: "luminosity", label: "明度" },
];

function BlendModesPanel() {
  const [layers, setLayers] = useState<LayerInfo[]>([]);
  const [activeLayerId, setActiveLayerId] = useState<string | null>(null);

  const loadLayers = useCallback(async () => {
    try {
      const data = await getLayers();
      setLayers(data);
      if (!activeLayerId && data.length > 0) setActiveLayerId(data[0].id);
    } catch {}
  }, [activeLayerId]);

  useEffect(() => { loadLayers(); }, [loadLayers]);
  useEffect(() => {
    const handler = () => loadLayers();
    window.addEventListener("retas:state-changed", handler);
    return () => window.removeEventListener("retas:state-changed", handler);
  }, [loadLayers]);

  const activeLayer = layers.find(l => l.id === activeLayerId);
  const currentMode = activeLayer?.blendMode || "normal";

  const handleChange = async (mode: string) => {
    if (!activeLayerId) return;
    try {
      await setLayerBlendMode(activeLayerId, mode);
      window.dispatchEvent(new CustomEvent("retas:state-changed"));
    } catch (e) {
      console.error("设置混合模式失败:", e);
    }
  };

  return (
    <div style={{ padding: 12 }}>
      <div style={{ fontSize: 11, fontWeight: 600, color: "#8b949e", marginBottom: 10, textTransform: "uppercase", letterSpacing: 0.8 }}>混合模式</div>
      {activeLayer && (
        <div style={{ fontSize: 12, color: "#8b949e", marginBottom: 8 }}>
          图层: <span style={{ color: "#e6edf3" }}>{activeLayer.name}</span>
        </div>
      )}
      <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
        {BLEND_MODES.map(mode => (
          <div
            key={mode.value}
            onClick={() => handleChange(mode.value)}
            style={{
              padding: "4px 8px",
              borderRadius: 3,
              fontSize: 12,
              cursor: "pointer",
              background: currentMode === mode.value ? "#094771" : "transparent",
              color: currentMode === mode.value ? "#58a6ff" : "#8b949e",
            }}
          >
            {mode.label}
          </div>
        ))}
      </div>
    </div>
  );
}

function XSheetWrapper(props: IDockviewPanelProps<{
  currentFrame?: number;
  totalFrames?: number;
  onFrameChange?: (frame: number) => void;
}>) {
  const [layers, setLayers] = useState<LayerInfo[]>([]);
  const [keyframes, setKeyframes] = useState<Map<string, Set<number>>>(new Map());

  const loadData = useCallback(async () => {
    try {
      const [layerData, xsheet] = await Promise.all([getLayers(), getXSheetData()]);
      setLayers(layerData);
      const kfMap = new Map<string, Set<number>>();
      for (const cell of xsheet) {
        if (cell.hasKeyframe) {
          if (!kfMap.has(cell.layerId)) kfMap.set(cell.layerId, new Set());
          kfMap.get(cell.layerId)!.add(cell.frame);
        }
      }
      setKeyframes(kfMap);
    } catch (e) {
      console.error("XSheet load failed:", e);
    }
  }, []);

  useEffect(() => { loadData(); }, [loadData]);
  useEffect(() => {
    const handler = () => loadData();
    window.addEventListener("retas:state-changed", handler);
    return () => window.removeEventListener("retas:state-changed", handler);
  }, [loadData]);

  const handleKeyframeToggle = async (layerId: string, frame: number) => {
    try {
      await toggleKeyframe(layerId, frame);
      await loadData();
      window.dispatchEvent(new CustomEvent("retas:state-changed"));
    } catch (e) { console.error(e); }
  };

  const handleFrameInsert = async (frame: number) => {
    try {
      await insertFrames(frame, 1);
      await loadData();
      window.dispatchEvent(new CustomEvent("retas:state-changed"));
    } catch (e) { console.error(e); }
  };

  const handleFrameDelete = async (frame: number) => {
    try {
      await deleteFrames(frame, 1);
      await loadData();
      window.dispatchEvent(new CustomEvent("retas:state-changed"));
    } catch (e) { console.error(e); }
  };

  return (
    <XSheetPanel
      layers={layers}
      currentFrame={props.params.currentFrame || 1}
      totalFrames={props.params.totalFrames || 100}
      keyframes={keyframes}
      onFrameSelect={props.params.onFrameChange || (() => {})}
      onKeyframeToggle={handleKeyframeToggle}
      onFrameInsert={handleFrameInsert}
      onFrameDelete={handleFrameDelete}
    />
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
  timeline: TimelinePanel,
  animationProps: AnimationPropsPanel,
  blendModes: BlendModesPanel,
  effectsPanel: EffectsPanel,
  renderQueue: RenderQueuePanel,
  lightTable: LightTablePanel,
  motionCheck: MotionCheckPanel,
  cutSystem: CutSystemPanel,
  perspectiveGuide: PerspectiveGuidePanel,
  xsheet: XSheetWrapper,
};

function App() {
  const handleToolChange = useCallback((tool: string) => {
    setCurrentTool(tool as Tool);
  }, []);

  const handleBrushSizeChange = useCallback((delta: number) => {
    setBrushSize(prev => Math.max(1, Math.min(100, prev + delta * 2)));
  }, []);

  const handleExportShortcut = useCallback(() => {
    window.dispatchEvent(new CustomEvent("retas:open-export"));
  }, []);

  useKeyboardShortcuts(handleToolChange, handleBrushSizeChange, handleExportShortcut);
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
  const [onionSkinSettings, setOnionSkinSettings] = useState<OnionSkinSettings>({
    enabled: false,
    framesBefore: 1,
    framesAfter: 1,
    opacityBefore: 0.3,
    opacityAfter: 0.3,
    colorBefore: "#ff0000",
    colorAfter: "#00ff00",
    blendMode: "tint",
  });
  const [selectionData, setSelectionData] = useState<SelectionData | null>(null);
  const [selectionTool, setSelectionTool] = useState<"rect" | "ellipse" | "lasso" | "magicWand">("rect");
  const [selectionMode, setSelectionMode] = useState<"replace" | "add" | "subtract" | "intersect">("replace");
  const [newDocDialogOpen, setNewDocDialogOpen] = useState(false);
  const [showGrid, setShowGrid] = useState(false);
  const [showGuides, setShowGuides] = useState(false);

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

  // Tauri file drop handler
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        const win = getCurrentWindow();
        unlisten = await win.onDragDropEvent(async (event) => {
          if (event.payload.type === "drop") {
            const paths = event.payload.paths;
            if (!paths || paths.length === 0) return;
            const filePath = paths[0];
            // Only attempt to open .json project files
            if (filePath.endsWith(".json") || filePath.endsWith(".retas")) {
              try {
                await openDocument(filePath);
                window.dispatchEvent(new CustomEvent("retas:state-changed"));
              } catch (e) {
                console.error("拖拽打开文件失败:", e);
              }
            }
          }
        });
      } catch {
        // Not running in Tauri environment
      }
    })();
    return () => { unlisten?.(); };
  }, []);

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
      params: { 
        tool: currentTool, 
        zoom, 
        color: brushColor, 
        brushSize, 
        onionSkin: onionSkinSettings, 
        currentFrame, 
        totalFrames,
        selection: selectionData,
        selectionTool,
        selectionMode,
        onSelectionChange: setSelectionData,
      },
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
        id: "perspectiveGuide",
        component: "perspectiveGuide",
        title: "透视参考线",
        position: { referencePanel: "color", direction: "below" },
      });
    } else if (workspace === "animation") {
      api.addPanel({
        id: "animationProps",
        component: "animationProps",
        title: "动画属性",
        position: { referencePanel: "canvas", direction: "right" },
        params: { currentFrame, totalFrames, fps },
      });
      api.addPanel({
        id: "xsheet",
        component: "xsheet",
        title: "X-Sheet",
        position: { referencePanel: "animationProps", direction: "below" },
        params: { currentFrame, totalFrames, onFrameChange: (frame: number) => setCurrentFrame(frame) },
      });
      api.addPanel({
        id: "lightTable",
        component: "lightTable",
        title: "灯光台",
        position: { referencePanel: "xsheet", direction: "below" },
        params: { currentFrame },
      });
      api.addPanel({
        id: "motionCheck",
        component: "motionCheck",
        title: "动检",
        position: { referencePanel: "lightTable", direction: "below" },
        params: { currentFrame },
      });
      api.addPanel({
        id: "cutSystem",
        component: "cutSystem",
        title: "卡系统",
        position: { referencePanel: "motionCheck", direction: "below" },
        params: { currentFrame },
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
        id: "effects",
        component: "effectsPanel",
        title: "效果",
        position: { referencePanel: "blendModes", direction: "below" },
      });
      api.addPanel({
        id: "renderQueue",
        component: "renderQueue",
        title: "渲染队列",
        position: { referencePanel: "effects", direction: "below" },
      });
    }
  }, [currentTool, zoom, brushColor, brushSize, currentWorkspace, isPlaying, currentFrame, totalFrames, fps, onionSkinSettings, selectionData, selectionTool, selectionMode]);

  const updateCanvas = useCallback(() => {
    if (apiRef.current) {
      const panel = apiRef.current.getPanel("canvas");
      if (panel) {
        panel.api.updateParameters({ 
          tool: currentTool, 
          zoom, 
          color: brushColor, 
          brushSize,
          onionSkin: onionSkinSettings,
          currentFrame,
          totalFrames,
          showGrid,
          showGuides,
          selection: selectionData,
          selectionTool,
          selectionMode,
          onSelectionChange: setSelectionData,
          onColorPick: setBrushColor,
          activeLayerId,
        });
      }
    }
  }, [currentTool, zoom, brushColor, brushSize, onionSkinSettings, currentFrame, totalFrames, showGrid, showGuides, selectionData, selectionTool, selectionMode, activeLayerId]);

  const updateColorPanel = useCallback(() => {
    if (apiRef.current) {
      const panel = apiRef.current.getPanel("color");
      if (panel) {
        panel.api.updateParameters({ color: brushColor, onColorChange: setBrushColor, brushSize, onBrushSizeChange: setBrushSize });
      }
    }
  }, [brushColor, brushSize]);

  const updateAnimationPanels = useCallback(() => {
    if (!apiRef.current) return;
    const xsheetPanel = apiRef.current.getPanel("xsheet");
    if (xsheetPanel) {
      xsheetPanel.api.updateParameters({ currentFrame, totalFrames, onFrameChange: (frame: number) => setCurrentFrame(frame) });
    }
    const animPropsPanel = apiRef.current.getPanel("animationProps");
    if (animPropsPanel) {
      animPropsPanel.api.updateParameters({ currentFrame, totalFrames, fps });
    }
  }, [currentFrame, totalFrames, fps]);

  useEffect(() => {
    updateCanvas();
    updateColorPanel();
    updateAnimationPanels();
  }, [updateCanvas, updateColorPanel, updateAnimationPanels]);

  useEffect(() => {
    if (!apiRef.current) return;
    const api = apiRef.current;

    const rightPanelIds = ["color", "animationProps", "blendModes", "effects", "renderQueue", "lightTable", "motionCheck", "cutSystem", "perspectiveGuide"];
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
    } else if (currentWorkspace === "animation") {
      api.addPanel({
        id: "animationProps",
        component: "animationProps",
        title: "动画属性",
        position: { referencePanel: "canvas", direction: "right" },
      });
    } else if (currentWorkspace === "coloring") {
      api.addPanel({
        id: "color",
        component: "colorPanel",
        title: "颜色",
        position: { referencePanel: "canvas", direction: "right" },
        params: { color: brushColor, onColorChange: setBrushColor, brushSize, onBrushSizeChange: setBrushSize },
      });
    } else if (currentWorkspace === "compositing") {
      api.addPanel({
        id: "blendModes",
        component: "blendModes",
        title: "混合模式",
        position: { referencePanel: "canvas", direction: "right" },
      });
      api.addPanel({
        id: "effects",
        component: "effectsPanel",
        title: "效果",
        position: { referencePanel: "blendModes", direction: "below" },
      });
    }
  }, [currentWorkspace, brushColor, brushSize]);

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
        await openDocument(path);
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
          <Button small icon={<FilePlus size={14} />} onClick={() => setNewDocDialogOpen(true)}>新建</Button>
          <Button small icon={<FolderOpen size={14} />} onClick={handleOpen}>打开</Button>
          <Button small icon={<Save size={14} />} onClick={handleSave}>保存</Button>
        </ButtonGroup>

        <div style={{ width: 1, height: 20, background: "#2d3139", margin: "0 6px" }} />

        <ButtonGroup minimal>
          <Button
            small
            data-testid="toolbar-undo"
            icon={<Undo2 size={14} />}
            onClick={handleUndo}
            disabled={!undoAvailable}
            title="撤销 (Ctrl+Z)"
          />
          <Button
            small
            data-testid="toolbar-redo"
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

        {currentTool === "select" && (
          <Popover
            content={
              <div style={{ padding: 8, minWidth: 200 }}>
                <SelectionToolPanel
                  selection={selectionData}
                  onSelectionChange={setSelectionData}
                  onToolChange={setSelectionTool}
                  onModeChange={setSelectionMode}
                  initialTool={selectionTool}
                  initialMode={selectionMode}
                />
              </div>
            }
            position="bottom"
          >
            <Button small minimal>
              选区工具
            </Button>
          </Popover>
        )}

        <OnionSkinPanel 
          settings={onionSkinSettings} 
          onSettingsChange={setOnionSkinSettings} 
        />

        <ButtonGroup minimal>
          <Button small icon={<ZoomOut size={12} />} onClick={() => setZoom(z => Math.max(10, z - 10))} />
          <span style={{ padding: "0 6px", fontSize: 11, display: "flex", alignItems: "center", color: "#8b949e" }}>
            {zoom}%
          </span>
          <Button small icon={<ZoomIn size={12} />} onClick={() => setZoom(z => Math.min(500, z + 10))} />
        </ButtonGroup>

        <ButtonGroup minimal style={{ marginLeft: 6 }}>
          <Button
            small
            icon={<Grid3X3 size={12} />}
            active={showGrid}
            onClick={() => setShowGrid(g => !g)}
            title="显示网格"
          />
          <Button
            small
            icon={<Ruler size={12} />}
            active={showGuides}
            onClick={() => setShowGuides(g => !g)}
            title="显示参考线"
          />
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

      <NewDocumentDialog isOpen={newDocDialogOpen} onClose={() => setNewDocDialogOpen(false)} />
    </div>
  );
}

export default App;
