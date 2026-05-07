<script lang="ts">
  import { onMount } from "svelte";
  let { accent = "#d97757" }: { accent?: string } = $props();
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
      t += 0.05;

      // no grid — too heavy for ambient bg

      // dual waveform with phosphor glow effect (multi-pass strokes)
      const drawWave = (phase: number, amp: number, freq: number, width: number, alpha: number) => {
        ctx.strokeStyle = `rgba(${cr},${cg},${cb},${alpha})`;
        ctx.lineWidth = width;
        ctx.lineJoin = "round";
        ctx.beginPath();
        for (let x = 0; x <= w; x += 2) {
          const u = x / w;
          const y = h / 2
            + Math.sin(u * Math.PI * freq + t + phase) * h * amp
            + Math.sin(u * Math.PI * freq * 2.7 + t * 1.4 + phase) * h * amp * 0.25
            + Math.sin(u * Math.PI * freq * 5.1 - t * 0.6 + phase) * h * amp * 0.10;
          if (x === 0) ctx.moveTo(x, y);
          else ctx.lineTo(x, y);
        }
        ctx.stroke();
      };

      // single faint wave — ambient feel
      drawWave(0, 0.10, 4, 1.0, 0.18);
      drawWave(Math.PI / 3, 0.07, 7, 0.8, 0.10);

      raf = requestAnimationFrame(frame);
    }
    frame();
    return () => { running = false; cancelAnimationFrame(raf); ro.disconnect(); };
  });
</script>

<canvas bind:this={canvas} class="scope"></canvas>

<style>
  .scope { position: absolute; inset: 0; width: 100%; height: 100%; display: block; }
</style>
