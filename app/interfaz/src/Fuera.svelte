<script lang="ts">
  import type { Snippet } from "svelte";

  // Un enlace que sale de la app lleva el icono de salir, para que no sorprenda.
  let { href, clase = "", children }: { href: string; clase?: string; children: Snippet } = $props();
</script>

<a class="inline-flex items-center gap-1 text-acento underline {clase}" {href} target="_blank"
  >{@render children()}<svg class="shrink-0" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"
    ><path d="M21 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h6M21 3l-9 9M15 3h6v6" /></svg
  ></a
>
