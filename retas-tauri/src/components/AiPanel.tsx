import { useState, useCallback } from "react";
import { Button, Tag } from "@blueprintjs/core";
import { Wand2, Palette, Image, Loader2 } from "lucide-react";
import { aiAutoColor, aiStyleTransfer, aiQueueStatus, AiQueueStatus } from "../api";

export default function AiPanel() {
  const [isProcessing, setIsProcessing] = useState(false);
  const [queueStatus, setQueueStatus] = useState<AiQueueStatus | null>(null);
  const [lastResult, setLastResult] = useState<string | null>(null);

  const refreshQueueStatus = useCallback(async () => {
    try {
      const status = await aiQueueStatus();
      setQueueStatus(status);
    } catch (e) {
      console.error("Failed to get queue status:", e);
    }
  }, []);

  const handleAutoColor = async () => {
    setIsProcessing(true);
    setLastResult("自动上色请求已发送...");
    try {
      const dummyImage = new Uint8Array(512 * 512 * 4);
      const result = await aiAutoColor(dummyImage);
      if (result.success) {
        setLastResult("自动上色完成");
      } else {
        setLastResult("自动上色失败: " + (result.error || "未知错误"));
      }
      await refreshQueueStatus();
    } catch (e) {
      setLastResult("错误: " + String(e));
    } finally {
      setIsProcessing(false);
    }
  };

  const handleStyleTransfer = async () => {
    setIsProcessing(true);
    setLastResult("风格迁移请求已发送...");
    try {
      const dummyImage = new Uint8Array(512 * 512 * 4);
      const dummyStyle = new Uint8Array(512 * 512 * 4);
      const result = await aiStyleTransfer(dummyImage, dummyStyle);
      if (result.success) {
        setLastResult("风格迁移完成");
      } else {
        setLastResult("风格迁移失败: " + (result.error || "未知错误"));
      }
      await refreshQueueStatus();
    } catch (e) {
      setLastResult("错误: " + String(e));
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div style={{ padding: 12 }}>
      <div style={{ 
        fontSize: 13, 
        fontWeight: 600, 
        marginBottom: 12,
        color: "#e0e0e0",
        display: "flex",
        alignItems: "center",
        gap: 6
      }}>
        <Wand2 size={14} />
        AI 功能
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 8 }}>
        <Button
          fill
          intent="primary"
          icon={<Palette size={14} />}
          onClick={handleAutoColor}
          disabled={isProcessing}
        >
          {isProcessing ? <Loader2 size={14} className="spin" /> : "自动上色"}
        </Button>

        <Button
          fill
          intent="warning"
          icon={<Image size={14} />}
          onClick={handleStyleTransfer}
          disabled={isProcessing}
        >
          {isProcessing ? <Loader2 size={14} className="spin" /> : "风格迁移"}
        </Button>
      </div>

      {queueStatus && (
        <div style={{ marginTop: 12 }}>
          <div style={{ 
            fontSize: 11, 
            color: "#8b949e",
            marginBottom: 4
          }}>
            队列状态
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
            <Tag minimal intent={queueStatus.pending > 0 ? "warning" : "success"}>
              {queueStatus.pending} / {queueStatus.maxSize}
            </Tag>
            {queueStatus.isFull && (
              <Tag minimal intent="danger">队列已满</Tag>
            )}
          </div>
        </div>
      )}

      {lastResult && (
        <div style={{ 
          marginTop: 12,
          padding: 8,
          background: "#21262d",
          borderRadius: 4,
          fontSize: 11,
          color: "#8b949e",
          wordBreak: "break-all"
        }}>
          {lastResult}
        </div>
      )}

      <div style={{ marginTop: 12 }}>
        <Button
          minimal
          small
          fill
          onClick={refreshQueueStatus}
        >
          刷新状态
        </Button>
      </div>
    </div>
  );
}
