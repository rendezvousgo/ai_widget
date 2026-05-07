<script lang="ts">
  import { onMount } from "svelte";

  let { accent = "#d97757", cols = 24, rows = 13, amplitude = 14, density = 0.22 }: {
    accent?: string; cols?: number; rows?: number; amplitude?: number; density?: number;
  } = $props();
  let canvas: HTMLCanvasElement;

  function hex2rgb(hex: string): [number, number, number] {
    const m = hex.replace("#", "").match(/^([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i);
    return m ? [parseInt(m[1], 16), parseInt(m[2], 16), parseInt(m[3], 16)] : [217, 119, 87];
  }

  onMount(() => {
    const ctx = canvas.getContext("2d", { alpha: true });
    if (!ctx) return;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    let w = canvas.clientWidth, h = canvas.clientHeight;
    function resize() {
      w = canvas.clientWidth; h = canvas.clientHeight;
      canvas.width = w * dpr; canvas.height = h * dpr;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    }
    resize();
    const ro = new ResizeObserver(resize);
    ro.observe(canvas);

    const [cr, cg, cb] = hex2rgb(accent);
    let raf = 0, t = 0, running = true;

    function frame() {
      if (!running) return;
      ctx.clearRect(0, 0, w, h);
      t += 0.015;

      const sx = w / (cols - 1);
      const sy = (h * 0.85) / (rows - 1);
      const yOff = h * 0.10;

      // compute grid points
      const pts: Array<Array<{ x: number; y: number; depth: number }>> = [];
      for (let r = 0; r < rows; r++) {
        const row: Array<{ x: number; y: number; depth: number }> = [];
        for (let c = 0; c < cols; c++) {
          const baseX = c * sx;
          const baseY = r * sy + yOff;
          // multi-frequency wave
          const wave =
            Math.sin(c * 0.45 + t * 0.9 + r * 0.18) * amplitude +
            Math.sin(r * 0.55 + t * 1.1 - c * 0.08) * amplitude * 0.6 +
            Math.cos((c + r) * 0.35 + t * 0.7) * amplitude * 0.4;
          // depth = 0 (top/far) → 1 (bottom/near). lines fade with depth.
          const depth = r / (rows - 1);
          row.push({ x: baseX, y: baseY + wave, depth });
        }
        pts.push(row);
      }

      // draw connecting lines (horizontal + vertical)
      ctx.lineWidth = 0.9;
      for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols - 1; c++) {
          const a = pts[r][c], b = pts[r][c + 1];
          // alpha modulated by depth (closer = brighter) and wave height
          const alpha = density * (0.45 + 0.55 * a.depth);
          ctx.strokeStyle = `rgba(${cr},${cg},${cb},${alpha.toFixed(3)})`;
          ctx.beginPath();
          ctx.moveTo(a.x, a.y);
          ctx.lineTo(b.x, b.y);
          ctx.stroke();
        }
      }
      for (let c = 0; c < cols; c++) {
        for (let r = 0; r < rows - 1; r++) {
          const a = pts[r][c], b = pts[r + 1][c];
          const alpha = density * (0.45 + 0.55 * a.depth);
          ctx.strokeStyle = `rgba(${cr},${cg},${cb},${alpha.toFixed(3)})`;
          ctx.beginPath();
          ctx.moveTo(a.x, a.y);
          ctx.lineTo(b.x, b.y);
          ctx.stroke();
        }
      }

      // draw nodes (small dots, brighter near bottom)
      for (let r = 0; r < rows; r++) {
        for (let c = 0; c < cols; c++) {
          const p = pts[r][c];
          const alpha = density * (0.6 + 0.4 * p.depth) * 1.6;
          ctx.fillStyle = `rgba(${cr},${cg},${cb},${Math.min(0.9, alpha).toFixed(3)})`;
          ctx.beginPath();
          ctx.arc(p.x, p.y, 1.0 + p.depth * 0.6, 0, Math.PI * 2);
          ctx.fill();
        }
      }

      raf = requestAnimationFrame(frame);
    }
    frame();

    return () => {
      running = false;
      cancelAnimationFrame(raf);
      ro.disconnect();
    };
  });
</script>

<canvas bind:this={canvas} class="wf"></canvas>

<style>
  .wf {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    display: block;
  }
</style>
