import { Button, Slider } from "@blueprintjs/core";
import { useEffect, useRef, useState } from "react";

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

// --- HSV <-> Hex Helpers ---
function hexToHsv(hex: string) {
  let r = 0, g = 0, b = 0;
  if (hex.length === 4) {
    r = parseInt(hex[1] + hex[1], 16);
    g = parseInt(hex[2] + hex[2], 16);
    b = parseInt(hex[3] + hex[3], 16);
  } else if (hex.length === 7) {
    r = parseInt(hex.substring(1, 3), 16);
    g = parseInt(hex.substring(3, 5), 16);
    b = parseInt(hex.substring(5, 7), 16);
  }
  r /= 255; g /= 255; b /= 255;
  const max = Math.max(r, g, b), min = Math.min(r, g, b);
  let h = 0, s = 0;
  const v = max;
  const d = max - min;
  s = max === 0 ? 0 : d / max;
  if (max === min) {
    h = 0; // achromatic
  } else {
    switch (max) {
      case r: h = (g - b) / d + (g < b ? 6 : 0); break;
      case g: h = (b - r) / d + 2; break;
      case b: h = (r - g) / d + 4; break;
    }
    h /= 6;
  }
  return { h: h * 360, s, v };
}

function hsvToHex(h: number, s: number, v: number) {
  let r = 0, g = 0, b = 0;
  const i = Math.floor((h / 360) * 6);
  const f = (h / 360) * 6 - i;
  const p = v * (1 - s);
  const q = v * (1 - f * s);
  const t = v * (1 - (1 - f) * s);
  switch (i % 6) {
    case 0: r = v; g = t; b = p; break;
    case 1: r = q; g = v; b = p; break;
    case 2: r = p; g = v; b = t; break;
    case 3: r = p; g = q; b = v; break;
    case 4: r = t; g = p; b = v; break;
    case 5: r = v; g = p; b = q; break;
  }
  const toHex = (c: number) => {
    const hex = Math.round(c * 255).toString(16);
    return hex.length === 1 ? "0" + hex : hex;
  };
  return `#${toHex(r)}${toHex(g)}${toHex(b)}`.toUpperCase();
}

