export interface DrawEvent {
  type: "mousedown" | "mousemove" | "mouseup" | "flush" | "composite" | "error";
  timestamp: number;
  x?: number;
  y?: number;
  points?: number;
  message?: string;
  error?: string;
  canvasSnapshot?: string;
  paintColor?: string;
  paintWidth?: number;
}

export class CanvasMonitor {
  private events: DrawEvent[] = [];
  private maxEvents = 500;
  private isRecording = false;
  private surfaceRef: any = null;

  start() {
    this.isRecording = true;
    this.events = [];
  }

  stop() {
    this.isRecording = false;
  }

  setSurface(surface: any) {
    this.surfaceRef = surface;
  }

  log(event: Omit<DrawEvent, "timestamp">) {
    if (!this.isRecording) return;
    this.events.push({
      ...event,
      timestamp: performance.now(),
    });
    if (this.events.length > this.maxEvents) {
      this.events.shift();
    }
  }

  captureSnapshot(): string | null {
    if (!this.surfaceRef) return null;
    try {
      const snapshot = this.surfaceRef.makeImageSnapshot();
      if (!snapshot) return null;
      const data = snapshot.encodeToBytes();
      if (data) {
        const blob = new Blob([data], { type: "image/png" });
        const url = URL.createObjectURL(blob);
        snapshot.delete();
        return url;
      }
      snapshot.delete();
      return null;
    } catch (e) {
      return null;
    }
  }

  getEvents(): DrawEvent[] {
    return [...this.events];
  }

  getLastEvents(n: number): DrawEvent[] {
    return this.events.slice(-n);
  }

  clear() {
    this.events = [];
  }
}

export const canvasMonitor = new CanvasMonitor();
