<script lang="ts">
  import { onMount } from "svelte";
  let { accent = "#d97757", nodeCount = 9 }: { accent?: string; nodeCount?: number } = $props();
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

    // fixed nodes (deterministic)
    const seed = (i: number) => ((i * 9301 + 49297) % 233280) / 233280;
    const nodes = Array.from({ length: nodeCount }, (_, i) => ({
      x: 0.10 + seed(i + 11) * 0.80,
      y: 0.15 + seed(i + 41) * 0.70,
    }));

    // pre-compute edges (each node connects to 2-3 nearest)
    type Edge = { a: number; b: number };
    const edges: Edge[] = [];
    for (let i = 0; i < nodes.length; i++) {
      const dists = nodes.map((n, j) => ({
        j,
        d: Math.hypot(nodes[i].x - n.x, nodes[i].y - n.y),
      })).filter(({ j }) => j !== i).sort((a, b) => a.d - b.d);
      for (const { j } of dists.slice(0, 2)) {
        if (!edges.some(e => (e.a === i && e.b === j) || (e.a === j && e.b === i))) {
          edges.push({ a: i, b: j });
        }
      }
    }

    // active packets — each travels along an edge
    type Packet = { edge: number; t: number; speed: number; reverse: boolean; color: [number, number, number] };
    const packets: Packet[] = [];
    function spawnPacket() {
      const edge = Math.floor(Math.random() * edges.length);
      packets.push({
        edge,
        t: 0,
        speed: 0.005 + Math.random() * 0.015,
        reverse: Math.random() > 0.5,
        color: [cr, cg, cb],
      });
    }
    for (let i = 0; i < 6; i++) spawnPacket();

    let raf = 0, last = performance.now(), running = true;

    function frame() {
      if (!running) return;
      const now = performance.now();
      const dt = Math.min(now - last, 50);
      last = now;
      ctx.clearRect(0, 0, w, h);

      // draw edges (very faint)
      ctx.strokeStyle = `rgba(${cr},${cg},${cb},0.07)`;
      ctx.lineWidth = 0.7;
      for (const e of edges) {
        const a = nodes[e.a], b = nodes[e.b];
        ctx.beginPath();
        ctx.moveTo(a.x * w, a.y * h);
        ctx.lineTo(b.x * w, b.y * h);
        ctx.stroke();
      }

      // draw packets and lit edge segment near them
      for (let i = packets.length - 1; i >= 0; i--) {
        const p = packets[i];
        p.t += p.speed * dt * 0.06;
        if (p.t >= 1) { packets.splice(i, 1); continue; }

        const e = edges[p.edge];
        const a = nodes[e.a], b = nodes[e.b];
        const tt = p.reverse ? 1 - p.t : p.t;
        const px = (a.x + (b.x - a.x) * tt) * w;
        const py = (a.y + (b.y - a.y) * tt) * h;

        // short glow trail near packet (last 25% of traveled distance)
        ctx.strokeStyle = `rgba(${cr},${cg},${cb},0.18)`;
        ctx.lineWidth = 1.0;
        const trailT = Math.max(0, p.t - 0.20);
        const trailX = (a.x + (b.x - a.x) * (p.reverse ? 1 - trailT : trailT)) * w;
        const trailY = (a.y + (b.y - a.y) * (p.reverse ? 1 - trailT : trailT)) * h;
        ctx.beginPath();
        ctx.moveTo(trailX, trailY);
        ctx.lineTo(px, py);
        ctx.stroke();

        // packet head — soft dot
        const grad = ctx.createRadialGradient(px, py, 0, px, py, 6);
        grad.addColorStop(0, `rgba(${cr},${cg},${cb},0.50)`);
        grad.addColorStop(1, `rgba(${cr},${cg},${cb},0)`);
        ctx.fillStyle = grad;
        ctx.beginPath();
        ctx.arc(px, py, 6, 0, Math.PI * 2);
        ctx.fill();
        ctx.fillStyle = `rgba(${cr},${cg},${cb},0.55)`;
        ctx.beginPath();
        ctx.arc(px, py, 1.6, 0, Math.PI * 2);
        ctx.fill();
      }

      // spawn new packets steadily
      if (packets.length < 8 && Math.random() < 0.04) spawnPacket();

      // draw nodes (faint)
      ctx.fillStyle = `rgba(${cr},${cg},${cb},0.30)`;
      for (const n of nodes) {
        const nx = n.x * w, ny = n.y * h;
        ctx.beginPath();
        ctx.arc(nx, ny, 2.0, 0, Math.PI * 2);
        ctx.fill();
      }

      raf = requestAnimationFrame(frame);
    }
    frame();
    return () => { running = false; cancelAnimationFrame(raf); ro.disconnect(); };
  });
</script>

<canvas bind:this={canvas} class="pkt"></canvas>

<style>
  .pkt { position: absolute; inset: 0; width: 100%; height: 100%; display: block; }
</style>
