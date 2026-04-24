import { Button, Slider } from "@blueprintjs/core";

const PRESETS = [
  "#000000", "#FFFFFF", "#FF0000", "#00FF00", "#0000FF",
  "#FFFF00", "#FF00FF", "#00FFFF", "#FFA500", "#800080"
];

interface ColorPanelProps {
  color: string;
  onColorChange: (color: string) => void;
  brushSize: number;
  onBrushSizeChange: (size: number) => void;
}

export default function ColorPanel({ color, onColorChange, brushSize, onBrushSizeChange }: ColorPanelProps) {
  return (
    <div>
      <div className="panel">
        <div className="panel-title">颜色</div>
        <div
          data-testid="current-color-preview"
          style={{
            width: "100%",
            height: 40,
            background: color,
            borderRadius: 4,
            border: "1px solid #555",
            marginBottom: 8,
          }}
        />
        <div style={{ fontSize: 11, color: "#888", marginBottom: 8 }}>{color}</div>
        
        <div className="panel-title">预设</div>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(5, 1fr)", gap: 4 }}>
          {PRESETS.map(c => (
            <Button
              key={c}
              minimal
              data-testid={`color-preset-${c.replace("#", "")}`}
              style={{
                width: 32,
                height: 32,
                background: c,
                border: color === c ? "2px solid white" : "1px solid #555",
                borderRadius: 4,
                padding: 0,
              }}
              onClick={() => onColorChange(c)}
            />
          ))}
        </div>
      </div>
      
      <div className="panel">
        <div className="panel-title">笔刷大小</div>
        <Slider
          min={1}
          max={100}
          stepSize={1}
          labelStepSize={20}
          value={brushSize}
          onChange={onBrushSizeChange}
        />
        <div data-testid="brush-size-display" style={{ fontSize: 11, color: "#888", textAlign: "center", marginTop: 4 }}>
          {brushSize}px
        </div>
      </div>
    </div>
  );
}
