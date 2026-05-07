<script lang="ts">
  import { onMount } from "svelte";
  let { accent = "#d97757", trailFade = "rgba(247, 245, 239, 0.32)" }: { accent?: string; trailFade?: string } = $props();
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
    const FONT_SIZE = 13;
    const CHARS = "01ABCDEF#|/\\><";
    // sparser: only ~40% of columns are active
    const colStep = Math.ceil(FONT_SIZE * 2.2);
    let cols = Math.ceil(w / colStep);
    let drops: number[] = new Array(cols).fill(0).map(() => Math.floor(Math.random() * 40));
    let speeds: number[] = new Array(cols).fill(0).map(() => 0.25 + Math.random() * 0.4);

    let raf = 0, running = true;
    function frame() {
      if (!running) return;

      // recompute cols if width changed
      const newCols = Math.ceil(w / colStep);
      if (newCols !== cols) {
        cols = newCols;
        drops = new Array(cols).fill(0).map(() => Math.floor(Math.random() * 40));
        speeds = new Array(cols).fill(0).map(() => 0.25 + Math.random() * 0.4);
      }

      // trail fade — semi-transparent rect over previous frame
      ctx.fillStyle = trailFade;
      ctx.fillRect(0, 0, w, h);

      ctx.font = `${FONT_SIZE}px "JetBrains Mono", ui-monospace, monospace`;
      ctx.textBaseline = "top";

      for (let i = 0; i < cols; i++) {
        const x = i * colStep;
        const y = drops[i] * FONT_SIZE;
        const ch = CHARS[Math.floor(Math.random() * CHARS.length)];

        // leading char — already faint
        ctx.fillStyle = `rgba(${cr},${cg},${cb},0.32)`;
        ctx.fillText(ch, x, y);

        drops[i] += speeds[i];
        if (drops[i] * FONT_SIZE > h && Math.random() > 0.97) {
          drops[i] = -Math.floor(Math.random() * 8);
          speeds[i] = 0.25 + Math.random() * 0.4;
        }
      }

      raf = requestAnimationFrame(frame);
    }
    frame();
    return () => { running = false; cancelAnimationFrame(raf); ro.disconnect(); };
  });
</script>

<canvas bind:this={canvas} class="matrix"></canvas>

<style>
  .matrix { position: absolute; inset: 0; width: 100%; height: 100%; display: block; }
</style>
