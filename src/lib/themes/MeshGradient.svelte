<script lang="ts">
  import { onMount } from "svelte";

  let { palette = ["#d97757", "#4e86d3", "#c35fc3", "#aadc82"] }: { palette?: string[] } = $props();
  let canvas: HTMLCanvasElement;

  function hex2rgb(h: string): [number, number, number] {
    const m = h.replace("#", "").match(/^([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i);
    return m ? [parseInt(m[1], 16), parseInt(m[2], 16), parseInt(m[3], 16)] : [200, 100, 100];
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

    // each blob has slow elliptical path
    type Blob = { rgb: [number, number, number]; cx0: number; cy0: number; ax: number; ay: number; period: number; phase: number; r: number };
    const blobs: Blob[] = palette.map((c, i) => ({
      rgb: hex2rgb(c),
      cx0: [0.20, 0.80, 0.55, 0.15][i % 4],
      cy0: [0.25, 0.20, 0.78, 0.70][i % 4],
      ax:  [0.22, 0.18, 0.25, 0.20][i % 4],
      ay:  [0.18, 0.22, 0.15, 0.20][i % 4],
      period: 22 + i * 5,
      phase: i * 1.4,
      r: 0.55,
    }));

    let raf = 0, t = 0, running = true;

    function frame() {
      if (!running) return;
      ctx.clearRect(0, 0, w, h);
      t += 1 / 60;

      // soft blend mode for color mixing
      ctx.globalCompositeOperation = "lighter";
      for (const b of blobs) {
        const ph = (t / b.period) * Math.PI * 2 + b.phase;
        const cx = (b.cx0 + Math.sin(ph) * b.ax) * w;
        const cy = (b.cy0 + Math.cos(ph * 0.7) * b.ay) * h;
        const radius = Math.max(w, h) * b.r;
        const grad = ctx.createRadialGradient(cx, cy, 0, cx, cy, radius);
        grad.addColorStop(0, `rgba(${b.rgb[0]},${b.rgb[1]},${b.rgb[2]},0.45)`);
        grad.addColorStop(0.4, `rgba(${b.rgb[0]},${b.rgb[1]},${b.rgb[2]},0.15)`);
        grad.addColorStop(1, `rgba(${b.rgb[0]},${b.rgb[1]},${b.rgb[2]},0)`);
        ctx.fillStyle = grad;
        ctx.fillRect(0, 0, w, h);
      }
      ctx.globalCompositeOperation = "source-over";

      raf = requestAnimationFrame(frame);
    }
    frame();
    return () => { running = false; cancelAnimationFrame(raf); ro.disconnect(); };
  });
</script>

<canvas bind:this={canvas} class="mesh"></canvas>

<style>
  .mesh { position: absolute; inset: 0; width: 100%; height: 100%; display: block; }
</style>
