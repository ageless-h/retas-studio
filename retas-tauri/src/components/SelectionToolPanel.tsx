import { useState, useCallback, useEffect } from "react";
import { Button, ButtonGroup, Slider, Popover, Menu } from "@blueprintjs/core";
import { Square, Circle, Pencil, Wand2, Plus, Minus, X, Paintbrush, FlipVertical, Eraser } from "lucide-react";
import { applySelectionToLayer, clearSelection } from "../api";

export type SelectionToolType = "rect" | "ellipse" | "lasso" | "magicWand";
export type SelectionMode = "replace" | "add" | "subtract" | "intersect";

export interface SelectionData {
  type: SelectionToolType;
  mode: SelectionMode;
  points: { x: number; y: number }[];
  rect?: { x: number; y: number; width: number; height: number };
  feather: number;
  tolerance: number;
  contiguous: boolean;
}

interface SelectionToolPanelProps {
  selection: SelectionData | null;
  onSelectionChange: (selection: SelectionData | null) => void;
  onApplySelection?: () => void;
  onToolChange?: (tool: SelectionToolType) => void;
  onModeChange?: (mode: SelectionMode) => void;
  initialTool?: SelectionToolType;
  initialMode?: SelectionMode;
}

interface Point {
  x: number;
  y: number;
}

export function SelectionToolPanel({
  selection,
  onSelectionChange,
  onApplySelection,
  onToolChange,
  onModeChange,
  initialTool = "rect",
  initialMode = "replace",
}: SelectionToolPanelProps) {
  const [tool, setTool] = useState<SelectionToolType>(initialTool);
  const [mode, setMode] = useState<SelectionMode>(initialMode);
  const [feather, setFeather] = useState(0);
  const [tolerance, setTolerance] = useState(32);
  const [contiguous, setContiguous] = useState(true);

  useEffect(() => {
    if (selection) {
      setTool(selection.type);
      setMode(selection.mode);
      setFeather(selection.feather);
      setTolerance(selection.tolerance);
      setContiguous(selection.contiguous);
    }
  }, [selection]);

  const handleToolChange = (newTool: SelectionToolType) => {
    setTool(newTool);
    onToolChange?.(newTool);
    if (selection) {
      onSelectionChange({ ...selection, type: newTool });
    }
  };

  const handleModeChange = (newMode: SelectionMode) => {
    setMode(newMode);
    onModeChange?.(newMode);
    if (selection) {
      onSelectionChange({ ...selection, mode: newMode });
    }
  };

  const handleFeatherChange = (value: number) => {
    setFeather(value);
    if (selection) {
      onSelectionChange({ ...selection, feather: value });
    }
  };

  const handleToleranceChange = (value: number) => {
    setTolerance(value);
    if (selection) {
      onSelectionChange({ ...selection, tolerance: value });
    }
  };

  const handleContiguousChange = () => {
    const newValue = !contiguous;
    setContiguous(newValue);
    if (selection) {
      onSelectionChange({ ...selection, contiguous: newValue });
    }
  };

  const handleClear = async () => {
    try {
      await clearSelection();
      onSelectionChange(null);
      window.dispatchEvent(new CustomEvent("retas:state-changed"));
    } catch (e) {
      console.error("清除选区失败:", e);
      onSelectionChange(null);
    }
  };

  const handleSelectionOp = async (operation: string) => {
    try {
      await applySelectionToLayer("", operation);
      window.dispatchEvent(new CustomEvent("retas:state-changed"));
    } catch (e) {
      console.error(`选区操作 ${operation} 失败:`, e);
    }
  };

  const settingsMenu = (
    <Menu style={{ padding: 12, minWidth: 200 }}>
      <div style={{ marginBottom: 12 }}>
        <label style={{ fontSize: 12, color: "#8b949e", display: "block", marginBottom: 4 }}>
          羽化: {feather}px
        </label>
        <Slider
          min={0}
          max={50}
          stepSize={1}
          value={feather}
          onChange={handleFeatherChange}
        />
      </div>

      {tool === "magicWand" && (
        <>
          <div style={{ marginBottom: 12 }}>
            <label style={{ fontSize: 12, color: "#8b949e", display: "block", marginBottom: 4 }}>
              容差: {tolerance}
            </label>
            <Slider
              min={0}
              max={100}
              stepSize={1}
              value={tolerance}
              onChange={handleToleranceChange}
            />
          </div>

          <div style={{ marginBottom: 8 }}>
            <Button
              small
              minimal
              active={contiguous}
              onClick={handleContiguousChange}
            >
              连续选区
            </Button>
          </div>
        </>
      )}
    </Menu>
  );

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
        <span style={{ fontSize: 11, color: "#8b949e", minWidth: 40 }}>工具:</span>
        <ButtonGroup minimal>
          <Button
            small
            active={tool === "rect"}
            onClick={() => handleToolChange("rect")}
            icon={<Square size={14} />}
            title="矩形选区"
          />
          <Button
            small
            active={tool === "ellipse"}
            onClick={() => handleToolChange("ellipse")}
            icon={<Circle size={14} />}
            title="椭圆选区"
          />
          <Button
            small
            active={tool === "lasso"}
            onClick={() => handleToolChange("lasso")}
            icon={<Pencil size={14} />}
            title="套索选区"
          />
          <Button
            small
            active={tool === "magicWand"}
            onClick={() => handleToolChange("magicWand")}
            icon={<Wand2 size={14} />}
            title="魔术棒"
          />
        </ButtonGroup>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
        <span style={{ fontSize: 11, color: "#8b949e", minWidth: 40 }}>模式:</span>
        <ButtonGroup minimal>
          <Button
            small
            active={mode === "replace"}
            onClick={() => handleModeChange("replace")}
            title="替换选区"
          >
            <Square size={12} />
          </Button>
          <Button
            small
            active={mode === "add"}
            onClick={() => handleModeChange("add")}
            icon={<Plus size={14} />}
            title="添加到选区"
          />
          <Button
            small
            active={mode === "subtract"}
            onClick={() => handleModeChange("subtract")}
            icon={<Minus size={14} />}
            title="从选区减去"
          />
          <Button
            small
            active={mode === "intersect"}
            onClick={() => handleModeChange("intersect")}
            icon={<X size={14} />}
            title="与选区交叉"
          />
        </ButtonGroup>
      </div>

      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <Popover content={settingsMenu} position="bottom">
          <Button small minimal>
            设置
          </Button>
        </Popover>

        {selection && (
          <>
            <Button small minimal onClick={handleClear} icon={<Eraser size={12} />} title="清除选区 (Delete)">
              清除
            </Button>
            <Button small minimal onClick={() => handleSelectionOp("fill")} icon={<Paintbrush size={12} />} title="填充选区">
              填充
            </Button>
            <Button small minimal onClick={() => handleSelectionOp("invert")} icon={<FlipVertical size={12} />} title="反选">
              反选
            </Button>
            {onApplySelection && (
              <Button small intent="primary" onClick={onApplySelection}>
                应用
              </Button>
            )}
          </>
        )}
      </div>

      {selection && (
        <div style={{ fontSize: 11, color: "#8b949e", marginTop: 4 }}>
          {selection.rect && (
            <span>
              选区: {Math.round(selection.rect.width)} × {Math.round(selection.rect.height)}px
            </span>
          )}
          {selection.points && selection.points.length > 0 && !selection.rect && (
            <span>套索: {selection.points.length} 个点</span>
          )}
        </div>
      )}
    </div>
  );
}

