import { useState } from "react";
import { Button, Dialog, FormGroup, HTMLSelect, Intent } from "@blueprintjs/core";
import { Download, Film } from "lucide-react";

interface ExportDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function ExportDialog({ isOpen, onClose }: ExportDialogProps) {
  const [format, setFormat] = useState("png");
  const [quality, setQuality] = useState("high");
  const [isExporting, setIsExporting] = useState(false);

  const handleExport = async () => {
    setIsExporting(true);
    await new Promise(resolve => setTimeout(resolve, 2000));
    setIsExporting(false);
    onClose();
  };

  return (
    <Dialog
      isOpen={isOpen}
      onClose={onClose}
      title="导出"
      style={{ width: 400 }}
    >
      <div style={{ padding: 16 }}>
        <FormGroup label="格式">
          <HTMLSelect
            value={format}
            onChange={(e) => setFormat(e.target.value)}
            options={[
              { value: "png", label: "PNG 序列帧" },
              { value: "jpeg", label: "JPEG 序列帧" },
              { value: "mp4", label: "MP4 视频" },
              { value: "gif", label: "GIF 动画" },
              { value: "webm", label: "WebM 视频" },
            ]}
          />
        </FormGroup>

        <FormGroup label="质量">
          <HTMLSelect
            value={quality}
            onChange={(e) => setQuality(e.target.value)}
            options={[
              { value: "draft", label: "草稿 (50%)" },
              { value: "medium", label: "中等 (75%)" },
              { value: "high", label: "高质量 (100%)" },
            ]}
          />
        </FormGroup>

        <div style={{ display: "flex", gap: 8, justifyContent: "flex-end", marginTop: 16 }}>
          <Button onClick={onClose}>取消</Button>
          <Button
            intent={Intent.PRIMARY}
            onClick={handleExport}
            loading={isExporting}
            icon={<Download size={14} />}
          >
            导出
          </Button>
        </div>
      </div>
    </Dialog>
  );
}

export function ExportButton() {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <>
      <Button
        minimal
        small
        icon={<Film size={14} />}
        onClick={() => setIsOpen(true)}
        title="导出"
      />
      <ExportDialog isOpen={isOpen} onClose={() => setIsOpen(false)} />
    </>
  );
}
