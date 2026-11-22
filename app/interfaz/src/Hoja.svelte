<script lang="ts">
  import type { PreguntaDeSesion, Respuesta } from "./nucleo/nucleo.js";

  // Sin `respuesta`, la pregunta espera su respuesta.
  let { q, respuesta }: { q: PreguntaDeSesion; respuesta: Respuesta | undefined } = $props();
</script>

<div class="hoja pc:sticky pc:top-4 pc:max-h-[calc(100vh-32px)] pc:overflow-auto pc:[grid-area:hoja]">
  {#if !respuesta}
    <p class="vacia hidden pc:block pc:text-tenue">Elige una opción. La explicación sale aquí.</p>
  {:else}
    <p>{q.explicacion}</p>
    <p>{q.concepto} ({respuesta.resultado === "Acierto" && respuesta.seguro ? q.paso_tras_seguro : q.paso_tras_otra})</p>
    <p>{q.fuente} · <a href={q.pdf} target="_blank">Abrir el PDF en la página {q.pagina}</a></p>
    <img class="hidden w-full pc:block" src={q.imagen} alt="La página {q.pagina} de la fuente" />
  {/if}
</div>