export function useSelectionCanvas(
  tool: SelectionToolType,
  mode: SelectionMode,
  feather: number,
  tolerance: number,
  contiguous: boolean
) {
  const [isDrawing, setIsDrawing] = useState(false);
  const [startPoint, setStartPoint] = useState<Point | null>(null);
  const [currentPoints, setCurrentPoints] = useState<Point[]>([]);
  const [selectionRect, setSelectionRect] = useState<{
    x: number;
    y: number;
    width: number;
    height: number;
  } | null>(null);

  const handleMouseDown = useCallback(
    (pos: Point, currentSelection: SelectionData | null) => {
      setIsDrawing(true);
      setStartPoint(pos);

      if (tool === "lasso") {
        setCurrentPoints([pos]);
      } else if (tool === "rect" || tool === "ellipse") {
        setSelectionRect({ x: pos.x, y: pos.y, width: 0, height: 0 });
      }

      if (mode === "replace" || !currentSelection) {
        return null;
      }
      return currentSelection;
    },
    [tool, mode]
  );

  const handleMouseMove = useCallback(
    (pos: Point) => {
      if (!isDrawing || !startPoint) return null;

      if (tool === "lasso") {
        setCurrentPoints((prev) => [...prev, pos]);
        return {
          type: "lasso",
          mode,
          points: [...currentPoints, pos],
          feather,
          tolerance,
          contiguous,
        };
      } else if (tool === "rect" || tool === "ellipse") {
        const rect = {
          x: Math.min(startPoint.x, pos.x),
          y: Math.min(startPoint.y, pos.y),
          width: Math.abs(pos.x - startPoint.x),
          height: Math.abs(pos.y - startPoint.y),
        };
        setSelectionRect(rect);
        return {
          type: tool,
          mode,
          points: [],
          rect,
          feather,
          tolerance,
          contiguous,
        };
      }

      return null;
    },
    [isDrawing, startPoint, tool, mode, currentPoints, feather, tolerance, contiguous]
  );

  const handleMouseUp = useCallback(() => {
    setIsDrawing(false);
    setStartPoint(null);

    if (tool === "lasso" && currentPoints.length > 2) {
      const closedPoints = [...currentPoints, currentPoints[0]];
      setCurrentPoints(closedPoints);
      return {
        type: "lasso" as SelectionToolType,
        mode,
        points: closedPoints,
        feather,
        tolerance,
        contiguous,
      };
    }

    return null;
  }, [tool, mode, currentPoints, feather, tolerance, contiguous]);

  const reset = useCallback(() => {
    setIsDrawing(false);
    setStartPoint(null);
    setCurrentPoints([]);
    setSelectionRect(null);
  }, []);

  return {
    isDrawing,
    currentPoints,
    selectionRect,
    handleMouseDown,
    handleMouseMove,
    handleMouseUp,
    reset,
  };
}

export default SelectionToolPanel;
