<script lang="ts">
  import { onMount } from "svelte";

  let {
    accent = "#d97757",
    count = 32,
    maxDist = 100,
    nodeAlpha = 0.45,
    edgeAlpha = 0.20,
    nodeRadius = 1.1,
    speed = 0.22,
  }: {
    accent?: string;
    count?: number;
    maxDist?: number;
    nodeAlpha?: number;
    edgeAlpha?: number;
    nodeRadius?: number;
    speed?: number;
  } = $props();
  let canvas: HTMLCanvasElement;

  function hexToRgb(hex: string): [number, number, number] {
    const m = hex.replace("#", "").match(/^([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i);
    if (!m) return [217, 119, 87];
    return [parseInt(m[1], 16), parseInt(m[2], 16), parseInt(m[3], 16)];
  }

  onMount(() => {
    const ctx = canvas.getContext("2d", { alpha: true });
    if (!ctx) return;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    let w = canvas.clientWidth;
    let h = canvas.clientHeight;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    ctx.scale(dpr, dpr);

    const [cr, cg, cb] = hexToRgb(accent);

    type P = { x: number; y: number; vx: number; vy: number };
    const ps: P[] = [];
    for (let i = 0; i < count; i++) {
      ps.push({
        x: Math.random() * w,
        y: Math.random() * h,
        vx: (Math.random() - 0.5) * speed,
        vy: (Math.random() - 0.5) * speed,
      });
    }

    let raf = 0;
    let running = true;
    const ro = new ResizeObserver(() => {
      w = canvas.clientWidth;
      h = canvas.clientHeight;
      canvas.width = w * dpr;
      canvas.height = h * dpr;
      ctx.scale(dpr, dpr);
    });
    ro.observe(canvas);

    function frame() {
      if (!running) return;
      ctx.clearRect(0, 0, w, h);

      // update + draw nodes
      for (const p of ps) {
        p.x += p.vx;
        p.y += p.vy;
        if (p.x < 0 || p.x > w) p.vx = -p.vx;
        if (p.y < 0 || p.y > h) p.vy = -p.vy;
      }

      // edges with distance-based alpha
      ctx.lineWidth = 0.8;
      for (let i = 0; i < count; i++) {
        for (let j = i + 1; j < count; j++) {
          const dx = ps[i].x - ps[j].x;
          const dy = ps[i].y - ps[j].y;
          const d2 = dx * dx + dy * dy;
          if (d2 < maxDist * maxDist) {
            const a = (1 - Math.sqrt(d2) / maxDist) * edgeAlpha;
            ctx.strokeStyle = `rgba(${cr},${cg},${cb},${a.toFixed(3)})`;
            ctx.beginPath();
            ctx.moveTo(ps[i].x, ps[i].y);
            ctx.lineTo(ps[j].x, ps[j].y);
            ctx.stroke();
          }
        }
      }

      // nodes
      ctx.fillStyle = `rgba(${cr},${cg},${cb},${nodeAlpha})`;
      for (const p of ps) {
        ctx.beginPath();
        ctx.arc(p.x, p.y, nodeRadius, 0, Math.PI * 2);
        ctx.fill();
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

<canvas bind:this={canvas} class="net"></canvas>

<style>
  .net {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    display: block;
  }
</style>
