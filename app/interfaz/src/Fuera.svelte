<script lang="ts">
  import type { Snippet } from "svelte";

  // Un enlace que sale de la app lleva el icono de salir, para que no sorprenda.
  let { href, clase = "", children }: { href: string; clase?: string; children: Snippet } = $props();
</script>

<!-- El icono mide lo que una mayúscula de JetBrains Mono (0,73 em) y se apoya en la línea base: así queda a la altura de "PDF".
     El `viewBox` recorta el margen del dibujo, y el trazo llega justo al borde. -->
<a class="text-acento underline {clase}" {href} target="_blank"
  >{@render children()}<svg class="ml-1 inline size-[0.73em] align-baseline" viewBox="1.7 1.7 20.6 20.6" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round"
    ><path d="M21 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h6M21 3l-9 9M15 3h6v6" /></svg
  ></a
>
