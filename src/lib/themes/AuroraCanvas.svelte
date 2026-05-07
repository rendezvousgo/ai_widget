<script lang="ts">
  import { onMount } from "svelte";

  let { palette = ["#d97757", "#c35fc3", "#4e86d3", "#aadc82"] }: { palette?: string[] } = $props();
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

    // each ribbon: a curved horizontal band that wobbles
    type Ribbon = { rgb: [number, number, number]; baseY: number; period: number; speed: number; ampY: number; thick: number; phase: number };
    const ribbons: Ribbon[] = [
      { rgb: hex2rgb(palette[0]), baseY: 0.20, period: 28, speed: 0.8,  ampY: 0.10, thick: 0.30, phase: 0 },
      { rgb: hex2rgb(palette[1]), baseY: 0.45, period: 36, speed: -0.6, ampY: 0.08, thick: 0.32, phase: 1.4 },
      { rgb: hex2rgb(palette[2]), baseY: 0.65, period: 32, speed: 0.7,  ampY: 0.09, thick: 0.30, phase: 2.7 },
      { rgb: hex2rgb(palette[3] ?? palette[0]), baseY: 0.85, period: 30, speed: -0.9, ampY: 0.07, thick: 0.28, phase: 4.0 },
    ];

    let raf = 0, t = 0, running = true;

    function drawRibbon(rb: Ribbon) {
      const segments = 80;
      const ribbonH = h * rb.thick;
      // top edge (y_top per x)
      const topY: number[] = new Array(segments + 1);
      for (let i = 0; i <= segments; i++) {
        const x = (i / segments) * w;
        const phaseShift = rb.phase + t * rb.speed * 0.4;
        const wave =
          Math.sin((x / w) * Math.PI * 2 * 1.2 + phaseShift) * rb.ampY +
          Math.sin((x / w) * Math.PI * 2 * 2.8 + phaseShift * 1.7) * rb.ampY * 0.4;
        topY[i] = rb.baseY * h + wave * h - ribbonH * 0.5;
      }
      // build closed path: top edge then back along bottom edge
      ctx.beginPath();
      ctx.moveTo(0, topY[0]);
      for (let i = 1; i <= segments; i++) ctx.lineTo((i / segments) * w, topY[i]);
      for (let i = segments; i >= 0; i--) ctx.lineTo((i / segments) * w, topY[i] + ribbonH);
      ctx.closePath();

      // vertical gradient — transparent edges, bright middle
      const gradY1 = (rb.baseY * h) - ribbonH * 0.5;
      const gradY2 = gradY1 + ribbonH;
      const grad = ctx.createLinearGradient(0, gradY1, 0, gradY2);
      const [r, g, b] = rb.rgb;
      grad.addColorStop(0, `rgba(${r},${g},${b},0)`);
      grad.addColorStop(0.5, `rgba(${r},${g},${b},0.55)`);
      grad.addColorStop(1, `rgba(${r},${g},${b},0)`);
      ctx.fillStyle = grad;
      ctx.fill();
    }

    function frame() {
      if (!running) return;
      ctx.clearRect(0, 0, w, h);
      t += 1 / 60;

      ctx.globalCompositeOperation = "lighter";
      for (const r of ribbons) drawRibbon(r);
      ctx.globalCompositeOperation = "source-over";

      raf = requestAnimationFrame(frame);
    }
    frame();
    return () => { running = false; cancelAnimationFrame(raf); ro.disconnect(); };
  });
</script>

<canvas bind:this={canvas} class="aur"></canvas>

<style>
  .aur { position: absolute; inset: 0; width: 100%; height: 100%; display: block; }
</style>
