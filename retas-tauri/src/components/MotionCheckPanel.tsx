import { useState, useEffect, useCallback } from "react";
import {
  getMotionCheck, setMotionCheck, toggleMotionTrails,
  clearTrails, MotionCheckInfo,
} from "../api";

interface Props {
  currentFrame?: number;
}

export default function MotionCheckPanel({ currentFrame = 0 }: Props) {
  const [info, setInfo] = useState<MotionCheckInfo | null>(null);
  const [enabled, setEnabled] = useState(false);
  const [mode, setMode] = useState("Overlay");
  const [compFrames, setCompFrames] = useState("1");
  const [opacity, setOpacity] = useState(0.5);
  const [showTrails, setShowTrails] = useState(false);
  const [trailLength, setTrailLength] = useState(5);

  const refresh = useCallback(async () => {
    const data = await getMotionCheck(currentFrame);
    if (data) {
      setInfo(data);
      setEnabled(data.enabled);
      setMode(data.mode);
      setCompFrames(data.comparison_frames.join(","));
      setOpacity(data.overlay_opacity);
      setShowTrails(data.show_trails);
      setTrailLength(data.trail_length);
    }
  }, [currentFrame]);

  useEffect(() => { refresh(); }, [refresh]);

  const apply = async () => {
    const frames = compFrames.split(",").map(s => parseInt(s.trim(), 10)).filter(n => !isNaN(n));
    await setMotionCheck(currentFrame, enabled, mode, frames, opacity);
    await refresh();
  };

  const applyTrails = async () => {
    await toggleMotionTrails(currentFrame, showTrails, trailLength);
    await refresh();
  };

  const handleClearTrails = async () => {
    await clearTrails();
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
    btnDanger: { background: "#5a2020", border: "1px solid #733", color: "#faa", padding: "3px 8px", borderRadius: 3, cursor: "pointer", fontSize: 11 },
    select: { background: "#2a2a2a", border: "1px solid #444", color: "#eee", padding: "2px 4px", borderRadius: 3, fontSize: 11, flex: 1 },
    checkbox: { accentColor: "#4a9eff" },
    range: { flex: 1, height: 3 },
    status: { display: "inline-block", width: 8, height: 8, borderRadius: "50%", marginRight: 4 },
  };

  return (
    <div style={s.panel}>
      <div style={s.section}>
        <div style={s.label}>动检设置 (Motion Check)</div>
        <div style={s.row}>
          <input type="checkbox" checked={enabled} onChange={e => setEnabled(e.target.checked)} style={s.checkbox} />
          <span>启用动检</span>
          <span style={{ ...s.status, background: enabled ? "#4a4" : "#555" }} />
        </div>
        <div style={s.row}>
          <span style={{ width: 56 }}>比较模式</span>
          <select value={mode} onChange={e => setMode(e.target.value)} style={s.select}>
            <option value="Overlay">叠加</option>
            <option value="Difference">差值</option>
            <option value="SideBySide">并排</option>
            <option value="OnionSkin">洋葱皮</option>
          </select>
        </div>
        <div style={s.row}>
          <span style={{ width: 56 }}>比较帧</span>
          <input type="text" value={compFrames} placeholder="1,2"
            onChange={e => setCompFrames(e.target.value)} style={s.inputWide}
            title="逗号分隔的帧偏移 (如 1,2,3)" />
        </div>
        <div style={s.row}>
          <span style={{ width: 56 }}>叠加透明</span>
          <input type="range" min={0} max={1} step={0.05} value={opacity}
            onChange={e => setOpacity(Number(e.target.value))} style={s.range} />
          <span style={{ width: 28, textAlign: "right" as const }}>{Math.round(opacity * 100)}%</span>
        </div>
        <div style={{ ...s.row, justifyContent: "flex-end" }}>
          <button style={s.btn} onClick={apply}>应用</button>
        </div>
      </div>

      <div style={s.section}>
        <div style={s.label}>运动轨迹 (Motion Trails)</div>
        <div style={s.row}>
          <input type="checkbox" checked={showTrails} onChange={e => setShowTrails(e.target.checked)} style={s.checkbox} />
          <span>显示轨迹</span>
        </div>
        <div style={s.row}>
          <span style={{ width: 56 }}>轨迹长度</span>
          <input type="number" value={trailLength} min={1} max={50}
            onChange={e => setTrailLength(Number(e.target.value))} style={s.input} />
          <span>帧</span>
        </div>
        <div style={{ ...s.row, justifyContent: "flex-end", gap: 6 }}>
          <button style={s.btnDanger} onClick={handleClearTrails}>清除轨迹</button>
          <button style={s.btn} onClick={applyTrails}>应用</button>
        </div>
      </div>

      <div style={{ color: "#555", fontSize: 10, padding: "4px 0" }}>
        帧 {currentFrame} | {info?.enabled ? "动检开启" : "动检关闭"}
        {info?.show_trails && " | 轨迹可见"}
      </div>
    </div>
  );
}
