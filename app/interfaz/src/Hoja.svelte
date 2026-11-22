<script lang="ts">
  import type { PreguntaDeSesion } from "./nucleo/nucleo.js";

  // `paso` es el paso del concepto tras la respuesta. Sin `paso`, la pregunta espera su respuesta.
  let { q, paso }: { q: PreguntaDeSesion; paso: string | undefined } = $props();
</script>

<div class="hoja pc:sticky pc:top-4 pc:max-h-[calc(100vh-32px)] pc:overflow-auto pc:[grid-area:hoja]">
  {#if paso === undefined}
    <p class="vacia hidden pc:block pc:text-tenue">Elige una opción. La explicación sale aquí.</p>
  {:else}
    <p>{q.explicacion}</p>
    <p>{q.concepto} ({paso})</p>
    <p>{q.fuente} · <a href={q.pdf} target="_blank">Abrir el PDF en la página {q.pagina}</a></p>
    <img class="hidden w-full pc:block" src={q.imagen} alt="La página {q.pagina} de la fuente" />
  {/if}
</div>
