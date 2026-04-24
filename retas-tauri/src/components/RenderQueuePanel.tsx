import { useState, useEffect, useCallback } from "react";
import { Button, ProgressBar, HTMLSelect } from "@blueprintjs/core";
import { Plus, Trash2, X, Layers } from "lucide-react";
import {
  addRenderJob, getRenderJobs, cancelRenderJob, clearCompletedJobs, RenderJobInfo,
  addBatchExport, addBatchColorReplace, getBatchQueue, cancelBatchItem,
  clearBatchCompleted, getBatchStats, BatchItemInfo,
} from "../api";

export default function RenderQueuePanel() {
  const [jobs, setJobs] = useState<RenderJobInfo[]>([]);
  const [startFrame, setStartFrame] = useState(1);
  const [endFrame, setEndFrame] = useState(100);
  const [format, setFormat] = useState("png");
  const [quality, setQuality] = useState("standard");

  // Batch state
  const [batchItems, setBatchItems] = useState<BatchItemInfo[]>([]);
  const [batchStats, setBatchStats] = useState<[number, number, number, number]>([0, 0, 0, 0]);
  const [batchFormat, setBatchFormat] = useState("png");
  const [batchDir, setBatchDir] = useState("./batch_output");
  const [batchStart, setBatchStart] = useState(1);
  const [batchEnd, setBatchEnd] = useState(100);
  const [batchPriority, setBatchPriority] = useState("Normal");
  const [showBatch, setShowBatch] = useState(false);

  const loadJobs = useCallback(async () => {
    try {
      const data = await getRenderJobs();
      setJobs(data);
    } catch (e) {
      console.error("加载渲染队列失败:", e);
    }
  }, []);

  const loadBatch = useCallback(async () => {
    try {
      const [items, stats] = await Promise.all([getBatchQueue(), getBatchStats()]);
      setBatchItems(items);
      setBatchStats(stats);
    } catch (e) {
      console.error("加载批处理队列失败:", e);
    }
  }, []);

  useEffect(() => {
    loadJobs();
    loadBatch();
    const interval = setInterval(() => { loadJobs(); loadBatch(); }, 2000);
    return () => clearInterval(interval);
  }, [loadJobs, loadBatch]);

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

  const handleAddBatchExport = async () => {
    await addBatchExport(batchDir, batchFormat, batchStart - 1, batchEnd - 1, batchPriority);
    await loadBatch();
  };

  const handleCancelBatch = async (id: number) => {
    await cancelBatchItem(id);
    await loadBatch();
  };

  const handleClearBatch = async () => {
    await clearBatchCompleted();
    await loadBatch();
  };

  const statusColor = (status: string) => {
    if (status === "queued" || status === "Pending") return "#8b949e";
    if (status === "rendering" || status === "Running") return "#f0883e";
    if (status === "completed" || status === "Completed") return "#3fb950";
    if (status.startsWith("failed") || status === "Failed") return "#f85149";
    return "#484f58";
  };

  const statusLabel = (status: string) => {
    if (status === "queued" || status === "Pending") return "排队中";
    if (status === "rendering" || status === "Running") return "执行中";
    if (status === "completed" || status === "Completed") return "已完成";
    if (status === "cancelled" || status === "Cancelled") return "已取消";
    if (status.startsWith("failed") || status === "Failed") return "失败";
    return status;
  };

  const sInput = { width: 50, background: "#0d1117", border: "1px solid #30363d", borderRadius: 3, color: "#c9d1d9", padding: "2px 4px", fontSize: 11 };
  const sInputWide = { ...sInput, width: "auto", flex: 1 };

  return (
    <div className="panel">
      <div className="panel-title" style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <span>渲染队列</span>
        <button
          onClick={() => setShowBatch(!showBatch)}
          style={{ background: "none", border: "none", color: showBatch ? "#58a6ff" : "#8b949e", cursor: "pointer", fontSize: 11, display: "flex", alignItems: "center", gap: 3 }}
          title="批处理"
        >
          <Layers size={12} /> 批处理
        </button>
      </div>

      {!showBatch ? (
        <>
          <div style={{ padding: "8px", display: "flex", flexDirection: "column", gap: 6 }}>
            <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
              <label style={{ fontSize: 11, color: "#8b949e", whiteSpace: "nowrap" }}>帧范围</label>
              <input type="number" min={1} value={startFrame} onChange={e => setStartFrame(Number(e.target.value))} style={sInput} />
              <span style={{ color: "#8b949e", fontSize: 11 }}>-</span>
              <input type="number" min={1} value={endFrame} onChange={e => setEndFrame(Number(e.target.value))} style={sInput} />
            </div>
            <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
              <HTMLSelect minimal small value={format} onChange={e => setFormat(e.target.value)}
                options={[
                  { value: "png", label: "PNG" }, { value: "jpeg", label: "JPEG" },
                  { value: "gif", label: "GIF" }, { value: "mp4", label: "MP4" }, { value: "webm", label: "WebM" },
                ]} style={{ fontSize: 11 }} />
              <HTMLSelect minimal small value={quality} onChange={e => setQuality(e.target.value)}
                options={[
                  { value: "draft", label: "草稿" }, { value: "standard", label: "标准" },
                  { value: "high", label: "高质量" }, { value: "maximum", label: "最高" },
                ]} style={{ fontSize: 11 }} />
              <Button small icon={<Plus size={12} />} onClick={handleAddJob} intent="primary">添加</Button>
            </div>
          </div>

          <div style={{ borderTop: "1px solid #21262d", maxHeight: 300, overflowY: "auto" }}>
            {jobs.length === 0 && (
              <div style={{ padding: 16, textAlign: "center", color: "#484f58", fontSize: 12 }}>暂无渲染任务</div>
            )}
            {jobs.map(job => (
              <div key={job.id} style={{ padding: "6px 8px", borderBottom: "1px solid #21262d", display: "flex", flexDirection: "column", gap: 3 }}>
                <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                  <span style={{ flex: 1, fontSize: 11, color: "#c9d1d9" }}>{job.name}</span>
                  <span style={{ fontSize: 10, color: statusColor(job.status), fontWeight: 600 }}>{statusLabel(job.status)}</span>
                  {job.status === "queued" && <Button minimal small icon={<X size={10} />} onClick={() => handleCancel(job.id)} />}
                </div>
                <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                  <span style={{ fontSize: 10, color: "#484f58" }}>帧 {job.frame_range[0] + 1}-{job.frame_range[1] + 1} | {job.format}</span>
                  <div style={{ flex: 1 }}>
                    <ProgressBar value={job.progress / 100}
                      intent={job.status === "completed" ? "success" : job.status.startsWith("failed") ? "danger" : "primary"}
                      stripes={job.status === "rendering"} animate={job.status === "rendering"} />
                  </div>
                  <span style={{ fontSize: 10, color: "#8b949e", minWidth: 30, textAlign: "right" }}>{Math.round(job.progress)}%</span>
                </div>
              </div>
            ))}
          </div>

          {jobs.some(j => j.status === "completed" || j.status === "cancelled") && (
            <div style={{ padding: "6px 8px", borderTop: "1px solid #21262d" }}>
              <Button minimal small icon={<Trash2 size={12} />} onClick={handleClearCompleted}>清除已完成</Button>
            </div>
          )}
        </>
      ) : (
        <>
          <div style={{ padding: "8px", display: "flex", flexDirection: "column", gap: 6 }}>
            <div style={{ fontSize: 10, color: "#8b949e", textTransform: "uppercase", marginBottom: 2 }}>批量导出</div>
            <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
              <label style={{ fontSize: 11, color: "#8b949e", whiteSpace: "nowrap" }}>目录</label>
              <input type="text" value={batchDir} onChange={e => setBatchDir(e.target.value)} style={sInputWide} />
            </div>
            <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
              <input type="number" min={1} value={batchStart} onChange={e => setBatchStart(Number(e.target.value))} style={sInput} />
              <span style={{ color: "#8b949e", fontSize: 11 }}>-</span>
              <input type="number" min={1} value={batchEnd} onChange={e => setBatchEnd(Number(e.target.value))} style={sInput} />
              <HTMLSelect minimal small value={batchFormat} onChange={e => setBatchFormat(e.target.value)}
                options={[{ value: "png", label: "PNG" }, { value: "jpeg", label: "JPEG" }, { value: "tga", label: "TGA" }]}
                style={{ fontSize: 11 }} />
            </div>
            <div style={{ display: "flex", gap: 6, alignItems: "center", justifyContent: "flex-end" }}>
              <HTMLSelect minimal small value={batchPriority} onChange={e => setBatchPriority(e.target.value)}
                options={[{ value: "Low", label: "低" }, { value: "Normal", label: "普通" }, { value: "High", label: "高" }, { value: "Urgent", label: "紧急" }]}
                style={{ fontSize: 11 }} />
              <Button small icon={<Plus size={12} />} onClick={handleAddBatchExport} intent="primary">添加</Button>
            </div>
          </div>

          <div style={{ padding: "4px 8px", display: "flex", gap: 8, fontSize: 10, color: "#8b949e", borderTop: "1px solid #21262d" }}>
            <span>等待: {batchStats[0]}</span>
            <span style={{ color: "#f0883e" }}>执行: {batchStats[1]}</span>
            <span style={{ color: "#3fb950" }}>完成: {batchStats[2]}</span>
            <span style={{ color: "#f85149" }}>失败: {batchStats[3]}</span>
          </div>

          <div style={{ maxHeight: 250, overflowY: "auto" }}>
            {batchItems.length === 0 && (
              <div style={{ padding: 16, textAlign: "center", color: "#484f58", fontSize: 12 }}>暂无批处理任务</div>
            )}
            {batchItems.map(item => (
              <div key={item.id} style={{ padding: "4px 8px", borderBottom: "1px solid #21262d", display: "flex", alignItems: "center", gap: 6 }}>
                <span style={{ fontSize: 10, color: "#c9d1d9", flex: 1 }}>#{item.id} {item.operation}</span>
                <span style={{ fontSize: 10, color: statusColor(item.status), fontWeight: 600 }}>{statusLabel(item.status)}</span>
                <span style={{ fontSize: 9, color: "#484f58" }}>{item.priority}</span>
                {(item.status === "Pending") && (
                  <button onClick={() => handleCancelBatch(item.id)}
                    style={{ background: "none", border: "none", color: "#f85149", cursor: "pointer", fontSize: 10 }}>✕</button>
                )}
              </div>
            ))}
          </div>

          {batchItems.some(i => i.status === "Completed" || i.status === "Cancelled") && (
            <div style={{ padding: "6px 8px", borderTop: "1px solid #21262d" }}>
              <Button minimal small icon={<Trash2 size={12} />} onClick={handleClearBatch}>清除已完成</Button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
