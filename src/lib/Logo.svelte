<script lang="ts">
  import claudeLogo from "../assets/logos/claude.svg";
  import openaiLogo from "../assets/logos/openai.png";
  import openaiWhite from "../assets/logos/openai-white.png";

  let { id, size = 14, dark = false }: { id: string; size?: number; dark?: boolean } = $props();
</script>

{#if id === "claude"}
  <span class="logo-crop" style="width: {size}px; height: {size}px;">
    <img src={claudeLogo} style="width: {Math.round(size * 1.6)}px; height: {Math.round(size * 1.6)}px; top: {-Math.round(size * 0.3)}px; left: {-Math.round(size * 0.3)}px;" alt="Claude" />
  </span>
{:else if id === "openai"}
  <span class="logo-crop" style="width: {size}px; height: {size}px;">
    <img src={dark ? openaiWhite : openaiLogo} style="width: {size}px; height: {size}px; top: 0; left: 0;" alt="OpenAI" />
  </span>
{:else if id === "gemini"}
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none" aria-hidden="true">
    <path
      d="M8 1.5 C 8 5, 8 5, 14.5 8 C 8 11, 8 11, 8 14.5 C 8 11, 8 11, 1.5 8 C 8 5, 8 5, 8 1.5 Z"
      fill="currentColor"
      opacity="0.95"
    />
  </svg>
{:else if id === "perplexity"}
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none" aria-hidden="true">
    <path d="M3 4h7a3 3 0 010 6H6v3M6 7h2" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
  </svg>
{:else if id === "xai"}
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none" aria-hidden="true">
    <path d="M3 3l10 10M13 3L3 13" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
  </svg>
{:else if id === "groq"}
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none" aria-hidden="true">
    <path d="M11 5a3 3 0 10-1 5l1 1v3" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
    <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.2"/>
  </svg>
{:else if id === "mistral"}
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none" aria-hidden="true">
    <path d="M2 3h3v3h3v3h3V6h3v9" stroke="currentColor" stroke-width="1.6" stroke-linejoin="miter"/>
  </svg>
{:else if id === "deepseek"}
  <svg width={size} height={size} viewBox="0 0 16 16" fill="none" aria-hidden="true">
    <path d="M2 8a6 6 0 1112 0M14 8c0 .5-.4.6-1 .6M5 11l3-2 3 2" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
    <circle cx="11" cy="6" r="1" fill="currentColor"/>
  </svg>
{:else}
  <svg width={size} height={size} viewBox="0 0 16 16" aria-hidden="true">
    <circle cx="8" cy="8" r="3" fill="currentColor" />
  </svg>
{/if}

<style>
  .logo-crop {
    display: inline-block;
    position: relative;
    overflow: hidden;
    line-height: 0;
  }
  .logo-crop img {
    position: absolute;
    object-fit: contain;
    display: block;
  }
</style>