export default function ColorPanel({ color, onColorChange, brushSize, onBrushSizeChange }: ColorPanelProps) {
  const svCanvasRef = useRef<HTMLCanvasElement>(null);
  const hueCanvasRef = useRef<HTMLCanvasElement>(null);

  const [hsv, setHsv] = useState(() => hexToHsv(color));
  const [recentColors, setRecentColors] = useState<string[]>([]);
  const [isDraggingSV, setIsDraggingSV] = useState(false);
  const [isDraggingHue, setIsDraggingHue] = useState(false);

  // Sync internal hsv state when color prop changes externally
  useEffect(() => {
    setHsv(hexToHsv(color));
  }, [color]);

  // Update recent colors when color prop changes
  useEffect(() => {
    setRecentColors(prev => {
      const filtered = prev.filter(c => c.toUpperCase() !== color.toUpperCase());
      const newRecents = [color, ...filtered].slice(0, 12);
      return newRecents;
    });
  }, [color]);

  const updateColorFromHSV = (h: number, s: number, v: number) => {
    setHsv({ h, s, v });
    const hex = hsvToHex(h, s, v);
    onColorChange(hex);
  };

  // Draw SV Canvas
  useEffect(() => {
    const canvas = svCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    
    const width = canvas.width;
    const height = canvas.height;
    
    // Fill base hue color
    ctx.fillStyle = `hsl(${hsv.h}, 100%, 50%)`;
    ctx.fillRect(0, 0, width, height);
    
    // White gradient (Saturation)
    const whiteGrad = ctx.createLinearGradient(0, 0, width, 0);
    whiteGrad.addColorStop(0, "rgba(255,255,255,1)");
    whiteGrad.addColorStop(1, "rgba(255,255,255,0)");
    ctx.fillStyle = whiteGrad;
    ctx.fillRect(0, 0, width, height);
    
    // Black gradient (Value)
    const blackGrad = ctx.createLinearGradient(0, 0, 0, height);
    blackGrad.addColorStop(0, "rgba(0,0,0,0)");
    blackGrad.addColorStop(1, "rgba(0,0,0,1)");
    ctx.fillStyle = blackGrad;
    ctx.fillRect(0, 0, width, height);

    // Draw indicator
    const x = hsv.s * width;
    const y = (1 - hsv.v) * height;
    ctx.beginPath();
    ctx.arc(x, y, 4, 0, 2 * Math.PI);
    ctx.strokeStyle = "white";
    ctx.lineWidth = 1.5;
    ctx.stroke();
    ctx.beginPath();
    ctx.arc(x, y, 5, 0, 2 * Math.PI);
    ctx.strokeStyle = "black";
    ctx.lineWidth = 1;
    ctx.stroke();

  }, [hsv.h, hsv.s, hsv.v]);

  // Draw Hue Canvas
  useEffect(() => {
    const canvas = hueCanvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const width = canvas.width;
    const height = canvas.height;

    const hueGrad = ctx.createLinearGradient(0, 0, width, 0);
    hueGrad.addColorStop(0, "#ff0000");
    hueGrad.addColorStop(1/6, "#ffff00");
    hueGrad.addColorStop(2/6, "#00ff00");
    hueGrad.addColorStop(3/6, "#00ffff");
    hueGrad.addColorStop(4/6, "#0000ff");
    hueGrad.addColorStop(5/6, "#ff00ff");
    hueGrad.addColorStop(1, "#ff0000");

    ctx.fillStyle = hueGrad;
    ctx.fillRect(0, 0, width, height);

    // Draw indicator
    const x = (hsv.h / 360) * width;
    ctx.beginPath();
    ctx.rect(x - 2, 0, 4, height);
    ctx.fillStyle = "white";
    ctx.fill();
    ctx.strokeStyle = "black";
    ctx.stroke();

  }, [hsv.h]);


  const handleSVPick = (e: React.MouseEvent | MouseEvent) => {
    const canvas = svCanvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
    const y = Math.max(0, Math.min(e.clientY - rect.top, rect.height));
    const s = x / rect.width;
    const v = 1 - (y / rect.height);
    updateColorFromHSV(hsv.h, s, v);
  };

  const handleHuePick = (e: React.MouseEvent | MouseEvent) => {
    const canvas = hueCanvasRef.current;
    if (!canvas) return;
    const rect = canvas.getBoundingClientRect();
    const x = Math.max(0, Math.min(e.clientX - rect.left, rect.width));
    const h = (x / rect.width) * 360;
    updateColorFromHSV(h, hsv.s, hsv.v);
  };

  useEffect(() => {
    const handleMouseUp = () => {
      setIsDraggingSV(false);
      setIsDraggingHue(false);
    };
    const handleMouseMove = (e: MouseEvent) => {
      if (isDraggingSV) handleSVPick(e);
      if (isDraggingHue) handleHuePick(e);
    };

    if (isDraggingSV || isDraggingHue) {
      window.addEventListener("mousemove", handleMouseMove);
      window.addEventListener("mouseup", handleMouseUp);
    }
    return () => {
      window.removeEventListener("mousemove", handleMouseMove);
      window.removeEventListener("mouseup", handleMouseUp);
    };
  }, [isDraggingSV, isDraggingHue, hsv]);


  return (
    <div style={{ color: "#e6edf3" }}>
      <div className="panel" style={{ padding: 8, background: "#161b22", border: "1px solid #2d3139", borderRadius: 4, marginBottom: 8 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 8 }}>
            <div className="panel-title" style={{ margin: 0, color: "#8b949e", fontSize: 12, fontWeight: 600 }}>Color</div>
            <Button
                icon="tint"
                minimal
                small
                title="Eyedropper placeholder"
                style={{ color: "#8b949e" }}
            />
        </div>
        
        {/* Color Picker */}
        <div style={{ display: "flex", flexDirection: "column", gap: 8, alignItems: "center", marginBottom: 8 }}>
            <canvas
                ref={svCanvasRef}
                width={200}
                height={200}
                style={{ cursor: "crosshair", borderRadius: 4, border: "1px solid #2d3139" }}
                onMouseDown={(e) => {
                    setIsDraggingSV(true);
                    handleSVPick(e);
                }}
            />
            <canvas
                ref={hueCanvasRef}
                width={200}
                height={16}
                style={{ cursor: "crosshair", borderRadius: 4, border: "1px solid #2d3139" }}
                onMouseDown={(e) => {
                    setIsDraggingHue(true);
                    handleHuePick(e);
                }}
            />
        </div>

        <div
          data-testid="current-color-preview"
          style={{
            width: "100%",
            height: 24,
            background: color,
            borderRadius: 4,
            border: "1px solid #2d3139",
            marginBottom: 4,
          }}
        />
        <div style={{ fontSize: 11, color: "#8b949e", textAlign: "center", marginBottom: 12 }}>{color}</div>
        
        <div className="panel-title" style={{ margin: "0 0 4px 0", color: "#8b949e", fontSize: 12, fontWeight: 600 }}>Recent</div>
        <div style={{ display: "flex", flexWrap: "wrap", gap: 4, marginBottom: 12, minHeight: 24 }}>
          {recentColors.map((c, i) => (
             <Button
                key={`${c}-${i}`}
                minimal
                style={{
                  width: 24,
                  height: 24,
                  background: c,
                  border: color.toUpperCase() === c.toUpperCase() ? "2px solid #58a6ff" : "1px solid #2d3139",
                  borderRadius: 4,
                  padding: 0,
                  minWidth: 24,
                  minHeight: 24
                }}
                onClick={() => onColorChange(c)}
                title={c}
              />
          ))}
        </div>

        <div className="panel-title" style={{ margin: "0 0 4px 0", color: "#8b949e", fontSize: 12, fontWeight: 600 }}>Presets</div>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(5, 1fr)", gap: 4 }}>
          {PRESETS.map(c => (
            <Button
              key={c}
              minimal
              data-testid={`color-preset-${c.replace("#", "")}`}
              style={{
                width: "100%",
                aspectRatio: "1/1",
                background: c,
                border: color.toUpperCase() === c.toUpperCase() ? "2px solid #58a6ff" : "1px solid #2d3139",
                borderRadius: 4,
                padding: 0,
                minWidth: 0,
                minHeight: 0
              }}
              onClick={() => onColorChange(c)}
              title={c}
            />
          ))}
        </div>
      </div>
      
      <div className="panel" style={{ padding: 8, background: "#161b22", border: "1px solid #2d3139", borderRadius: 4 }}>
        <div className="panel-title" style={{ margin: "0 0 8px 0", color: "#8b949e", fontSize: 12, fontWeight: 600 }}>Brush Size</div>
        <div style={{ padding: "0 8px" }}>
            <Slider
              min={1}
              max={100}
              stepSize={1}
              labelStepSize={20}
              value={brushSize}
              onChange={onBrushSizeChange}
            />
        </div>
        <div data-testid="brush-size-display" style={{ fontSize: 11, color: "#8b949e", textAlign: "center", marginTop: 4 }}>
          {brushSize}px
        </div>
      </div>
    </div>
  );
}
