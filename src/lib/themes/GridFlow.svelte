<script lang="ts">
  import { onMount } from "svelte";

  let { accent = "#d97757", lines = 18, speed = 0.18 }: { accent?: string; lines?: number; speed?: number } = $props();
  let canvas: HTMLCanvasElement;

  function hex2rgb(h: string): [number, number, number] {
    const m = h.replace("#", "").match(/^([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i);
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
      t += speed;

      // vanishing point at top-center, grid recedes upward
      const vpX = w * 0.5;
      const vpY = h * 0.30;
      const horizonY = vpY;
      const groundTop = horizonY + 4;

      // horizontal lines (rows) — y increases as row depth decreases
      // we use 1 / (1 + depth) projection
      ctx.lineCap = "round";
      const rowCount = lines;
      for (let i = 0; i < rowCount; i++) {
        // each row has a virtual depth that decreases with t (move toward viewer)
        const offset = (t + i) % rowCount;
        const depth = 1 + (rowCount - offset) * 0.45;
        const yScreen = horizonY + (h - horizonY) / depth;
        if (yScreen < horizonY || yScreen > h + 2) continue;
        // alpha: brighter near bottom, faded near horizon
        const alpha = 0.10 + 0.55 * (1 - 1 / depth);
        ctx.strokeStyle = `rgba(${cr},${cg},${cb},${alpha.toFixed(3)})`;
        ctx.lineWidth = 0.8 + (1 / depth) * 0.6;
        ctx.beginPath();
        ctx.moveTo(0, yScreen);
        ctx.lineTo(w, yScreen);
        ctx.stroke();
      }

      // vertical lines: from horizon to bottom, converging at vpX
      const cols = 22;
      for (let c = 0; c <= cols; c++) {
        const xBottom = (c / cols) * w * 1.6 - w * 0.3;
        const xTop = vpX + (xBottom - vpX) * 0.05;
        // alpha varies with distance from center
        const distFromCenter = Math.abs(c - cols / 2) / (cols / 2);
        const alpha = 0.10 + 0.40 * (1 - distFromCenter * 0.6);
        ctx.strokeStyle = `rgba(${cr},${cg},${cb},${alpha.toFixed(3)})`;
        ctx.lineWidth = 0.7;
        ctx.beginPath();
        ctx.moveTo(xTop, horizonY);
        ctx.lineTo(xBottom, h);
        ctx.stroke();
      }

      // soft horizon glow
      const horizonGrad = ctx.createLinearGradient(0, horizonY - 30, 0, horizonY + 60);
      horizonGrad.addColorStop(0, `rgba(${cr},${cg},${cb},0)`);
      horizonGrad.addColorStop(0.5, `rgba(${cr},${cg},${cb},0.12)`);
      horizonGrad.addColorStop(1, `rgba(${cr},${cg},${cb},0)`);
      ctx.fillStyle = horizonGrad;
      ctx.fillRect(0, horizonY - 30, w, 90);

      raf = requestAnimationFrame(frame);
    }
    frame();
    return () => { running = false; cancelAnimationFrame(raf); ro.disconnect(); };
  });
</script>

<canvas bind:this={canvas} class="gf"></canvas>

<style>
  .gf { position: absolute; inset: 0; width: 100%; height: 100%; display: block; }
</style>
