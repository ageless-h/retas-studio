import { useState, useEffect, useCallback, useRef } from "react";
import { Button, ButtonGroup, Slider } from "@blueprintjs/core";
import { 
  Play, Pause, SkipBack, SkipForward, 
  StepBack, StepForward, Repeat
} from "lucide-react";

interface PlaybackControllerProps {
  totalFrames: number;
  fps?: number;
  currentFrame?: number;
  onFrameChange?: (frame: number) => void;
}

export default function PlaybackController({ 
  totalFrames, 
  fps = 24,
  currentFrame: externalFrame,
  onFrameChange 
}: PlaybackControllerProps) {
  const [internalFrame, setInternalFrame] = useState(1);
  const [isPlaying, setIsPlaying] = useState(false);
  const [isLooping, setIsLooping] = useState(true);
  
  const currentFrame = externalFrame !== undefined ? externalFrame : internalFrame;
  const setCurrentFrame = (frame: number | ((prev: number) => number)) => {
    const newFrame = typeof frame === 'function' ? frame(internalFrame) : frame;
    if (externalFrame === undefined) {
      setInternalFrame(newFrame);
    }
    if (onFrameChange) {
      onFrameChange(newFrame);
    }
  };
  const animationRef = useRef<number | null>(null);
  const lastTimeRef = useRef<number>(0);
  const frameInterval = 1000 / fps;

  const animate = useCallback((timestamp: number) => {
    if (!lastTimeRef.current) lastTimeRef.current = timestamp;
    
    const elapsed = timestamp - lastTimeRef.current;
    
    if (elapsed >= frameInterval) {
      const nextFrame = currentFrame + 1;
      if (nextFrame > totalFrames) {
        if (isLooping) {
          setCurrentFrame(1);
        } else {
          setIsPlaying(false);
          setCurrentFrame(totalFrames);
        }
      } else {
        setCurrentFrame(nextFrame);
      }
      lastTimeRef.current = timestamp;
    }
    
    if (isPlaying) {
      animationRef.current = requestAnimationFrame(animate);
    }
  }, [isPlaying, isLooping, totalFrames, frameInterval, currentFrame]);

  useEffect(() => {
    if (isPlaying) {
      lastTimeRef.current = 0;
      animationRef.current = requestAnimationFrame(animate);
    } else {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    }
    
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [isPlaying, animate]);

  const handlePlayPause = () => {
    setIsPlaying(!isPlaying);
  };

  const handleFirstFrame = () => {
    setIsPlaying(false);
    setCurrentFrame(1);
  };

  const handleLastFrame = () => {
    setIsPlaying(false);
    setCurrentFrame(totalFrames);
  };

  const handlePrevFrame = () => {
    setIsPlaying(false);
    setCurrentFrame(Math.max(1, currentFrame - 1));
  };

  const handleNextFrame = () => {
    setIsPlaying(false);
    setCurrentFrame(Math.min(totalFrames, currentFrame + 1));
  };

  const handleSliderChange = (value: number) => {
    setIsPlaying(false);
    setCurrentFrame(value);
  };

  const toggleLoop = () => {
    setIsLooping(!isLooping);
  };

  return (
    <div style={{
      display: "flex",
      alignItems: "center",
      gap: 12,
      padding: "8px 12px",
      background: "#161b22",
      borderTop: "1px solid #30363d",
    }}>
      {/* Transport controls */}
      <ButtonGroup minimal>
        <Button 
          minimal 
          icon={<SkipBack size={14} />} 
          onClick={handleFirstFrame}
          title="第一帧"
        />
        <Button 
          minimal 
          icon={<StepBack size={14} />} 
          onClick={handlePrevFrame}
          title="上一帧"
        />
        <Button 
          minimal 
          icon={isPlaying ? <Pause size={16} /> : <Play size={16} />}
          onClick={handlePlayPause}
          intent={isPlaying ? "primary" : "none"}
          title={isPlaying ? "暂停" : "播放"}
        />
        <Button 
          minimal 
          icon={<StepForward size={14} />} 
          onClick={handleNextFrame}
          title="下一帧"
        />
        <Button 
          minimal 
          icon={<SkipForward size={14} />} 
          onClick={handleLastFrame}
          title="最后一帧"
        />
      </ButtonGroup>

      {/* Frame slider */}
      <div style={{ flex: 1, display: "flex", alignItems: "center", gap: 8 }}>
        <Slider
          min={1}
          max={totalFrames}
          stepSize={1}
          labelStepSize={Math.ceil(totalFrames / 10)}
          value={currentFrame}
          onChange={handleSliderChange}
          disabled={isPlaying}
        />
      </div>

      {/* Frame info */}
      <div style={{
        fontSize: 12,
        color: "#8b949e",
        whiteSpace: "nowrap",
        minWidth: 120,
        textAlign: "center",
      }}>
        帧: {currentFrame} / {totalFrames}
      </div>

      {/* Loop toggle */}
      <Button 
        minimal 
        icon={<Repeat size={14} />}
        active={isLooping}
        onClick={toggleLoop}
        title={isLooping ? "循环播放: 开" : "循环播放: 关"}
      />

      {/* FPS info */}
      <div style={{
        fontSize: 11,
        color: "#484f58",
        whiteSpace: "nowrap",
      }}>
        {fps} FPS
      </div>
    </div>
  );
}
