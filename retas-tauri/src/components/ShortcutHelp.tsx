import { useState } from "react";
import { Button, Dialog, Tag } from "@blueprintjs/core";
import { Keyboard } from "lucide-react";

const SHORTCUTS = [
  { key: "B", action: "画笔工具" },
  { key: "E", action: "橡皮工具" },
  { key: "V", action: "选择工具" },
  { key: "P", action: "钢笔工具" },
  { key: "G", action: "填充工具" },
  { key: "I", action: "吸管工具" },
  { key: "H", action: "抓手工具" },
  { key: "Ctrl + Z", action: "撤销" },
  { key: "Ctrl + Y", action: "重做" },
  { key: "Ctrl + C", action: "复制选区像素" },
  { key: "Ctrl + V", action: "粘贴像素" },
  { key: "Ctrl + +", action: "放大" },
  { key: "Ctrl + -", action: "缩小" },
  { key: "Space", action: "平移画布" },
  { key: "[ / ]", action: "减小/增大笔刷" },
];

export default function ShortcutHelp() {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <>
      <Button
        minimal
        small
        icon={<Keyboard size={14} />}
        onClick={() => setIsOpen(true)}
        title="快捷键帮助"
      />
      
      <Dialog
        isOpen={isOpen}
        onClose={() => setIsOpen(false)}
        title="快捷键"
        style={{ width: 400 }}
      >
        <div style={{ padding: 16 }}>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 2fr", gap: "8px 16px" }}>
            {SHORTCUTS.map(({ key, action }) => (
              <div key={key} style={{ display: "contents" }}>
                <Tag minimal intent="primary" style={{ justifySelf: "start" }}>
                  {key}
                </Tag>
                <span style={{ color: "#e6edf3", fontSize: 13 }}>{action}</span>
              </div>
            ))}
          </div>
        </div>
      </Dialog>
    </>
  );
}
