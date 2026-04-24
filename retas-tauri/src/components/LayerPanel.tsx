import { useState, useEffect, useRef, useCallback } from "react";
import { Button, Slider } from "@blueprintjs/core";
import { Eye, EyeOff, Lock, Unlock, Plus, Trash2, GripVertical, Copy } from "lucide-react";
import {
  getLayers, addLayer, deleteLayer, toggleLayerVisibility,
  toggleLayerLock, selectLayer, renameLayer, setLayerOpacity, moveLayer,
  duplicateLayer,
  LayerInfo,
} from "../api";

export default function LayerPanel() {
  const [layers, setLayers] = useState<LayerInfo[]>([
    { id: "bg-1", name: "背景", visible: true, locked: false, opacity: 1.0 },
    { id: "layer-1", name: "图层 1", visible: true, locked: false, opacity: 1.0 },
  ]);
  const [activeLayer, setActiveLayer] = useState("bg-1");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingName, setEditingName] = useState("");
  const [dragId, setDragId] = useState<string | null>(null);
  const [dropTargetId, setDropTargetId] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const loadLayers = useCallback(async () => {
    try {
      const data = await getLayers();
      if (data && data.length > 0) {
        setLayers(data);
        setActiveLayer(prev => {
          const ids = new Set(data.map(l => l.id));
          return ids.has(prev) ? prev : data[0].id;
        });
      }
    } catch (e) {
      console.error("加载图层失败:", e);
    }
  }, []);

  useEffect(() => {
    loadLayers();
  }, [loadLayers]);

  // Auto-refresh on state changes (draw stroke, undo, etc.)
  useEffect(() => {
    const handler = () => loadLayers();
    window.addEventListener("retas:state-changed", handler);
    return () => window.removeEventListener("retas:state-changed", handler);
  }, [loadLayers]);

  // Focus input when entering edit mode
  useEffect(() => {
    if (editingId && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [editingId]);

  const handleToggleVisibility = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await toggleLayerVisibility(id);
      await loadLayers();
    } catch (e) {
      console.error(e);
    }
  };

  const handleToggleLock = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await toggleLayerLock(id);
      await loadLayers();
    } catch (e) {
      console.error(e);
    }
  };

  const handleAddLayer = async () => {
    try {
      await addLayer(`图层 ${layers.length + 1}`);
      await loadLayers();
    } catch (e) {
      console.error(e);
    }
  };

  const handleDeleteLayer = async () => {
    if (layers.length <= 1) return;
    try {
      await deleteLayer(activeLayer);
      await loadLayers();
    } catch (e) {
      console.error(e);
    }
  };

  const handleDuplicateLayer = async () => {
    try {
      const newLayer = await duplicateLayer(activeLayer);
      await loadLayers();
      setActiveLayer(newLayer.id);
      selectLayer(newLayer.id);
      window.dispatchEvent(new CustomEvent("retas:state-changed"));
    } catch (e) {
      console.error("复制图层失败:", e);
    }
  };

  // --- Double-click rename ---
  const handleDoubleClick = (layer: LayerInfo) => {
    setEditingId(layer.id);
    setEditingName(layer.name);
  };

  const commitRename = async () => {
    if (!editingId) return;
    const trimmed = editingName.trim();
    if (trimmed && trimmed !== layers.find(l => l.id === editingId)?.name) {
      try {
        await renameLayer(editingId, trimmed);
        await loadLayers();
      } catch (e) {
        console.error("重命名失败:", e);
      }
    }
    setEditingId(null);
  };

  const handleRenameKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      commitRename();
    } else if (e.key === "Escape") {
      setEditingId(null);
    }
  };

  // --- Opacity slider ---
  const handleOpacityChange = async (id: string, value: number) => {
    // Optimistic update
    setLayers(prev => prev.map(l => l.id === id ? { ...l, opacity: value } : l));
    try {
      await setLayerOpacity(id, value);
      window.dispatchEvent(new CustomEvent("retas:state-changed"));
    } catch (e) {
      console.error("设置不透明度失败:", e);
      await loadLayers(); // revert on failure
    }
  };

  // --- Drag reorder ---
  const handleDragStart = (e: React.DragEvent, id: string) => {
    setDragId(id);
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", id);
  };

  const handleDragOver = (e: React.DragEvent, id: string) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = "move";
    setDropTargetId(id);
  };

  const handleDragLeave = () => {
    setDropTargetId(null);
  };

  const handleDrop = async (e: React.DragEvent, targetId: string) => {
    e.preventDefault();
    setDropTargetId(null);
    if (!dragId || dragId === targetId) {
      setDragId(null);
      return;
    }
    const targetIndex = layers.findIndex(l => l.id === targetId);
    if (targetIndex >= 0) {
      try {
        await moveLayer(dragId, targetIndex);
        await loadLayers();
      } catch (e) {
        console.error("移动图层失败:", e);
      }
    }
    setDragId(null);
  };

  const handleDragEnd = () => {
    setDragId(null);
    setDropTargetId(null);
  };

  return (
    <div>
      <div className="panel">
        <div className="panel-title">图层</div>
        <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
          {layers.map(layer => {
            const isActive = activeLayer === layer.id;
            const isEditing = editingId === layer.id;
            const isDragTarget = dropTargetId === layer.id;

            return (
              <div key={layer.id}>
                <div
                  data-testid={`layer-row-${layer.id}`}
                  draggable={!isEditing}
                  onClick={() => { setActiveLayer(layer.id); selectLayer(layer.id); }}
                  onDoubleClick={() => handleDoubleClick(layer)}
                  onDragStart={(e) => handleDragStart(e, layer.id)}
                  onDragOver={(e) => handleDragOver(e, layer.id)}
                  onDragLeave={handleDragLeave}
                  onDrop={(e) => handleDrop(e, layer.id)}
                  onDragEnd={handleDragEnd}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 4,
                    padding: "4px 8px",
                    background: isActive ? "#094771" : "transparent",
                    borderRadius: 4,
                    cursor: "pointer",
                    fontSize: 12,
                    opacity: layer.visible ? 1 : 0.5,
                    borderTop: isDragTarget ? "2px solid #48aff0" : "2px solid transparent",
                  }}
                >
                  <GripVertical
                    size={10}
                    style={{ cursor: "grab", opacity: 0.4, flexShrink: 0 }}
                  />

                  {isEditing ? (
                    <input
                      ref={inputRef}
                      value={editingName}
                      onChange={(e) => setEditingName(e.target.value)}
                      onBlur={commitRename}
                      onKeyDown={handleRenameKeyDown}
                      onClick={(e) => e.stopPropagation()}
                      style={{
                        flex: 1,
                        background: "#1a1a2e",
                        border: "1px solid #48aff0",
                        borderRadius: 2,
                        color: "inherit",
                        fontSize: 12,
                        padding: "1px 4px",
                        outline: "none",
                      }}
                    />
                  ) : (
                    <span style={{ flex: 1, userSelect: "none" }}>{layer.name}</span>
                  )}

                  <Button
                    minimal
                    small
                    data-testid={`layer-visibility-${layer.id}`}
                    icon={layer.visible ? <Eye size={12} /> : <EyeOff size={12} />}
                    onClick={(e) => handleToggleVisibility(layer.id, e)}
                  />
                  <Button
                    minimal
                    small
                    data-testid={`layer-lock-${layer.id}`}
                    icon={layer.locked ? <Lock size={12} /> : <Unlock size={12} />}
                    onClick={(e) => handleToggleLock(layer.id, e)}
                  />
                </div>

                {/* Opacity slider shown only for active layer */}
                {isActive && (
                  <div
                    style={{
                      display: "flex",
                      alignItems: "center",
                      gap: 6,
                      padding: "2px 8px 4px 28px",
                      fontSize: 11,
                    }}
                    onClick={(e) => e.stopPropagation()}
                  >
                    <span style={{ opacity: 0.6, whiteSpace: "nowrap" }}>不透明度</span>
                    <Slider
                      min={0}
                      max={1}
                      stepSize={0.01}
                      value={layer.opacity}
                      onChange={(v) => handleOpacityChange(layer.id, v)}
                      labelRenderer={false}
                    />
                    <span style={{ minWidth: 30, textAlign: "right", opacity: 0.6 }}>
                      {Math.round(layer.opacity * 100)}%
                    </span>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      <div style={{ padding: 8, display: "flex", gap: 4 }}>
        <Button minimal small data-testid="layer-add" icon={<Plus size={14} />} onClick={handleAddLayer}>新建</Button>
        <Button minimal small data-testid="layer-duplicate" icon={<Copy size={14} />} onClick={handleDuplicateLayer} title="复制图层">复制</Button>
        <Button minimal small data-testid="layer-delete" icon={<Trash2 size={14} />} onClick={handleDeleteLayer}>删除</Button>
      </div>
    </div>
  );
}
