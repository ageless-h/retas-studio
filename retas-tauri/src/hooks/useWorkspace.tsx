import { createContext, useContext, useState, ReactNode } from "react";
import { Pencil, Film, Palette, Layers } from "lucide-react";

type WorkspaceType = "drawing" | "animation" | "coloring" | "compositing";

interface WorkspaceConfig {
  id: WorkspaceType;
  label: string;
  icon: ReactNode;
  description: string;
}

const WORKSPACES: WorkspaceConfig[] = [
  { id: "drawing", label: "绘画", icon: <Pencil size={14} />, description: "绘图与编辑" },
  { id: "animation", label: "动画", icon: <Film size={14} />, description: "时间轴与帧编辑" },
  { id: "coloring", label: "上色", icon: <Palette size={14} />, description: "颜色与填充" },
  { id: "compositing", label: "合成", icon: <Layers size={14} />, description: "图层与特效" },
];

interface WorkspaceContextType {
  currentWorkspace: WorkspaceType;
  setWorkspace: (ws: WorkspaceType) => void;
  workspaces: WorkspaceConfig[];
}

const WorkspaceContext = createContext<WorkspaceContextType>({
  currentWorkspace: "drawing",
  setWorkspace: () => {},
  workspaces: WORKSPACES,
});

export function WorkspaceProvider({ children }: { children: ReactNode }) {
  const [currentWorkspace, setWorkspace] = useState<WorkspaceType>("drawing");

  return (
    <WorkspaceContext.Provider value={{ currentWorkspace, setWorkspace, workspaces: WORKSPACES }}>
      {children}
    </WorkspaceContext.Provider>
  );
}

export function useWorkspace() {
  return useContext(WorkspaceContext);
}
