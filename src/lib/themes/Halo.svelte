<script lang="ts">
  import { onMount } from "svelte";

  let { accent = "#d97757", rays = 18, speed = 0.0006 }: {
    accent?: string; rays?: number; speed?: number;
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
      t += speed;

      const cx = w * 0.65;       // off-center looks better
      const cy = h * 0.45;
      const maxR = Math.hypot(Math.max(cx, w - cx), Math.max(cy, h - cy)) * 1.2;

      // soft halo background — radial gradient
      const halo = ctx.createRadialGradient(cx, cy, 0, cx, cy, maxR * 0.7);
      halo.addColorStop(0, `rgba(${cr},${cg},${cb},0.25)`);
      halo.addColorStop(0.4, `rgba(${cr},${cg},${cb},0.08)`);
      halo.addColorStop(1, `rgba(${cr},${cg},${cb},0)`);
      ctx.fillStyle = halo;
      ctx.fillRect(0, 0, w, h);

      // rotating rays — wedge-shaped sweeps
      ctx.save();
      ctx.translate(cx, cy);
      const rotation = t * Math.PI * 2;
      for (let i = 0; i < rays; i++) {
        const a = rotation + (i / rays) * Math.PI * 2;
        const len = maxR;
        // each ray: thin wedge using radial gradient on triangle
        const grad = ctx.createLinearGradient(0, 0, Math.cos(a) * len, Math.sin(a) * len);
        const pulse = 0.10 + 0.10 * Math.sin(t * 30 + i * 0.7);
        grad.addColorStop(0, `rgba(${cr},${cg},${cb},${pulse.toFixed(3)})`);
        grad.addColorStop(0.4, `rgba(${cr},${cg},${cb},${(pulse * 0.4).toFixed(3)})`);
        grad.addColorStop(1, `rgba(${cr},${cg},${cb},0)`);
        ctx.fillStyle = grad;
        const half = 0.05;
        ctx.beginPath();
        ctx.moveTo(0, 0);
        ctx.lineTo(Math.cos(a - half) * len, Math.sin(a - half) * len);
        ctx.lineTo(Math.cos(a + half) * len, Math.sin(a + half) * len);
        ctx.closePath();
        ctx.fill();
      }
      ctx.restore();

      // counter-rotating thin rays for layered effect
      ctx.save();
      ctx.translate(cx, cy);
      const rotation2 = -t * Math.PI * 2 * 1.5;
      const rays2 = Math.floor(rays * 1.7);
      for (let i = 0; i < rays2; i++) {
        const a = rotation2 + (i / rays2) * Math.PI * 2;
        const len = maxR * 0.85;
        const alpha = 0.05 + 0.04 * Math.sin(t * 50 + i);
        ctx.strokeStyle = `rgba(${cr},${cg},${cb},${alpha.toFixed(3)})`;
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(Math.cos(a) * 30, Math.sin(a) * 30);
        ctx.lineTo(Math.cos(a) * len, Math.sin(a) * len);
        ctx.stroke();
      }
      ctx.restore();

      // bright core
      const core = ctx.createRadialGradient(cx, cy, 0, cx, cy, 80);
      core.addColorStop(0, `rgba(${cr},${cg},${cb},0.55)`);
      core.addColorStop(0.5, `rgba(${cr},${cg},${cb},0.15)`);
      core.addColorStop(1, `rgba(${cr},${cg},${cb},0)`);
      ctx.fillStyle = core;
      ctx.beginPath();
      ctx.arc(cx, cy, 80, 0, Math.PI * 2);
      ctx.fill();

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

<canvas bind:this={canvas} class="halo"></canvas>

<style>
  .halo {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    display: block;
  }
</style>
