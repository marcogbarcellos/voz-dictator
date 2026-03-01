import { useRef, useEffect } from "react";

interface WaveformProps {
  level: number; // 0-1
  isActive: boolean;
  barCount?: number;
  color?: string;
}

export function Waveform({
  level,
  isActive,
  barCount = 5,
  color = "#F59E0B",
}: WaveformProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animRef = useRef<number>(0);
  const barsRef = useRef<number[]>(Array(barCount).fill(0.1));

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    canvas.width = canvas.offsetWidth * dpr;
    canvas.height = canvas.offsetHeight * dpr;
    ctx.scale(dpr, dpr);

    const width = canvas.offsetWidth;
    const height = canvas.offsetHeight;
    const barWidth = width / (barCount * 2 - 1);
    const gap = barWidth;

    function animate() {
      ctx!.clearRect(0, 0, width, height);

      for (let i = 0; i < barCount; i++) {
        const targetHeight = isActive
          ? Math.max(0.15, level * (0.5 + Math.random() * 0.5))
          : 0.1;

        barsRef.current[i] +=
          (targetHeight - barsRef.current[i]) * 0.15;

        const barHeight = barsRef.current[i] * height;
        const x = i * (barWidth + gap);
        const y = (height - barHeight) / 2;
        const radius = barWidth / 2;

        ctx!.beginPath();
        ctx!.roundRect(x, y, barWidth, barHeight, radius);
        ctx!.fillStyle = color;
        ctx!.fill();
      }

      animRef.current = requestAnimationFrame(animate);
    }

    animate();

    return () => cancelAnimationFrame(animRef.current);
  }, [level, isActive, barCount, color]);

  return (
    <canvas
      ref={canvasRef}
      className="w-12 h-5"
      style={{ imageRendering: "auto" }}
    />
  );
}
