import { useState, useEffect, useCallback } from "react";
import {
  getLightTable, setOnionSkin, setOnionSkinColors,
  addReferenceLayer, removeReferenceLayer,
  setReferenceOpacity, toggleReferenceVisibility,
  LightTableInfo, ReferenceLayerInfo,
} from "../api";

interface Props {
  currentFrame?: number;
}

export default function LightTablePanel({ currentFrame = 0 }: Props) {
  const [info, setInfo] = useState<LightTableInfo | null>(null);
  const [enabled, setEnabled] = useState(false);
  const [framesBefore, setFramesBefore] = useState(1);
  const [framesAfter, setFramesAfter] = useState(1);
  const [opacityBefore, setOpacityBefore] = useState(0.3);
  const [opacityAfter, setOpacityAfter] = useState(0.3);
  const [blendMode, setBlendMode] = useState("Tint");
  const [newRefName, setNewRefName] = useState("");

  const refresh = useCallback(async () => {
    const data = await getLightTable(currentFrame);
    if (data) {
      setInfo(data);
      setEnabled(data.onion_skin.enabled);
      setFramesBefore(data.onion_skin.frames_before);
      setFramesAfter(data.onion_skin.frames_after);
      setOpacityBefore(data.onion_skin.opacity_before);
      setOpacityAfter(data.onion_skin.opacity_after);
      setBlendMode(data.onion_skin.blend_mode);
    }
  }, [currentFrame]);

  useEffect(() => { refresh(); }, [refresh]);

  const applyOnionSkin = async () => {
    await setOnionSkin(currentFrame, enabled, framesBefore, framesAfter, opacityBefore, opacityAfter);
    await refresh();
  };

  const applyColors = async () => {
    const colorBefore: [number, number, number, number] = [255, 0, 0, 255];
    const colorAfter: [number, number, number, number] = [0, 255, 0, 255];
    if (info) {
      colorBefore[0] = info.onion_skin.color_before[0];
      colorBefore[1] = info.onion_skin.color_before[1];
      colorBefore[2] = info.onion_skin.color_before[2];
      colorBefore[3] = info.onion_skin.color_before[3];
      colorAfter[0] = info.onion_skin.color_after[0];
      colorAfter[1] = info.onion_skin.color_after[1];
      colorAfter[2] = info.onion_skin.color_after[2];
      colorAfter[3] = info.onion_skin.color_after[3];
    }
    await setOnionSkinColors(currentFrame, colorBefore, colorAfter, blendMode);
    await refresh();
  };

  const handleAddRef = async () => {
    if (!newRefName.trim()) return;
    await addReferenceLayer(currentFrame, newRefName.trim());
    setNewRefName("");
    await refresh();
  };

  const handleRemoveRef = async (id: string) => {
    await removeReferenceLayer(currentFrame, id);
    await refresh();
  };

  const handleRefOpacity = async (id: string, val: number) => {
    await setReferenceOpacity(currentFrame, id, val);
    await refresh();
  };

  const handleToggleVis = async (id: string) => {
    await toggleReferenceVisibility(currentFrame, id);
    await refresh();
  };

  const s = {
    panel: { padding: 8, fontSize: 12, color: "#ccc", height: "100%", overflow: "auto" as const, background: "#1e1e1e" },
    section: { marginBottom: 10, borderBottom: "1px solid #333", paddingBottom: 8 },
    label: { color: "#888", fontSize: 10, textTransform: "uppercase" as const, marginBottom: 4 },
    row: { display: "flex", alignItems: "center" as const, gap: 6, marginBottom: 4 },
    input: { background: "#2a2a2a", border: "1px solid #444", color: "#eee", padding: "2px 4px", width: 48, borderRadius: 3, fontSize: 11 },
    inputWide: { background: "#2a2a2a", border: "1px solid #444", color: "#eee", padding: "2px 4px", flex: 1, borderRadius: 3, fontSize: 11 },
    btn: { background: "#3a3a3a", border: "1px solid #555", color: "#eee", padding: "3px 8px", borderRadius: 3, cursor: "pointer", fontSize: 11 },
    btnSm: { background: "none", border: "none", color: "#888", cursor: "pointer", fontSize: 11, padding: "0 4px" },
    select: { background: "#2a2a2a", border: "1px solid #444", color: "#eee", padding: "2px 4px", borderRadius: 3, fontSize: 11 },
    checkbox: { accentColor: "#4a9eff" },
    refItem: { display: "flex", alignItems: "center" as const, gap: 4, padding: "3px 0", borderBottom: "1px solid #2a2a2a" },
    range: { flex: 1, height: 3 },
  };

  return (
    <div style={s.panel}>
      <div style={s.section}>
        <div style={s.label}>洋葱皮 (Onion Skin)</div>
        <div style={s.row}>
          <input type="checkbox" checked={enabled} onChange={e => setEnabled(e.target.checked)} style={s.checkbox} />
          <span>启用</span>
        </div>
        <div style={s.row}>
          <span style={{ width: 36 }}>前</span>
          <input type="number" value={framesBefore} min={0} max={10}
            onChange={e => setFramesBefore(Number(e.target.value))} style={s.input} />
          <span style={{ width: 36 }}>后</span>
          <input type="number" value={framesAfter} min={0} max={10}
            onChange={e => setFramesAfter(Number(e.target.value))} style={s.input} />
        </div>
        <div style={s.row}>
          <span style={{ width: 56 }}>前透明度</span>
          <input type="range" min={0} max={1} step={0.05} value={opacityBefore}
            onChange={e => setOpacityBefore(Number(e.target.value))} style={s.range} />
          <span style={{ width: 28, textAlign: "right" as const }}>{Math.round(opacityBefore * 100)}%</span>
        </div>
        <div style={s.row}>
          <span style={{ width: 56 }}>后透明度</span>
          <input type="range" min={0} max={1} step={0.05} value={opacityAfter}
            onChange={e => setOpacityAfter(Number(e.target.value))} style={s.range} />
          <span style={{ width: 28, textAlign: "right" as const }}>{Math.round(opacityAfter * 100)}%</span>
        </div>
        <div style={s.row}>
          <span style={{ width: 56 }}>混合模式</span>
          <select value={blendMode} onChange={e => setBlendMode(e.target.value)} style={s.select}>
            <option value="Tint">着色</option>
            <option value="Overlay">叠加</option>
            <option value="Difference">差值</option>
            <option value="Normal">正常</option>
          </select>
        </div>
        <div style={{ ...s.row, justifyContent: "flex-end" }}>
          <button style={s.btn} onClick={applyOnionSkin}>应用设置</button>
          <button style={s.btn} onClick={applyColors}>应用颜色</button>
        </div>
      </div>

      <div style={s.section}>
        <div style={s.label}>参考图层 (Reference Layers)</div>
        <div style={s.row}>
          <input type="text" placeholder="参考名称..." value={newRefName}
            onChange={e => setNewRefName(e.target.value)} style={s.inputWide}
            onKeyDown={e => e.key === "Enter" && handleAddRef()} />
          <button style={s.btn} onClick={handleAddRef}>+</button>
        </div>
        {(info?.references || []).map((ref_: ReferenceLayerInfo) => (
          <div key={ref_.id} style={s.refItem}>
            <button style={s.btnSm} onClick={() => handleToggleVis(ref_.id)}
              title={ref_.visible ? "隐藏" : "显示"}>
              {ref_.visible ? "👁" : "─"}
            </button>
            <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" as const }}>
              {ref_.name}
            </span>
            <input type="range" min={0} max={1} step={0.05} value={ref_.opacity}
              onChange={e => handleRefOpacity(ref_.id, Number(e.target.value))}
              style={{ width: 60, height: 3 }} title={`透明度: ${Math.round(ref_.opacity * 100)}%`} />
            <button style={{ ...s.btnSm, color: "#f55" }} onClick={() => handleRemoveRef(ref_.id)} title="删除">✕</button>
          </div>
        ))}
        {(info?.references || []).length === 0 && (
          <div style={{ color: "#555", fontSize: 11, padding: 4 }}>无参考图层</div>
        )}
      </div>
    </div>
  );
}
