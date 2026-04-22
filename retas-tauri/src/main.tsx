import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { WorkspaceProvider } from "./hooks/useWorkspace";
import "@blueprintjs/core/lib/css/blueprint.css";
import "@blueprintjs/icons/lib/css/blueprint-icons.css";
import "./styles.css";

const originalWarn = console.warn;
console.warn = (...args: any[]) => {
  const msg = args.join(" ");
  if (msg.includes("mixing modules") && msg.includes("ag-grid")) return;
  originalWarn.apply(console, args);
};

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <WorkspaceProvider>
      <App />
    </WorkspaceProvider>
  </React.StrictMode>
);
