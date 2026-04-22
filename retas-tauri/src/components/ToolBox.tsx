import { useState } from "react";
import { Button } from "@blueprintjs/core";
import { 
  Brush, Eraser, PenTool, PaintBucket, 
  Move, Hand, Type, ZoomIn, 
  Lasso, Wand2
} from "lucide-react";

type ToolCategory = "brush" | "select" | "shape" | "text" | "transform";

interface ToolDef {
  id: string;
  icon: React.ReactNode;
  label: string;
  category: ToolCategory;
  shortcut?: string;
}

const TOOLS: ToolDef[] = [
  { id: "brush", icon: <Brush size={18} />, label: "画笔", category: "brush", shortcut: "B" },
  { id: "pen", icon: <PenTool size={18} />, label: "钢笔", category: "brush", shortcut: "P" },
  { id: "eraser", icon: <Eraser size={18} />, label: "橡皮", category: "brush", shortcut: "E" },
  { id: "fill", icon: <PaintBucket size={18} />, label: "填充", category: "brush", shortcut: "F" },
  { id: "select", icon: <Move size={18} />, label: "选择", category: "select", shortcut: "V" },
  { id: "lasso", icon: <Lasso size={18} />, label: "套索", category: "select" },
  { id: "magic", icon: <Wand2 size={18} />, label: "魔棒", category: "select" },
  { id: "hand", icon: <Hand size={18} />, label: "抓手", category: "transform", shortcut: "H" },
  { id: "zoom", icon: <ZoomIn size={18} />, label: "缩放", category: "transform", shortcut: "Z" },
  { id: "text", icon: <Type size={18} />, label: "文字", category: "text", shortcut: "T" },
];

const CATEGORIES: { id: ToolCategory; label: string }[] = [
  { id: "brush", label: "笔刷" },
  { id: "select", label: "选择" },
  { id: "transform", label: "变换" },
  { id: "text", label: "文字" },
];

interface ToolBoxProps {
  currentTool: string;
  onToolChange: (tool: string) => void;
}

export default function ToolBox({ currentTool, onToolChange }: ToolBoxProps) {
  const [expandedCategory, setExpandedCategory] = useState<ToolCategory>("brush");

  const toolsByCategory = (category: ToolCategory) => TOOLS.filter(t => t.category === category);

  return (
    <div style={{
      display: "flex",
      flexDirection: "column",
      width: 52,
      background: "#1a1d24",
      borderRight: "1px solid #2d3139",
      flexShrink: 0,
    }}>
      {CATEGORIES.map(cat => (
        <div key={cat.id} style={{ position: "relative" }}>
          <Button
            minimal
            small
            style={{
              width: 52,
              height: 52,
              borderRadius: 0,
              background: expandedCategory === cat.id ? "#2d3139" : "transparent",
              color: expandedCategory === cat.id ? "#58a6ff" : "#8b949e",
              borderLeft: expandedCategory === cat.id ? "2px solid #58a6ff" : "2px solid transparent",
            }}
            onClick={() => setExpandedCategory(expandedCategory === cat.id ? (cat.id as ToolCategory) : cat.id)}
            title={cat.label}
          >
            {toolsByCategory(cat.id)[0]?.icon}
          </Button>
          
          {expandedCategory === cat.id && (
            <div style={{
              position: "absolute",
              left: 56,
              top: 0,
              background: "#21252b",
              border: "1px solid #2d3139",
              borderRadius: 4,
              zIndex: 1000,
              boxShadow: "0 4px 12px rgba(0,0,0,0.5)",
              padding: "4px",
              display: "grid",
              gridTemplateColumns: "repeat(2, 40px)",
              gap: 2,
            }}>
              {toolsByCategory(cat.id).map(tool => (
                <Button
                  key={tool.id}
                  minimal
                  small
                  active={currentTool === tool.id}
                  onClick={() => onToolChange(tool.id)}
                  style={{
                    width: 40,
                    height: 40,
                    borderRadius: 4,
                    background: currentTool === tool.id ? "#1f6feb" : "transparent",
                    color: currentTool === tool.id ? "white" : "#c9d1d9",
                    padding: 0,
                  }}
                  title={`${tool.label} ${tool.shortcut ? `[${tool.shortcut}]` : ""}`}
                >
                  {tool.icon}
                </Button>
              ))}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
