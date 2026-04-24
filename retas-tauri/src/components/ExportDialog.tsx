import { useState } from "react";
import { Button, Dialog, FormGroup, HTMLSelect, Intent, Callout, RadioGroup, Radio, NumericInput } from "@blueprintjs/core";
import { Download, Film } from "lucide-react";
import { exportImage, exportFrameSequence, getFrameInfo } from "../api";
import { showSaveDialog } from "../utils/fileDialog";

interface ExportDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function ExportDialog({ isOpen, onClose }: ExportDialogProps) {
  const [format, setFormat] = useState("png");
  const [quality, setQuality] = useState("high");
  const [exportMode, setExportMode] = useState("current");
  const [isExporting, setIsExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const handleExport = async () => {
    setIsExporting(true);
    setError(null);
    setSuccess(false);
    
    try {
      if (exportMode === "current") {
        // Export current frame as single image
        const filters = format === "png"
          ? [{ name: "PNG 图片", extensions: ["png"] }]
          : [{ name: "JPEG 图片", extensions: ["jpg", "jpeg"] }];
        
        const path = await showSaveDialog(filters);
        if (!path) {
          setIsExporting(false);
          return;
        }
        
        await exportImage(path, format);
        setSuccess(true);
      } else {
        // Export frame sequence to directory
        const path = await showSaveDialog([
          { name: `${format.toUpperCase()} 序列帧`, extensions: [format === "jpeg" ? "jpg" : format] },
        ]);
        if (!path) {
          setIsExporting(false);
          return;
        }
        
        // Use directory of the selected file
        const dir = path.substring(0, path.lastIndexOf("\\") || path.lastIndexOf("/")) || path;
        const frameInfo = await getFrameInfo();
        
        await exportFrameSequence(dir, format, 1, frameInfo.total);
        setSuccess(true);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <Dialog
      isOpen={isOpen}
      onClose={onClose}
      title="导出"
      style={{ width: 420 }}
    >
      <div style={{ padding: 16 }}>
        {error && (
          <Callout intent={Intent.DANGER} style={{ marginBottom: 12 }}>
            导出失败: {error}
          </Callout>
        )}
        {success && (
          <Callout intent={Intent.SUCCESS} style={{ marginBottom: 12 }}>
            导出完成！
          </Callout>
        )}
        
        <FormGroup label="导出模式">
          <RadioGroup
            selectedValue={exportMode}
            onChange={(e) => setExportMode((e.target as HTMLInputElement).value)}
          >
            <Radio label="当前帧" value="current" />
            <Radio label="序列帧 (全部帧)" value="sequence" />
          </RadioGroup>
        </FormGroup>

        <FormGroup label="格式">
          <HTMLSelect
            value={format}
            onChange={(e) => setFormat(e.target.value)}
            options={[
              { value: "png", label: "PNG" },
              { value: "jpeg", label: "JPEG" },
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
