import { useState, useEffect, useCallback } from "react";
import {
  getGuides, addHorizontalGuide, addVerticalGuide,
  addOnePointPerspective, addTwoPointPerspective,
  removeGuide, GuideInfo,
} from "../api";

export default function PerspectiveGuidePanel() {
  const [guides, setGuides] = useState<GuideInfo | null>(null);
  const [hY, setHY] = useState(540);
  const [vX, setVX] = useState(960);
  const [vpX, setVpX] = useState(960);
  const [vpY, setVpY] = useState(540);
  const [vp2X, setVp2X] = useState(1920);
  const [vp2Y, setVp2Y] = useState(540);
  const [horizonY, setHorizonY] = useState(540);

  const refresh = useCallback(async () => {
    const data = await getGuides();
    setGuides(data);
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  const handleAddH = async () => { await addHorizontalGuide(hY); await refresh(); };
  const handleAddV = async () => { await addVerticalGuide(vX); await refresh(); };
  const handleAdd1P = async () => { await addOnePointPerspective(vpX, vpY); await refresh(); };
  const handleAdd2P = async () => { await addTwoPointPerspective(vpX, vpY, vp2X, vp2Y, horizonY); await refresh(); };
  const handleRemove = async (id: number) => { await removeGuide(id); await refresh(); };

  const s = {
    panel: { padding: 8, fontSize: 12, color: "#ccc", height: "100%", overflow: "auto" as const, background: "#1e1e1e" },
    section: { marginBottom: 10, borderBottom: "1px solid #333", paddingBottom: 8 },
    label: { color: "#888", fontSize: 10, textTransform: "uppercase" as const, marginBottom: 4 },
    row: { display: "flex", alignItems: "center" as const, gap: 6, marginBottom: 4 },
    input: { background: "#2a2a2a", border: "1px solid #444", color: "#eee", padding: "2px 4px", width: 52, borderRadius: 3, fontSize: 11 },
    btn: { background: "#3a3a3a", border: "1px solid #555", color: "#eee", padding: "3px 8px", borderRadius: 3, cursor: "pointer", fontSize: 11 },
    btnSm: { background: "none", border: "none", color: "#f55", cursor: "pointer", fontSize: 10, padding: "0 4px" },
    item: { display: "flex", alignItems: "center" as const, gap: 4, padding: "2px 0", borderBottom: "1px solid #2a2a2a" },
  };

  return (
    <div style={s.panel}>
      <div style={s.section}>
        <div style={s.label}>水平参考线</div>
        <div style={s.row}>
          <span>Y</span>
          <input type="number" value={hY} onChange={e => setHY(Number(e.target.value))} style={s.input} />
          <button style={s.btn} onClick={handleAddH}>添加</button>
        </div>
        {(guides?.horizontal || []).map(([id, y]) => (
          <div key={id} style={s.item}>
            <span style={{ color: "#0ff", fontSize: 10 }}>━</span>
            <span style={{ flex: 1, fontSize: 11 }}>Y={y.toFixed(0)}</span>
            <button style={s.btnSm} onClick={() => handleRemove(id)}>✕</button>
          </div>
        ))}
      </div>

      <div style={s.section}>
        <div style={s.label}>垂直参考线</div>
        <div style={s.row}>
          <span>X</span>
          <input type="number" value={vX} onChange={e => setVX(Number(e.target.value))} style={s.input} />
          <button style={s.btn} onClick={handleAddV}>添加</button>
        </div>
        {(guides?.vertical || []).map(([id, x]) => (
          <div key={id} style={s.item}>
            <span style={{ color: "#0ff", fontSize: 10 }}>┃</span>
            <span style={{ flex: 1, fontSize: 11 }}>X={x.toFixed(0)}</span>
            <button style={s.btnSm} onClick={() => handleRemove(id)}>✕</button>
          </div>
        ))}
      </div>

      <div style={s.section}>
        <div style={s.label}>透视参考线</div>
        <div style={s.row}>
          <span style={{ width: 28 }}>VP1</span>
          <input type="number" value={vpX} onChange={e => setVpX(Number(e.target.value))} style={s.input} placeholder="X" />
          <input type="number" value={vpY} onChange={e => setVpY(Number(e.target.value))} style={s.input} placeholder="Y" />
          <button style={s.btn} onClick={handleAdd1P}>1点透视</button>
        </div>
        <div style={s.row}>
          <span style={{ width: 28 }}>VP2</span>
          <input type="number" value={vp2X} onChange={e => setVp2X(Number(e.target.value))} style={s.input} placeholder="X" />
          <input type="number" value={vp2Y} onChange={e => setVp2Y(Number(e.target.value))} style={s.input} placeholder="Y" />
        </div>
        <div style={s.row}>
          <span style={{ width: 28 }}>地平</span>
          <input type="number" value={horizonY} onChange={e => setHorizonY(Number(e.target.value))} style={s.input} />
          <button style={s.btn} onClick={handleAdd2P}>2点透视</button>
        </div>
        {(guides?.perspective || []).map(pg => (
          <div key={pg.id} style={s.item}>
            <span style={{ color: "#f0f", fontSize: 10 }}>◇</span>
            <span style={{ flex: 1, fontSize: 11 }}>
              {pg.vanishing_points.length}点透视
              {pg.horizon_y != null && ` H=${pg.horizon_y.toFixed(0)}`}
            </span>
            <button style={s.btnSm} onClick={() => handleRemove(pg.id)}>✕</button>
          </div>
        ))}
      </div>

      <div style={{ color: "#555", fontSize: 10, padding: "4px 0" }}>
        {(guides?.horizontal?.length || 0) + (guides?.vertical?.length || 0) + (guides?.perspective?.length || 0)} 条参考线
      </div>
    </div>
  );
}
