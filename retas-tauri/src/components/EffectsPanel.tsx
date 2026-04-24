import { useState } from "react";
import { Button, Slider, HTMLSelect, Intent } from "@blueprintjs/core";
import { applyEffect, EffectParams } from "../api";

const EFFECTS = [
  { value: "gaussianBlur", label: "高斯模糊" },
  { value: "brightnessContrast", label: "亮度/对比度" },
  { value: "hueSaturation", label: "色相/饱和度" },
  { value: "invert", label: "反相" },
  { value: "posterize", label: "色调分离" },
  { value: "sharpen", label: "锐化" },
];

export default function EffectsPanel() {
  const [selectedEffect, setSelectedEffect] = useState("gaussianBlur");
  const [radius, setRadius] = useState(5);
  const [brightness, setBrightness] = useState(0);
  const [contrast, setContrast] = useState(0);
  const [hue, setHue] = useState(0);
  const [saturation, setSaturation] = useState(0);
  const [lightness, setLightness] = useState(0);
  const [levels, setLevels] = useState(4);
  const [isApplying, setIsApplying] = useState(false);

  const handleApply = async () => {
    setIsApplying(true);
    try {
      const params: EffectParams = { effectType: selectedEffect };

      switch (selectedEffect) {
        case "gaussianBlur":
          params.radius = radius;
          break;
        case "brightnessContrast":
          params.brightness = brightness / 100;
          params.contrast = contrast / 100;
          break;
        case "hueSaturation":
          params.hue = hue;
          params.saturation = saturation / 100;
          params.lightness = lightness / 100;
          break;
        case "invert":
          break;
        case "posterize":
          params.radius = levels;
          break;
        case "sharpen":
          params.radius = radius / 10;
          break;
      }

      await applyEffect(params);
      window.dispatchEvent(new CustomEvent("retas:state-changed"));
    } catch (e) {
      console.error("应用效果失败:", e);
    } finally {
      setIsApplying(false);
    }
  };

  const renderParams = () => {
    switch (selectedEffect) {
      case "gaussianBlur":
      case "sharpen":
        return (
          <div style={{ marginBottom: 8 }}>
            <div style={{ fontSize: 11, color: "#8b949e", marginBottom: 4 }}>
              {selectedEffect === "gaussianBlur" ? "模糊半径" : "锐化量"}
            </div>
            <Slider min={1} max={50} stepSize={1} value={radius} onChange={setRadius} labelStepSize={10} />
          </div>
        );
      case "brightnessContrast":
        return (
          <>
            <div style={{ marginBottom: 8 }}>
              <div style={{ fontSize: 11, color: "#8b949e", marginBottom: 4 }}>亮度</div>
              <Slider min={-100} max={100} stepSize={1} value={brightness} onChange={setBrightness} labelStepSize={50} />
            </div>
            <div style={{ marginBottom: 8 }}>
              <div style={{ fontSize: 11, color: "#8b949e", marginBottom: 4 }}>对比度</div>
              <Slider min={-100} max={100} stepSize={1} value={contrast} onChange={setContrast} labelStepSize={50} />
            </div>
          </>
        );
      case "hueSaturation":
        return (
          <>
            <div style={{ marginBottom: 8 }}>
              <div style={{ fontSize: 11, color: "#8b949e", marginBottom: 4 }}>色相</div>
              <Slider min={-180} max={180} stepSize={1} value={hue} onChange={setHue} labelStepSize={90} />
            </div>
            <div style={{ marginBottom: 8 }}>
              <div style={{ fontSize: 11, color: "#8b949e", marginBottom: 4 }}>饱和度</div>
              <Slider min={-100} max={100} stepSize={1} value={saturation} onChange={setSaturation} labelStepSize={50} />
            </div>
            <div style={{ marginBottom: 8 }}>
              <div style={{ fontSize: 11, color: "#8b949e", marginBottom: 4 }}>明度</div>
              <Slider min={-100} max={100} stepSize={1} value={lightness} onChange={setLightness} labelStepSize={50} />
            </div>
          </>
        );
      case "posterize":
        return (
          <div style={{ marginBottom: 8 }}>
            <div style={{ fontSize: 11, color: "#8b949e", marginBottom: 4 }}>色阶数</div>
            <Slider min={2} max={32} stepSize={1} value={levels} onChange={setLevels} labelStepSize={10} />
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div style={{ padding: 12 }}>
      <div style={{ fontSize: 11, fontWeight: 600, color: "#8b949e", marginBottom: 10, textTransform: "uppercase", letterSpacing: 0.8 }}>
        图像效果
      </div>

      <div style={{ marginBottom: 12 }}>
        <HTMLSelect
          value={selectedEffect}
          onChange={(e) => setSelectedEffect(e.target.value)}
          fill
        >
          {EFFECTS.map(eff => (
            <option key={eff.value} value={eff.value}>{eff.label}</option>
          ))}
        </HTMLSelect>
      </div>

      {renderParams()}

      <Button
        intent={Intent.PRIMARY}
        fill
        onClick={handleApply}
        loading={isApplying}
        style={{ marginTop: 4 }}
      >
        应用效果
      </Button>

      <div style={{ fontSize: 11, color: "#484f58", marginTop: 8 }}>
        提示: 效果直接应用到当前帧，可用 Ctrl+Z 撤销
      </div>
    </div>
  );
}
