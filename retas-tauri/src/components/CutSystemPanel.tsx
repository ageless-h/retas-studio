import { useState, useEffect, useCallback } from "react";
import {
  createCutFolder, deleteCutFolder, getCutFolders,
  addCut, removeCut, setCurrentCutFolder, findCutAtFrame,
  CutFolderInfo, CutInfo,
} from "../api";

interface Props {
  currentFrame?: number;
}

export default function CutSystemPanel({ currentFrame = 0 }: Props) {
  const [folders, setFolders] = useState<CutFolderInfo[]>([]);
  const [activeFolderId, setActiveFolderId] = useState<number | null>(null);
  const [newFolderName, setNewFolderName] = useState("");
  const [newCutName, setNewCutName] = useState("");
  const [cutStart, setCutStart] = useState(0);
  const [cutEnd, setCutEnd] = useState(24);
  const [currentCut, setCurrentCut] = useState<{ folder: CutFolderInfo; cut: CutInfo } | null>(null);

  const refresh = useCallback(async () => {
    const data = await getCutFolders();
    setFolders(data);
    const found = await findCutAtFrame(currentFrame);
    if (found) {
      setCurrentCut({ folder: found[0], cut: found[1] });
    } else {
      setCurrentCut(null);
    }
  }, [currentFrame]);

  useEffect(() => { refresh(); }, [refresh]);

  const handleCreateFolder = async () => {
    if (!newFolderName.trim()) return;
    const id = await createCutFolder(newFolderName.trim());
    setNewFolderName("");
    setActiveFolderId(id);
    await setCurrentCutFolder(id);
    await refresh();
  };

  const handleDeleteFolder = async (id: number) => {
    await deleteCutFolder(id);
    if (activeFolderId === id) setActiveFolderId(null);
    await refresh();
  };

  const handleAddCut = async () => {
    if (!activeFolderId || !newCutName.trim()) return;
    await addCut(activeFolderId, newCutName.trim(), cutStart, cutEnd);
    setNewCutName("");
    await refresh();
  };

  const handleRemoveCut = async (folderId: number, cutId: number) => {
    await removeCut(folderId, cutId);
    await refresh();
  };

  const handleSelectFolder = async (id: number) => {
    setActiveFolderId(id);
    await setCurrentCutFolder(id);
  };

  const activeFolder = folders.find(f => f.id === activeFolderId);

  const s = {
    panel: { padding: 8, fontSize: 12, color: "#ccc", height: "100%", overflow: "auto" as const, background: "#1e1e1e" },
    section: { marginBottom: 10, borderBottom: "1px solid #333", paddingBottom: 8 },
    label: { color: "#888", fontSize: 10, textTransform: "uppercase" as const, marginBottom: 4 },
    row: { display: "flex", alignItems: "center" as const, gap: 6, marginBottom: 4 },
    input: { background: "#2a2a2a", border: "1px solid #444", color: "#eee", padding: "2px 4px", width: 48, borderRadius: 3, fontSize: 11 },
    inputWide: { background: "#2a2a2a", border: "1px solid #444", color: "#eee", padding: "2px 4px", flex: 1, borderRadius: 3, fontSize: 11 },
    btn: { background: "#3a3a3a", border: "1px solid #555", color: "#eee", padding: "3px 8px", borderRadius: 3, cursor: "pointer", fontSize: 11 },
    btnSm: { background: "none", border: "none", color: "#888", cursor: "pointer", fontSize: 11, padding: "0 4px" },
    folderTab: (active: boolean) => ({
      padding: "3px 8px", borderRadius: 3, cursor: "pointer", fontSize: 11,
      background: active ? "#3a3a5a" : "#2a2a2a", border: active ? "1px solid #66a" : "1px solid #444", color: "#eee",
    }),
    cutItem: { display: "flex", alignItems: "center" as const, gap: 4, padding: "4px 0", borderBottom: "1px solid #2a2a2a" },
    cutColor: (c: number[]) => ({
      width: 8, height: 8, borderRadius: 2,
      background: `rgba(${c[0]},${c[1]},${c[2]},${c[3] / 255})`,
    }),
  };

  return (
    <div style={s.panel}>
      {currentCut && (
        <div style={{ ...s.section, background: "#1a2a1a", padding: 6, borderRadius: 4, marginBottom: 8 }}>
          <div style={{ fontSize: 10, color: "#6a6" }}>当前帧 {currentFrame}</div>
          <div style={{ fontSize: 11, color: "#ada" }}>
            {currentCut.folder.name} / {currentCut.cut.name}
          </div>
          <div style={{ fontSize: 10, color: "#686" }}>
            帧 {currentCut.cut.start_frame}-{currentCut.cut.end_frame} ({currentCut.cut.duration_frames}帧)
          </div>
        </div>
      )}

      <div style={s.section}>
        <div style={s.label}>卡文件夹 (Cut Folders)</div>
        <div style={s.row}>
          <input type="text" placeholder="文件夹名..." value={newFolderName}
            onChange={e => setNewFolderName(e.target.value)} style={s.inputWide}
            onKeyDown={e => e.key === "Enter" && handleCreateFolder()} />
          <button style={s.btn} onClick={handleCreateFolder}>+</button>
        </div>
        <div style={{ display: "flex", flexWrap: "wrap" as const, gap: 4 }}>
          {folders.map(f => (
            <div key={f.id} style={{ display: "flex", alignItems: "center", gap: 2 }}>
              <button style={s.folderTab(f.id === activeFolderId)} onClick={() => handleSelectFolder(f.id)}>
                {f.name} ({f.cuts.length})
              </button>
              <button style={{ ...s.btnSm, color: "#f55", fontSize: 9 }}
                onClick={() => handleDeleteFolder(f.id)} title="删除">✕</button>
            </div>
          ))}
          {folders.length === 0 && <span style={{ color: "#555", fontSize: 11 }}>无文件夹</span>}
        </div>
      </div>

      {activeFolder && (
        <div style={s.section}>
          <div style={s.label}>{activeFolder.name} - 卡 (Cuts)</div>
          <div style={s.row}>
            <input type="text" placeholder="卡名..." value={newCutName}
              onChange={e => setNewCutName(e.target.value)} style={s.inputWide}
              onKeyDown={e => e.key === "Enter" && handleAddCut()} />
          </div>
          <div style={s.row}>
            <span style={{ width: 24 }}>开始</span>
            <input type="number" value={cutStart} min={0}
              onChange={e => setCutStart(Number(e.target.value))} style={s.input} />
            <span style={{ width: 24 }}>结束</span>
            <input type="number" value={cutEnd} min={0}
              onChange={e => setCutEnd(Number(e.target.value))} style={s.input} />
            <button style={s.btn} onClick={handleAddCut}>添加卡</button>
          </div>
          {activeFolder.cuts.map(cut => (
            <div key={cut.id} style={s.cutItem}>
              <span style={s.cutColor(cut.color)} />
              <span style={{ flex: 1, fontSize: 11 }}>{cut.name}</span>
              <span style={{ fontSize: 10, color: "#888" }}>
                {cut.start_frame}-{cut.end_frame} ({cut.duration_frames}帧)
              </span>
              <button style={{ ...s.btnSm, color: "#f55" }}
                onClick={() => handleRemoveCut(activeFolder.id, cut.id)}>✕</button>
            </div>
          ))}
          {activeFolder.cuts.length === 0 && (
            <div style={{ color: "#555", fontSize: 11, padding: 4 }}>无卡</div>
          )}
        </div>
      )}
    </div>
  );
}
