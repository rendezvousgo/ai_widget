<script lang="ts">
  import { onMount } from "svelte";
  let { accent = "#d97757", bars = 56, opacityScale = 0.30 }: { accent?: string; bars?: number; opacityScale?: number } = $props();
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
    const peakHold: number[] = new Array(bars).fill(0);

    function frame() {
      if (!running) return;
      ctx.clearRect(0, 0, w, h);
      t += 0.045;

      const gap = 2;
      const totalGap = gap * (bars - 1);
      const barW = (w - totalGap) / bars;
      const baseY = h * 0.95;
      const maxBarH = h * 0.55;

      for (let i = 0; i < bars; i++) {
        // pseudo-spectrum: low freqs higher amplitude; mix of multiple sin waves per bar
        const f = i / bars; // 0..1 (bass → treble)
        const lowMul = Math.pow(1 - f, 1.4) * 0.7 + 0.3;
        const sig =
          0.5 + 0.5 * Math.sin(t * (0.5 + f * 1.2) + i * 0.18) * 0.6 +
          0.5 * Math.sin(t * (0.9 - f * 0.6) + i * 0.31) * 0.3 +
          0.5 * Math.cos(t * 1.4 + i * 0.07) * 0.2;
        const norm = Math.max(0, Math.min(1, sig)) * lowMul;
        const barH = norm * maxBarH;

        const x = i * (barW + gap);
        const g = ctx.createLinearGradient(0, baseY - barH, 0, baseY);
        g.addColorStop(0, `rgba(${cr},${cg},${cb},${(opacityScale).toFixed(3)})`);
        g.addColorStop(0.6, `rgba(${cr},${cg},${cb},${(opacityScale * 0.6).toFixed(3)})`);
        g.addColorStop(1, `rgba(${cr},${cg},${cb},${(opacityScale * 0.15).toFixed(3)})`);
        ctx.fillStyle = g;
        ctx.fillRect(x, baseY - barH, barW, barH);

        peakHold[i] = Math.max(peakHold[i] - 0.5, barH);
        ctx.fillStyle = `rgba(${cr},${cg},${cb},${(opacityScale * 1.2).toFixed(3)})`;
        ctx.fillRect(x, baseY - peakHold[i] - 1, barW, 1);
      }

      raf = requestAnimationFrame(frame);
    }
    frame();
    return () => { running = false; cancelAnimationFrame(raf); ro.disconnect(); };
  });
</script>

<canvas bind:this={canvas} class="spec"></canvas>

<style>
  .spec { position: absolute; inset: 0; width: 100%; height: 100%; display: block; }
</style>
