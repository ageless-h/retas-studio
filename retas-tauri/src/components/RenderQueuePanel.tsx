import { useState, useEffect, useCallback } from "react";
import { Button, ProgressBar, HTMLSelect } from "@blueprintjs/core";
import { Plus, Trash2, X } from "lucide-react";
import { addRenderJob, getRenderJobs, cancelRenderJob, clearCompletedJobs, RenderJobInfo } from "../api";

export default function RenderQueuePanel() {
  const [jobs, setJobs] = useState<RenderJobInfo[]>([]);
  const [startFrame, setStartFrame] = useState(1);
  const [endFrame, setEndFrame] = useState(100);
  const [format, setFormat] = useState("png");
  const [quality, setQuality] = useState("standard");

  const loadJobs = useCallback(async () => {
    try {
      const data = await getRenderJobs();
      setJobs(data);
    } catch (e) {
      console.error("加载渲染队列失败:", e);
    }
  }, []);

  useEffect(() => {
    loadJobs();
    const interval = setInterval(loadJobs, 2000);
    return () => clearInterval(interval);
  }, [loadJobs]);

  const handleAddJob = async () => {
    try {
      await addRenderJob(
        `导出 ${format.toUpperCase()} ${startFrame}-${endFrame}`,
        startFrame - 1, endFrame - 1,
        "./output", format, quality
      );
      await loadJobs();
    } catch (e) {
      console.error("添加渲染任务失败:", e);
    }
  };

  const handleCancel = async (jobId: number) => {
    try {
      await cancelRenderJob(jobId);
      await loadJobs();
    } catch (e) {
      console.error("取消任务失败:", e);
    }
  };

  const handleClearCompleted = async () => {
    try {
      await clearCompletedJobs();
      await loadJobs();
    } catch (e) {
      console.error("清除已完成任务失败:", e);
    }
  };

  const statusColor = (status: string) => {
    if (status === "queued") return "#8b949e";
    if (status === "rendering") return "#f0883e";
    if (status === "completed") return "#3fb950";
    if (status.startsWith("failed")) return "#f85149";
    return "#484f58";
  };

  const statusLabel = (status: string) => {
    if (status === "queued") return "排队中";
    if (status === "rendering") return "渲染中";
    if (status === "completed") return "已完成";
    if (status === "cancelled") return "已取消";
    if (status.startsWith("failed")) return "失败";
    return status;
  };

  return (
    <div className="panel">
      <div className="panel-title">渲染队列</div>

      <div style={{ padding: "8px", display: "flex", flexDirection: "column", gap: 6 }}>
        <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
          <label style={{ fontSize: 11, color: "#8b949e", whiteSpace: "nowrap" }}>帧范围</label>
          <input
            type="number" min={1} value={startFrame}
            onChange={e => setStartFrame(Number(e.target.value))}
            style={{ width: 50, background: "#0d1117", border: "1px solid #30363d", borderRadius: 3, color: "#c9d1d9", padding: "2px 4px", fontSize: 11 }}
          />
          <span style={{ color: "#8b949e", fontSize: 11 }}>-</span>
          <input
            type="number" min={1} value={endFrame}
            onChange={e => setEndFrame(Number(e.target.value))}
            style={{ width: 50, background: "#0d1117", border: "1px solid #30363d", borderRadius: 3, color: "#c9d1d9", padding: "2px 4px", fontSize: 11 }}
          />
        </div>
        <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
          <HTMLSelect
            minimal small value={format}
            onChange={e => setFormat(e.target.value)}
            options={[
              { value: "png", label: "PNG" },
              { value: "jpeg", label: "JPEG" },
              { value: "gif", label: "GIF" },
              { value: "mp4", label: "MP4" },
              { value: "webm", label: "WebM" },
            ]}
            style={{ fontSize: 11 }}
          />
          <HTMLSelect
            minimal small value={quality}
            onChange={e => setQuality(e.target.value)}
            options={[
              { value: "draft", label: "草稿" },
              { value: "standard", label: "标准" },
              { value: "high", label: "高质量" },
              { value: "maximum", label: "最高" },
            ]}
            style={{ fontSize: 11 }}
          />
          <Button small icon={<Plus size={12} />} onClick={handleAddJob} intent="primary">
            添加
          </Button>
        </div>
      </div>

      <div style={{ borderTop: "1px solid #21262d", maxHeight: 300, overflowY: "auto" }}>
        {jobs.length === 0 && (
          <div style={{ padding: 16, textAlign: "center", color: "#484f58", fontSize: 12 }}>
            暂无渲染任务
          </div>
        )}
        {jobs.map(job => (
          <div key={job.id} style={{
            padding: "6px 8px", borderBottom: "1px solid #21262d",
            display: "flex", flexDirection: "column", gap: 3,
          }}>
            <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <span style={{ flex: 1, fontSize: 11, color: "#c9d1d9" }}>{job.name}</span>
              <span style={{ fontSize: 10, color: statusColor(job.status), fontWeight: 600 }}>
                {statusLabel(job.status)}
              </span>
              {job.status === "queued" && (
                <Button minimal small icon={<X size={10} />} onClick={() => handleCancel(job.id)} />
              )}
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <span style={{ fontSize: 10, color: "#484f58" }}>
                帧 {job.frame_range[0] + 1}-{job.frame_range[1] + 1} | {job.format}
              </span>
              <div style={{ flex: 1 }}>
                <ProgressBar
                  value={job.progress / 100}
                  intent={job.status === "completed" ? "success" : job.status.startsWith("failed") ? "danger" : "primary"}
                  stripes={job.status === "rendering"}
                  animate={job.status === "rendering"}
                />
              </div>
              <span style={{ fontSize: 10, color: "#8b949e", minWidth: 30, textAlign: "right" }}>
                {Math.round(job.progress)}%
              </span>
            </div>
          </div>
        ))}
      </div>

      {jobs.some(j => j.status === "completed" || j.status === "cancelled") && (
        <div style={{ padding: "6px 8px", borderTop: "1px solid #21262d" }}>
          <Button minimal small icon={<Trash2 size={12} />} onClick={handleClearCompleted}>
            清除已完成
          </Button>
        </div>
      )}
    </div>
  );
}
