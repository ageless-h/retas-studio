import { useState, useEffect } from "react";
import { Button } from "@blueprintjs/core";
import { Eye, EyeOff, Lock, Unlock, Plus, Trash2 } from "lucide-react";
import { getLayers, addLayer, deleteLayer, toggleLayerVisibility, toggleLayerLock, selectLayer, LayerInfo } from "../api";

export default function LayerPanel() {
  const [layers, setLayers] = useState<LayerInfo[]>([
    { id: "bg-1", name: "背景", visible: true, locked: false, opacity: 1.0 },
    { id: "layer-1", name: "图层 1", visible: true, locked: false, opacity: 1.0 },
  ]);
  const [activeLayer, setActiveLayer] = useState("bg-1");

  useEffect(() => {
    loadLayers();
  }, []);

  const loadLayers = async () => {
    try {
      const data = await getLayers();
      if (data && data.length > 0) {
        setLayers(data);
        const ids = new Set(data.map(l => l.id));
        if (!ids.has(activeLayer)) {
          setActiveLayer(data[0].id);
        }
      }
    } catch (e) {
      console.error("加载图层失败:", e);
    }
  };

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

  return (
    <div>
      <div className="panel">
        <div className="panel-title">图层</div>
        <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
          {layers.map(layer => (
            <div
              key={layer.id}
              data-testid={`layer-row-${layer.id}`}
              onClick={() => { setActiveLayer(layer.id); selectLayer(layer.id); }}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 4,
                padding: "4px 8px",
                background: activeLayer === layer.id ? "#094771" : "transparent",
                borderRadius: 4,
                cursor: "pointer",
                fontSize: 12,
              }}
            >
              <span style={{ flex: 1 }}>{layer.name}</span>
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
          ))}
        </div>
      </div>

      <div style={{ padding: 8, display: "flex", gap: 4 }}>
        <Button minimal small data-testid="layer-add" icon={<Plus size={14} />} onClick={handleAddLayer}>新建</Button>
        <Button minimal small data-testid="layer-delete" icon={<Trash2 size={14} />} onClick={handleDeleteLayer}>删除</Button>
      </div>
    </div>
  );
}
