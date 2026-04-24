import { useState, useEffect } from "react";
import { Button } from "@blueprintjs/core";
import { 
  Eye, EyeOff, Lock, Unlock, Plus, Trash2, 
  ChevronRight, ChevronDown, Folder, FolderOpen, Layers 
} from "lucide-react";
import { 
  getLayers, addLayer, deleteLayer, toggleLayerVisibility, 
  toggleLayerLock, selectLayer, LayerInfo 
} from "../api";

interface LayerFolder {
  id: string;
  name: string;
  isFolder: true;
  expanded: boolean;
  visible: boolean;
  locked: boolean;
  children: (LayerInfo | LayerFolder)[];
}

type LayerTreeItem = LayerInfo | LayerFolder;

function isFolder(item: LayerTreeItem): item is LayerFolder {
  return (item as LayerFolder).isFolder === true;
}

interface LayerPanelEnhancedProps {
  onLayerSelect?: (id: string) => void;
}

export default function LayerPanelEnhanced({ 
  onLayerSelect, 
}: LayerPanelEnhancedProps) {
  const [layers, setLayers] = useState<LayerTreeItem[]>([]);
  const [activeLayer, setActiveLayer] = useState<string | null>(null);
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set());

  useEffect(() => {
    loadLayers();
  }, []);

  const loadLayers = async () => {
    try {
      const data = await getLayers();
      if (data && data.length > 0) {
        const treeItems: LayerTreeItem[] = data.map(l => ({
          ...l,
          isFolder: false,
        }));
        setLayers(treeItems);
        if (!activeLayer && data.length > 0) {
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

  const handleAddFolder = () => {
    const newFolder: LayerFolder = {
      id: `folder-${Date.now()}`,
      name: `文件夹 ${expandedFolders.size + 1}`,
      isFolder: true,
      expanded: true,
      visible: true,
      locked: false,
      children: [],
    };
    setLayers(prev => [...prev, newFolder]);
    setExpandedFolders(prev => new Set(prev).add(newFolder.id));
  };

  const handleDeleteLayer = async () => {
    if (!activeLayer || layers.length <= 1) return;
    
    const folder = layers.find(l => isFolder(l) && l.id === activeLayer);
    if (folder && isFolder(folder) && folder.children.length > 0) {
      return;
    }
    
    try {
      if (!isFolder(layers.find(l => l.id === activeLayer)!)) {
        await deleteLayer(activeLayer);
      }
      await loadLayers();
    } catch (e) {
      console.error(e);
    }
  };

  const toggleFolder = (folderId: string) => {
    setExpandedFolders(prev => {
      const next = new Set(prev);
      if (next.has(folderId)) {
        next.delete(folderId);
      } else {
        next.add(folderId);
      }
      return next;
    });
  };

  const handleSelect = (id: string) => {
    setActiveLayer(id);
    selectLayer(id);
    onLayerSelect?.(id);
  };

  const renderLayerItem = (item: LayerTreeItem, depth: number = 0) => {
    const folder = isFolder(item);
    const isExpanded = expandedFolders.has(item.id);
    const isActive = activeLayer === item.id;
    const indent = depth * 16;

    return (
      <div key={item.id}>
        <div
          data-testid={`layer-row-${item.id}`}
          onClick={() => !folder && handleSelect(item.id)}
          style={{
            display: "flex",
            alignItems: "center",
            gap: 4,
            padding: "4px 8px",
            paddingLeft: 8 + indent,
            background: isActive ? "#094771" : "transparent",
            borderRadius: 4,
            cursor: folder ? "pointer" : "default",
            fontSize: 12,
            opacity: item.visible ? 1 : 0.5,
          }}
        >
          {folder && (
            <span 
              onClick={() => toggleFolder(item.id)}
              style={{ cursor: "pointer", display: "flex" }}
            >
              {isExpanded ? (
                <ChevronDown size={12} />
              ) : (
                <ChevronRight size={12} />
              )}
            </span>
          )}
          
          <span style={{ display: "flex", marginRight: 4 }}>
            {folder ? (
              isExpanded ? <FolderOpen size={12} /> : <Folder size={12} />
            ) : (
              <Layers size={12} />
            )}
          </span>
          
          <span style={{ flex: 1 }}>{item.name}</span>
          
          <Button
            minimal
            small
            data-testid={`layer-visibility-${item.id}`}
            icon={item.visible ? <Eye size={12} /> : <EyeOff size={12} />}
            onClick={(e) => handleToggleVisibility(item.id, e)}
          />
          <Button
            minimal
            small
            data-testid={`layer-lock-${item.id}`}
            icon={item.locked ? <Lock size={12} /> : <Unlock size={12} />}
            onClick={(e) => handleToggleLock(item.id, e)}
          />
        </div>

        {folder && isExpanded && (
          <div>
            {(item as LayerFolder).children.map(child => 
              renderLayerItem(child, depth + 1)
            )}
          </div>
        )}
      </div>
    );
  };

  return (
    <div>
      <div className="panel">
        <div className="panel-title">
          图层
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 2 }}>
          {layers.map(layer => renderLayerItem(layer))}
        </div>
      </div>

      <div style={{ padding: 8, display: "flex", gap: 4, flexWrap: "wrap" }}>
        <Button 
          minimal 
          small 
          data-testid="layer-add" 
          icon={<Plus size={14} />} 
          onClick={handleAddLayer}
        >
          新建图层
        </Button>
        <Button 
          minimal 
          small 
          data-testid="folder-add" 
          icon={<Folder size={14} />} 
          onClick={handleAddFolder}
        >
          新建文件夹
        </Button>
        <Button 
          minimal 
          small 
          data-testid="layer-delete" 
          icon={<Trash2 size={14} />} 
          onClick={handleDeleteLayer}
        >
          删除
        </Button>
      </div>
    </div>
  );
}
