import { useState, useEffect } from "react";
import { Button, Dialog, FormGroup, NumericInput, InputGroup, Intent, HTMLSelect } from "@blueprintjs/core";
import { newDocument } from "../api";

interface NewDocumentDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

const PRESETS = [
  { label: "Full HD (1920×1080)", width: 1920, height: 1080 },
  { label: "HD (1280×720)", width: 1280, height: 720 },
  { label: "4K (3840×2160)", width: 3840, height: 2160 },
  { label: "A4 300dpi (2480×3508)", width: 2480, height: 3508 },
  { label: "方形 (1080×1080)", width: 1080, height: 1080 },
  { label: "自定义", width: 0, height: 0 },
];

export default function NewDocumentDialog({ isOpen, onClose }: NewDocumentDialogProps) {
  const [name, setName] = useState("未命名");
  const [width, setWidth] = useState(1920);
  const [height, setHeight] = useState(1080);
  const [fps, setFps] = useState(24);
  const [totalFrames, setTotalFrames] = useState(100);
  const [preset, setPreset] = useState("Full HD (1920×1080)");
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (isOpen) {
      setError(null);
      setIsCreating(false);
    }
  }, [isOpen]);

  const handlePresetChange = (value: string) => {
    setPreset(value);
    const found = PRESETS.find(p => p.label === value);
    if (found && found.width > 0) {
      setWidth(found.width);
      setHeight(found.height);
    }
  };

  const handleCreate = async () => {
    if (width < 1 || height < 1 || fps < 1 || totalFrames < 1) {
      setError("所有值必须大于 0");
      return;
    }
    if (width > 8192 || height > 8192) {
      setError("分辨率不能超过 8192");
      return;
    }

    setIsCreating(true);
    setError(null);

    try {
      await newDocument(name || "未命名", width, height, fps, totalFrames);
      window.dispatchEvent(new CustomEvent("retas:state-changed"));
      onClose();
    } catch (e) {
      setError(`创建失败: ${e}`);
    } finally {
      setIsCreating(false);
    }
  };

  return (
    <Dialog
      isOpen={isOpen}
      onClose={onClose}
      title="新建文档"
      style={{ width: 400 }}
      className="bp5-dark"
    >
      <div style={{ padding: 20 }}>
        {error && (
          <div style={{ color: "#f85149", fontSize: 12, marginBottom: 12, padding: "6px 10px", background: "rgba(248,81,73,0.1)", borderRadius: 4 }}>
            {error}
          </div>
        )}

        <FormGroup label="文档名称" labelFor="doc-name">
          <InputGroup
            id="doc-name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="未命名"
          />
        </FormGroup>

        <FormGroup label="预设" labelFor="preset-select">
          <HTMLSelect
            id="preset-select"
            value={preset}
            onChange={(e) => handlePresetChange(e.target.value)}
            fill
          >
            {PRESETS.map(p => (
              <option key={p.label} value={p.label}>{p.label}</option>
            ))}
          </HTMLSelect>
        </FormGroup>

        <div style={{ display: "flex", gap: 12 }}>
          <FormGroup label="宽度 (px)" style={{ flex: 1 }}>
            <NumericInput
              value={width}
              onValueChange={(v) => { setWidth(v); setPreset("自定义"); }}
              min={1}
              max={8192}
              fill
            />
          </FormGroup>
          <FormGroup label="高度 (px)" style={{ flex: 1 }}>
            <NumericInput
              value={height}
              onValueChange={(v) => { setHeight(v); setPreset("自定义"); }}
              min={1}
              max={8192}
              fill
            />
          </FormGroup>
        </div>

        <div style={{ display: "flex", gap: 12 }}>
          <FormGroup label="帧率 (fps)" style={{ flex: 1 }}>
            <NumericInput
              value={fps}
              onValueChange={setFps}
              min={1}
              max={120}
              fill
            />
          </FormGroup>
          <FormGroup label="总帧数" style={{ flex: 1 }}>
            <NumericInput
              value={totalFrames}
              onValueChange={setTotalFrames}
              min={1}
              max={10000}
              fill
            />
          </FormGroup>
        </div>

        <div style={{ display: "flex", justifyContent: "flex-end", gap: 8, marginTop: 12 }}>
          <Button onClick={onClose}>取消</Button>
          <Button
            intent={Intent.PRIMARY}
            onClick={handleCreate}
            loading={isCreating}
          >
            创建
          </Button>
        </div>
      </div>
    </Dialog>
  );
}
