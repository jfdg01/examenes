<script lang="ts" module>
  import type { Correccion, PreguntaDeSesion } from "./nucleo/nucleo.js";

  // `opcion` es null con "En blanco".
  export type Eleccion = { opcion: number | null; correccion: Correccion };
</script>

<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    q,
    i,
    total,
    simulacro,
    restante,
    eleccion,
    abiertas,
    nota,
    notasVisibles,
    error,
    children,
    onabandonar,
    oncontestar,
    onabiertas,
    onnota,
    onnotasvisibles,
    onverpagina,
    onsiguiente,
  }: {
    q: PreguntaDeSesion;
    i: number;
    total: number;
    simulacro: boolean;
    restante: number;
    // Sin `eleccion`, la pregunta espera su respuesta.
    eleccion: Eleccion | null;
    // Las otras opciones de una pregunta contestada, plegadas en el móvil.
    abiertas: boolean;
    nota: string;
    notasVisibles: boolean;
    error: string;
    // La hoja, que va al lado de las opciones.
    children: Snippet;
    onabandonar: () => void;
    oncontestar: (opcion: number | null, seguro: boolean) => void;
    onabiertas: () => void;
    onnota: (texto: string) => void;
    onnotasvisibles: () => void;
    onverpagina: () => void;
    onsiguiente: () => void;
  } = $props();

  const contestada = $derived(eleccion !== null);

  // El botón de copiar se marca un segundo después de copiar.
  let copiada = $state(false);
  async function copiar() {
    // Sin permiso del portapapeles, el botón no se marca.
    if (!(await navigator.clipboard.writeText(q.copiar).then(() => true, () => false))) return;
    copiada = true;
    setTimeout(() => (copiada = false), 1000);
  }
  // Los botones de la línea de arriba.
  const accion = "flex cursor-pointer items-center rounded-md border bg-tarjeta px-2 py-0.5 text-[12px]";
  const otras = $derived(q.opciones.length - (eleccion?.opcion == null || eleccion.opcion === q.correcta ? 1 : 2));
</script>

<!-- En el PC, cada parte ocupa su área de la rejilla de la ronda, y el cuerpo deja pasar sus 2 columnas. -->
<p
  class="m-0 flex items-center gap-2 px-3.5 pt-2 pb-1.5 text-[12px] whitespace-nowrap text-tenue pc:gap-1.5 pc:px-0 pc:pt-0 pc:pb-2.5 pc:text-[13px] pc:[grid-area:meta]"
>
  <span class="flex-1 truncate">
    {simulacro ? "Simulacro · " : ""}{i + 1} / {total} · {q.tema} ·
    {simulacro ? `${Math.floor(restante / 60)}:${String(restante % 60).padStart(2, "0")}` : q.paso}
  </span>
  <button class={[accion, "border-linea text-tenue"]} title="Abandonar" onclick={onabandonar}>✕</button>
  <button
    class={["ver-nota", accion, notasVisibles ? "border-acento text-acento" : "border-linea text-tenue"]}
    title="Mostrar u ocultar la nota"
    onclick={onnotasvisibles}
  >
    {@render icono(14, "M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7M18.5 2.5a2.1 2.1 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z")}
  </button>
  <button
    class={["copiar", accion, copiada ? "border-acierto text-acierto" : "border-linea text-tenue"]}
    title="Copiar la pregunta y las opciones"
    onclick={copiar}
  >
    {@render icono(14, "M5 15V5a2 2 0 0 1 2-2h10M11 9h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-8a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2z")}
  </button>
