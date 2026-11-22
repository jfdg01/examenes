<script lang="ts" module>
  import type { PreguntaDeSesion, Respuesta } from "./nucleo/nucleo.js";

  // `opcion` es null con "En blanco".
  export type Eleccion = { opcion: number | null; respuesta: Respuesta };
</script>

<script lang="ts">
  import type { Snippet } from "svelte";

  let { q, i, total, simulacro, restante, eleccion, abiertas, error, children, onabandonar, oncontestar, onabiertas, onverpagina, onsiguiente }: {
    q: PreguntaDeSesion;
    i: number;
    total: number;
    simulacro: boolean;
    restante: number;
    // Sin `eleccion`, la pregunta espera su respuesta.
    eleccion: Eleccion | null;
    // Las otras opciones de una pregunta contestada, plegadas en el móvil.
    abiertas: boolean;
    error: string;
    // La hoja, que va al lado de las opciones.
    children: Snippet;
    onabandonar: () => void;
    oncontestar: (opcion: number | null, seguro: boolean) => void;
    onabiertas: () => void;
    onverpagina: () => void;
    onsiguiente: () => void;
  } = $props();

  const contestada = $derived(eleccion !== null);
  const otras = $derived(q.opciones.length - (eleccion?.opcion == null || eleccion.opcion === q.correcta ? 1 : 2));
</script>

<!-- En el PC, cada parte ocupa su área de la rejilla de la ronda, y el cuerpo deja pasar sus 2 columnas. -->
<p class="m-0 flex items-center gap-2 px-3.5 py-2 text-[13px] pc:px-0 pc:pt-0 pc:pb-2 pc:[grid-area:meta]">
  <span class="flex-1 truncate">
    {simulacro ? "Simulacro · " : ""}{i + 1} / {total} · {q.tema} ·
    {simulacro ? `${Math.floor(restante / 60)}:${String(restante % 60).padStart(2, "0")}` : q.paso}
  </span>
  <button title="Abandonar" onclick={onabandonar}>✕</button>
</p>
<div class="h-[3px] bg-linea pc:[grid-area:barra]"><i class="block h-full bg-acento" style:width="{(100 * (i + 1)) / total}%"></i></div>
<div class="min-h-0 flex-1 overflow-auto px-3.5 py-3 pc:contents">
  <div class="pc:pt-3 pc:[grid-area:izq]">
    <p>{q.pregunta}</p>
    {#each q.opciones as opcion, k (k)}
      {@const marca = !contestada ? "" : k === q.correcta ? "✓" : k === eleccion?.opcion ? "✗" : ""}
      {@const resto = contestada && !marca}
      <div class={["opcion mb-2 items-center gap-1.5", resto && "opacity-50", resto && !abiertas ? "hidden pc:flex" : "flex"]}>
        <button class="flex-1 p-2.5 text-left" disabled={contestada} onclick={() => oncontestar(k, true)}>{opcion}</button>
        {#if !contestada}
          <button onclick={() => oncontestar(k, false)}>dudo</button>
        {:else}
          {marca}
        {/if}
      </div>
    {/each}
    {#if eleccion}
      <button class="chip mb-2 w-full border border-dashed border-chip bg-transparent pc:hidden" onclick={onabiertas}>
        {abiertas ? "ocultar las otras" : `ver las otras ${otras} opciones`}
      </button>
      {#if eleccion.respuesta.resultado === "Fallo" && eleccion.respuesta.seguro}
        <p>Fallo con «seguro»: una idea falsa. Repásala.</p>
      {:else if eleccion.respuesta.resultado === "Acierto" && !eleccion.respuesta.seguro}
        <p>Acierto con «dudo»: cuenta como un fallo.</p>
      {/if}
    {/if}
  </div>
  {@render children()}
</div>
<div class="border-t-2 border-linea px-3.5 pt-2.5 pb-[calc(10px+env(safe-area-inset-bottom))] pc:border-0 pc:p-0 pc:[grid-area:pie]">
  {#if !contestada}
    {#if simulacro}<button onclick={() => oncontestar(null, false)}>En blanco</button>{/if}
    <p class="mx-0 mt-1 mb-0 text-center text-[12px]">Toca el texto si estás seguro. Toca «dudo» si dudas.</p>
  {:else}
    <button class="ver-pagina pc:hidden" onclick={onverpagina}>pág. {q.pagina}</button>
    <button onclick={onsiguiente}>Siguiente</button>
  {/if}
  {#if error}<p class="mx-0 mt-1 mb-0 text-center text-[12px] text-mal">{error}</p>{/if}
</div>
