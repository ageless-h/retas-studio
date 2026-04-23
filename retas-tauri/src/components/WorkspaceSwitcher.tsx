import { Button, ButtonGroup } from "@blueprintjs/core";
import { useWorkspace } from "../hooks/useWorkspace";

export default function WorkspaceSwitcher() {
  const { currentWorkspace, setWorkspace, workspaces } = useWorkspace();

  return (
    <div style={{
      display: "flex",
      alignItems: "center",
      gap: 2,
      padding: "0 12px",
      background: "#161b22",
      borderBottom: "1px solid #30363d",
      height: 32,
    }}>
      <span style={{ fontSize: 10, color: "#6e7681", marginRight: 8, textTransform: "uppercase", letterSpacing: 1 }}>
        工作区
      </span>
      <ButtonGroup minimal style={{ gap: 2 }}>
        {workspaces.map(ws => (
          <Button
            key={ws.id}
            small
            minimal
            data-testid={`workspace-${ws.id}`}
            active={currentWorkspace === ws.id}
            onClick={() => setWorkspace(ws.id)}
            style={{
              fontSize: 12,
              padding: "4px 12px",
              borderRadius: 4,
              background: currentWorkspace === ws.id ? "#1f6feb22" : "transparent",
              color: currentWorkspace === ws.id ? "#58a6ff" : "#8b949e",
              border: currentWorkspace === ws.id ? "1px solid #1f6feb44" : "1px solid transparent",
              transition: "all 0.15s ease",
            }}
          >
            <span style={{ marginRight: 4 }}>{ws.icon}</span>
            {ws.label}
          </Button>
        ))}
      </ButtonGroup>
      <div style={{ flex: 1 }} />
      <span style={{ fontSize: 11, color: "#6e7681" }}>
        {workspaces.find(w => w.id === currentWorkspace)?.description}
      </span>
    </div>
  );
}
