import { useState, useEffect } from "react";
import { Button, Slider, Switch, Popover, Menu, MenuDivider } from "@blueprintjs/core";
import { Layers } from "lucide-react";

export interface OnionSkinSettings {
  enabled: boolean;
  framesBefore: number;
  framesAfter: number;
  opacityBefore: number;
  opacityAfter: number;
  colorBefore: string;
  colorAfter: string;
  blendMode: "tint" | "overlay" | "difference" | "normal";
}

interface OnionSkinPanelProps {
  settings: OnionSkinSettings;
  onSettingsChange: (settings: OnionSkinSettings) => void;
}

const defaultSettings: OnionSkinSettings = {
  enabled: false,
  framesBefore: 1,
  framesAfter: 1,
  opacityBefore: 0.3,
  opacityAfter: 0.3,
  colorBefore: "#ff0000",
  colorAfter: "#00ff00",
  blendMode: "tint",
};

export function OnionSkinPanel({ settings, onSettingsChange }: OnionSkinPanelProps) {
  const [localSettings, setLocalSettings] = useState<OnionSkinSettings>({
    ...defaultSettings,
    ...settings,
  });

  useEffect(() => {
    onSettingsChange(localSettings);
  }, [localSettings, onSettingsChange]);

  const updateSetting = <K extends keyof OnionSkinSettings>(
    key: K,
    value: OnionSkinSettings[K]
  ) => {
    setLocalSettings((prev) => ({ ...prev, [key]: value }));
  };

  const settingsMenu = (
    <Menu style={{ padding: 12, minWidth: 280 }}>
      <div style={{ marginBottom: 12 }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 8 }}>
          <span style={{ fontSize: 11, fontWeight: 600, color: "#8b949e", textTransform: "uppercase", letterSpacing: 0.8 }}>
            洋葱皮设置
          </span>
          <Switch
            checked={localSettings.enabled}
            onChange={(e) => updateSetting("enabled", e.currentTarget.checked)}
            innerLabelChecked="开"
            innerLabel="关"
          />
        </div>
      </div>

      <MenuDivider />

      <div style={{ padding: "8px 0" }}>
        <div style={{ marginBottom: 12 }}>
          <label style={{ fontSize: 12, color: "#8b949e", display: "block", marginBottom: 4 }}>
            前置帧数: {localSettings.framesBefore}
          </label>
          <Slider
            min={0}
            max={5}
            stepSize={1}
            value={localSettings.framesBefore}
            onChange={(value) => updateSetting("framesBefore", value)}
            disabled={!localSettings.enabled}
          />
        </div>

        <div style={{ marginBottom: 12 }}>
          <label style={{ fontSize: 12, color: "#8b949e", display: "block", marginBottom: 4 }}>
            后置帧数: {localSettings.framesAfter}
          </label>
          <Slider
            min={0}
            max={5}
            stepSize={1}
            value={localSettings.framesAfter}
            onChange={(value) => updateSetting("framesAfter", value)}
            disabled={!localSettings.enabled}
          />
        </div>
      </div>

      <MenuDivider />

      <div style={{ padding: "8px 0" }}>
        <div style={{ marginBottom: 12 }}>
          <label style={{ fontSize: 12, color: "#8b949e", display: "block", marginBottom: 4 }}>
            前置帧透明度: {Math.round(localSettings.opacityBefore * 100)}%
          </label>
          <Slider
            min={0}
            max={100}
            stepSize={5}
            value={localSettings.opacityBefore * 100}
            onChange={(value) => updateSetting("opacityBefore", value / 100)}
            disabled={!localSettings.enabled}
          />
        </div>

        <div style={{ marginBottom: 12 }}>
          <label style={{ fontSize: 12, color: "#8b949e", display: "block", marginBottom: 4 }}>
            后置帧透明度: {Math.round(localSettings.opacityAfter * 100)}%
          </label>
          <Slider
            min={0}
            max={100}
            stepSize={5}
            value={localSettings.opacityAfter * 100}
            onChange={(value) => updateSetting("opacityAfter", value / 100)}
            disabled={!localSettings.enabled}
          />
        </div>
      </div>

      <MenuDivider />

      <div style={{ padding: "8px 0" }}>
        <div style={{ marginBottom: 12 }}>
          <label style={{ fontSize: 12, color: "#8b949e", display: "block", marginBottom: 8 }}>
            颜色设置
          </label>
          <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
            <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <span style={{ fontSize: 11, color: "#8b949e" }}>前置帧:</span>
              <input
                type="color"
                value={localSettings.colorBefore}
                onChange={(e) => updateSetting("colorBefore", e.target.value)}
                disabled={!localSettings.enabled}
                style={{
                  width: 32,
                  height: 24,
                  border: "1px solid #333",
                  borderRadius: 3,
                  cursor: localSettings.enabled ? "pointer" : "default",
                }}
              />
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <span style={{ fontSize: 11, color: "#8b949e" }}>后置帧:</span>
              <input
                type="color"
                value={localSettings.colorAfter}
                onChange={(e) => updateSetting("colorAfter", e.target.value)}
                disabled={!localSettings.enabled}
                style={{
                  width: 32,
                  height: 24,
                  border: "1px solid #333",
                  borderRadius: 3,
                  cursor: localSettings.enabled ? "pointer" : "default",
                }}
              />
            </div>
          </div>
        </div>
      </div>

      <MenuDivider />

      <div style={{ padding: "8px 0" }}>
        <label style={{ fontSize: 12, color: "#8b949e", display: "block", marginBottom: 8 }}>
          混合模式
        </label>
        <div style={{ display: "flex", flexWrap: "wrap", gap: 4 }}>
          {(["tint", "overlay", "difference", "normal"] as const).map((mode) => (
            <Button
              key={mode}
              small
              minimal={localSettings.blendMode !== mode}
              active={localSettings.blendMode === mode}
              onClick={() => updateSetting("blendMode", mode)}
              disabled={!localSettings.enabled}
              style={{ fontSize: 11 }}
            >
              {mode === "tint" && "着色"}
              {mode === "overlay" && "叠加"}
              {mode === "difference" && "差值"}
              {mode === "normal" && "正常"}
            </Button>
          ))}
        </div>
      </div>
    </Menu>
  );

  return (
    <Popover content={settingsMenu} position="bottom">
      <Button
        minimal
        small
        data-testid="onion-skin-toggle"
        icon={<Layers size={14} />}
        active={localSettings.enabled}
        title={`洋葱皮: ${localSettings.enabled ? "开启" : "关闭"}`}
      >
        洋葱皮
        {localSettings.enabled && (
          <span style={{ marginLeft: 6, fontSize: 10, color: "#8b949e" }}>
            ({localSettings.framesBefore}/{localSettings.framesAfter})
          </span>
        )}
      </Button>
    </Popover>
  );
}

export default OnionSkinPanel;