</p>
<div class="h-[3px] bg-linea pc:hidden"><i class="block h-full bg-acento" style:width="{(100 * (i + 1)) / total}%"></i></div>
<div class="min-h-0 flex-1 overflow-auto px-3.5 py-3 pc:contents">
  <div class="pc:[grid-area:izq]">
    <p class="mt-1 mb-3.5 text-[16px]/[1.4] font-semibold pc:mt-1.5 pc:mb-[18px] pc:text-[19px]/[1.5]">{q.pregunta}</p>
    {#each q.opciones as opcion, k (k)}
      {@const marca = !contestada ? "" : k === q.correcta ? "acierto" : k === eleccion?.opcion ? "fallo" : ""}
      {@const resto = contestada && !marca}
      <div
        class={[
          "opcion mb-2 overflow-hidden rounded-[10px] border-2 pc:mb-2.5 pc:rounded-xl",
          marca === "acierto"
            ? "border-acierto bg-[color-mix(in_srgb,var(--color-acierto)_18%,var(--color-fondo))]"
            : marca === "fallo"
              ? "border-fallo bg-[color-mix(in_srgb,var(--color-fallo)_18%,var(--color-fondo))]"
              : "border-linea bg-tarjeta",
          resto && "opacity-50 pc:opacity-100",
          resto && !abiertas ? "hidden pc:flex" : "flex",
        ]}
      >
        <button
          class="flex-1 cursor-pointer px-3 py-[11px] text-left text-[13.5px] disabled:cursor-default pc:p-3.5 pc:text-[15px]"
          disabled={contestada}
          onclick={() => oncontestar(k, true)}>{opcion}</button
        >
        {#if !contestada}
          <button
            class="w-12 shrink-0 cursor-pointer border-l-2 border-linea bg-[color-mix(in_srgb,var(--color-aviso)_15%,var(--color-fondo))] text-[12px] font-bold text-aviso pc:w-16 pc:text-[13px]"
            onclick={() => oncontestar(k, false)}>dudo</button
          >
        {:else if marca}
          <span class={["icono flex items-center px-3 pc:px-3.5", marca === "acierto" ? "text-acierto" : "text-fallo"]}>
            {@render icono(24, `M12 2a10 10 0 1 0 0 20a10 10 0 1 0 0-20${marca === "acierto" ? "M7.5 12.5l3 3 6-6.5" : "M8.5 8.5l7 7M15.5 8.5l-7 7"}`, 2.5)}
          </span>
        {/if}
      </div>
    {/each}
    {#if eleccion}
      <button
        class="chip mb-2 w-full cursor-pointer rounded-lg border border-dashed border-linea px-2.5 py-1.5 text-[12px] text-tenue pc:hidden"
        onclick={onabiertas}
      >
        {abiertas ? "ocultar las otras" : `ver las otras ${otras} opciones`}
      </button>
      {#if eleccion.correccion.aviso}
        <p
          class={[
            "mb-2.5 rounded-[4px] border-l-[3px] px-2.5 py-1.5 text-[14px] font-semibold pc:mt-1",
            eleccion.correccion.respuesta.resultado === "Fallo"
              ? "bg-[color-mix(in_srgb,var(--color-fallo)_18%,var(--color-fondo))] text-fallo"
              : "bg-[color-mix(in_srgb,var(--color-aviso)_15%,var(--color-fondo))] text-aviso",
          ]}
        >
          {eleccion.correccion.aviso}
        </p>
      {/if}
    {/if}
    {#if notasVisibles}
      <textarea
        class="nota mt-3 block w-full resize-y rounded-[10px] border-2 border-linea bg-tarjeta p-2.5 text-[13px] pc:mt-4 pc:rounded-xl pc:p-3 pc:text-[14px]"
        rows="3"
        placeholder="La nota de esta pregunta"
        value={nota}
        oninput={(e) => onnota(e.currentTarget.value)}
      ></textarea>
    {/if}
  </div>
  {@render children()}
</div>
<div
  class={[
    "border-t-2 border-linea px-3.5 pt-2.5 pb-[calc(10px+env(safe-area-inset-bottom))] pc:border-0 pc:p-0 pc:[grid-area:pie]",
    contestada && "grid grid-cols-[auto_1fr] gap-2 pc:block",
  ]}
>
  {#if !contestada}
    {#if simulacro}
      <button class="mb-2 w-full cursor-pointer rounded-xl border-2 border-dashed border-linea p-3 text-tenue" onclick={() => oncontestar(null, false)}
        >En blanco</button
      >
    {/if}
    <p class="m-0 text-center text-[11.5px] text-tenue pc:my-[1em] pc:text-left pc:text-[13px]">Toca el texto si estás seguro. Toca «dudo» si dudas.</p>
  {:else}
    <button
      class="ver-pagina flex cursor-pointer pc:hidden items-center gap-1.5 rounded-[10px] border-2 border-linea bg-tarjeta px-3.5 text-[13px]"
      onclick={onverpagina}
    >
      {@render icono(16, "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8zM14 2v6h6M8 13h8M8 17h8")} pág. {q.pagina}
    </button>
    <button class="cursor-pointer rounded-[10px] bg-acento p-3.5 text-[15px] font-semibold text-sobre-acento pc:hidden" onclick={onsiguiente}>Siguiente</button>
  {/if}
  {#if error}<p class="col-span-full m-0 mt-1 text-center text-[12px] text-aviso pc:text-left">{error}</p>{/if}
</div>

<!-- Los iconos de la app: un trazo de 24 × 24. -->
{#snippet icono(lado: number, d: string, grosor = 2)}
  <svg width={lado} height={lado} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width={grosor} stroke-linecap="round" stroke-linejoin="round">
    <path {d} />
  </svg>
{/snippet}
