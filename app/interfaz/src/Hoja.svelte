<script lang="ts">
  import type { PreguntaDeSesion } from "./nucleo/nucleo.js";

  // `vuelve` dice cuándo vuelve la pregunta tras la respuesta. Sin `vuelve`, la pregunta espera su respuesta.
  // En el PC, "Siguiente" va en la hoja, debajo de la explicación. En el móvil va en el pie.
  let { q, vuelve, onsiguiente, etiqueta }: { q: PreguntaDeSesion; vuelve: string | undefined; onsiguiente: () => void; etiqueta: string } = $props();
</script>

<div class="hoja pc:sticky pc:top-4 pc:max-h-[calc(100vh-32px)] pc:overflow-auto pc:[grid-area:hoja]">
  {#if vuelve === undefined}
    <p class="vacia hidden pc:my-[1em] pc:block pc:text-tenue">Elige una opción. La explicación sale aquí.</p>
  {:else}
    <p class="mb-2.5 whitespace-pre-line">{q.explicacion}</p>
    <button
      class="my-3 hidden w-full cursor-pointer rounded-xl bg-acento p-4 text-[16px] font-semibold text-sobre-acento pc:block"
      onclick={onsiguiente}>{etiqueta}</button
    >
    <div class="mt-3 text-[13px] text-tenue">
      <p class="my-2">
        {q.fuente} · {q.concepto} · {vuelve} ·
        <a class="text-acento underline" href={q.pdf} target="_blank">Abrir el PDF en la página {q.pagina}</a>
      </p>
      <img class="my-2 mt-1.5 hidden w-full rounded-lg bg-white pc:block" src={q.imagen} alt="La página {q.pagina} de la fuente" />
    </div>
  {/if}
</div>
