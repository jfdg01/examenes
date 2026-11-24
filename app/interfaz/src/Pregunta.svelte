<script lang="ts" module>
  import type { Correccion, Edicion, PreguntaDeSesion } from "./nucleo/nucleo.js";

  // `opcion` es null con "En blanco".
  export type Eleccion = { opcion: number | null; correccion: Correccion };
</script>

<script lang="ts">
  import type { Snippet } from "svelte";
  import { fly } from "svelte/transition";
  import Cuadricula, { aviso, type Marca } from "./Cuadricula.svelte";

  let {
    q,
    i,
    total,
    simulacro,
    restante,
    eleccion,
    contestada,
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
    corregida,
    revision,
    marcas,
    oneditar,
    onir,
    onterminar,
    etiqueta,
  }: {
    q: PreguntaDeSesion;
    i: number;
    total: number;
    simulacro: boolean;
    restante: number;
    // La elección de esta pantalla. Una pregunta contestada antes de recargar la página no la tiene.
    eleccion: Eleccion | null;
    // Sin respuesta, la pregunta espera su respuesta. La revisión desde el final cuenta como contestada.
    contestada: boolean;
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
    // La sesión corrige al momento, y el simulacro al final.
    corregida: boolean;
    // La revisión desde el final, sin contestar.
    revision: boolean;
    marcas: Marca[];
    // Guarda la edición en el servidor, y da un error o "".
    oneditar: (edicion: Edicion) => Promise<string>;
    onir: (k: number) => void;
    onterminar: () => void;
    // El texto del botón grande: "Siguiente", "Terminar" o "Resultado".
    etiqueta: string;
  } = $props();

  const flecha = "cursor-pointer rounded-[10px] border-2 border-linea bg-tarjeta whitespace-nowrap px-3 py-2.5 text-[14px] text-texto disabled:cursor-default disabled:opacity-30";
  let cuadricula = $state(false);
  let borrador = $state<Edicion | null>(null);
  let errorDeLaEdicion = $state("");
  const enBlanco = $derived(!!borrador && [borrador.pregunta, borrador.explicacion, ...borrador.opciones].some((t) => !t.trim()));
  function editar() {
    errorDeLaEdicion = "";
    borrador = borrador ? null : { pregunta: q.pregunta, opciones: [...q.opciones], explicacion: q.explicacion };
  }
  function ir(k: number) {
    cuadricula = false;
    onir(k);
  }

  // El botón de copiar se marca un segundo después de copiar.
  let copiada = $state(false);
  async function copiar() {
    // Sin permiso del portapapeles, el botón no se marca.
    if (!(await navigator.clipboard.writeText(q.copiar).then(() => true, () => false))) return;
    copiada = true;
    setTimeout(() => (copiada = false), 1000);
  }
  // Los botones de la línea de arriba.
  const accion = "flex h-7 w-8 shrink-0 cursor-pointer items-center justify-center rounded-md border bg-tarjeta text-[12px]";
  const otras = $derived(q.opciones.length - (eleccion?.opcion == null || eleccion.opcion === q.correcta ? 1 : 2));
</script>

<!-- En el PC, cada parte ocupa su área de la rejilla de la ronda, y el cuerpo deja pasar sus 2 columnas. -->
<p
  class="m-0 flex items-center gap-2 px-3.5 pt-2 pb-1.5 text-[12px] whitespace-nowrap text-tenue pc:gap-1.5 pc:px-0 pc:pt-0 pc:pb-2.5 pc:text-[13px] pc:[grid-area:meta]"
>
  <span class="flex-1 truncate">
    <!-- La posición en la ronda va en el botón de la cuadrícula. -->
    {#if revision}Revisión · {q.apartado}
    {:else if simulacro}{Math.floor(restante / 60)}:{String(restante % 60).padStart(2, "0")} · {q.apartado}
    {:else}{q.apartado} · {q.paso}{/if}
  </span>
  <a class={[accion, "border-linea text-tenue"]} title="Plan de estudio" href="/?plan">
    {@render icono(14, "M8 2v4M16 2v4M3 10h18M5 4h14a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2z")}
  </a>
  <button
    class={[accion, borrador ? "border-acento text-acento" : "border-linea text-tenue"]}
    title="Editar la pregunta"
    onclick={editar}
  >
    {@render icono(14, "M12 20h9M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z")}
  </button>
  <button
    class={["ver-nota", accion, notasVisibles ? "border-acento text-acento" : "border-linea text-tenue"]}
    title="Mostrar u ocultar la nota"
    onclick={onnotasvisibles}
  >
    {@render icono(14, "M15.5 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V8.5zM15 3v4a2 2 0 0 0 2 2h4M7 13h7M7 17h4")}
  </button>
  <button
    class={["copiar", accion, copiada ? "border-acierto text-acierto" : "border-linea text-tenue"]}
    title="Copiar la pregunta y las opciones"
    onclick={copiar}
  >
    {@render icono(14, "M5 15V5a2 2 0 0 1 2-2h10M11 9h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-8a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2z")}
  </button>
  <!-- Cerrar va en la esquina, en rojo, para que no se confunda con los otros. -->
  <button
    class={[accion, "border-fallo text-fallo bg-[color-mix(in_srgb,var(--color-fallo)_18%,var(--color-fondo))]!"]}
    title={revision ? "Volver al resultado" : "Abandonar"}
    onclick={onabandonar}
  >
    {@render icono(14, "M6 6l12 12M18 6L6 18", 2.5)}
  </button>
</p>
<div class="h-[3px] bg-linea pc:hidden"><i class="block h-full bg-acento" style:width="{(100 * (i + 1)) / total}%"></i></div>
<div class="min-h-0 flex-1 overflow-auto px-3.5 py-3 pc:contents">
  <div class="pc:[grid-area:izq]">
    {#if borrador}
      {@const campo = "mb-2 block w-full resize-y rounded-[10px] border-2 bg-tarjeta p-2.5 text-[14px] pc:text-[15px]"}
      <p class="m-0 mb-1 text-[12px] text-tenue">El enunciado</p>
      <textarea class={[campo, "border-acento font-semibold"]} rows="3" bind:value={borrador.pregunta}></textarea>
      {#each borrador.opciones as _, k (k)}
        <p class="m-0 mb-1 text-[12px] {k === q.correcta ? 'text-acierto' : 'text-tenue'}">{k === q.correcta ? "La correcta" : "Un distractor"}</p>
        <textarea class={[campo, k === q.correcta ? "border-acierto" : "border-linea"]} rows="2" bind:value={borrador.opciones[k]}></textarea>
      {/each}
      <p class="m-0 mb-1 text-[12px] text-tenue">La explicación</p>
      <textarea class={[campo, "border-linea"]} rows="5" bind:value={borrador.explicacion}></textarea>
      <div class="mt-1 mb-3 grid grid-cols-2 gap-2">
        <button class="cursor-pointer rounded-[10px] border-2 border-linea bg-tarjeta p-3 text-[14px]" onclick={() => (borrador = null)}>Cancelar</button>
        <button
          class="cursor-pointer rounded-[10px] bg-acento p-3 text-[14px] font-semibold text-sobre-acento disabled:cursor-default disabled:bg-linea disabled:text-tenue"
          disabled={enBlanco}
          onclick={async () => {
            errorDeLaEdicion = await oneditar(borrador!);
            if (!errorDeLaEdicion) borrador = null;
          }}>Guardar</button
        >
      </div>
      {#if enBlanco}<p class="m-0 mb-3 text-[12px] text-aviso">Ningún texto va en blanco.</p>{/if}
      {#if errorDeLaEdicion}<p class="m-0 mb-3 text-[12px] text-aviso">{errorDeLaEdicion}</p>{/if}
    {:else}
    <p class="mt-1 mb-3.5 text-[16px]/[1.4] font-semibold pc:mt-1.5 pc:mb-[18px] pc:text-[19px]/[1.5]">{q.pregunta}</p>
    {#each q.opciones as opcion, k (k)}
      {@const marca = !contestada ? "" : !corregida ? (k === eleccion?.opcion ? "elegida" : "") : k === q.correcta ? "acierto" : k === eleccion?.opcion ? "fallo" : ""}
      {@const resto = contestada && corregida && !marca}
      <div
        class={[
          "opcion mb-2 overflow-hidden rounded-[10px] border-2 pc:mb-2.5 pc:rounded-xl",
          marca === "acierto"
            ? "border-acierto bg-[color-mix(in_srgb,var(--color-acierto)_18%,var(--color-fondo))]"
            : marca === "fallo"
              ? "border-fallo bg-[color-mix(in_srgb,var(--color-fallo)_18%,var(--color-fondo))]"
              : marca === "elegida"
                ? "border-acento bg-[color-mix(in_srgb,var(--color-acento)_18%,var(--color-fondo))]"
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
        {:else}
          <!-- El hueco tiene el ancho de «dudo», y el texto de la opción no cambia de líneas al contestar. -->
          <span class="flex w-12 shrink-0 items-center justify-center pc:w-16">
            {#if marca && marca !== "elegida"}
              <span class={["icono flex", marca === "acierto" ? "text-acierto" : "text-fallo"]}>
                {@render icono(24, `M12 2a10 10 0 1 0 0 20a10 10 0 1 0 0-20${marca === "acierto" ? "M7.5 12.5l3 3 6-6.5" : "M8.5 8.5l7 7M15.5 8.5l-7 7"}`, 2.5)}
              </span>
            {/if}
          </span>
        {/if}
      </div>
    {/each}
    {#if contestada && !corregida}
      <p class="mb-2.5 text-[13px] text-tenue">La respuesta queda guardada. La corrección sale al final.</p>
    {/if}
    {#if corregida && contestada}
      <button
        class="chip mb-2 w-full cursor-pointer rounded-lg border border-dashed border-linea px-2.5 py-1.5 text-[12px] text-tenue pc:hidden"
        onclick={onabiertas}
      >
        {abiertas ? "ocultar las otras" : `ver las otras ${otras} opciones`}
      </button>
      {#if eleccion?.correccion.aviso}
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
    {/if}
    {#if notasVisibles && !borrador}
      <textarea
        class="nota mt-3 mb-3 block w-full resize-y rounded-[10px] border-2 border-linea bg-tarjeta p-2.5 text-[13px] pc:mt-4 pc:rounded-xl pc:p-3 pc:text-[14px]"
        rows="3"
        placeholder="La nota de esta pregunta"
        value={nota}
        oninput={(e) => onnota(e.currentTarget.value)}
      ></textarea>
    {/if}
  </div>
  <!-- La edición ya lleva la explicación, así que la hoja se oculta mientras dura. -->
  {#if !borrador}{@render children()}{/if}
</div>
<div class="border-t-2 border-linea px-3.5 pt-2.5 pb-[calc(10px+env(safe-area-inset-bottom))] pc:border-0 pc:p-0 pc:[grid-area:pie]">
  <!-- La ayuda de una pregunta contestada queda invisible, y los botones de debajo no saltan. -->
  <div class={[contestada && "invisible"]}>
    {#if simulacro}
      <button class="mb-2 w-full cursor-pointer rounded-xl border-2 border-dashed border-linea p-3 text-tenue" onclick={() => oncontestar(null, false)}
        >En blanco</button
      >
    {/if}
    <p class="m-0 mb-2 text-center text-[11.5px] text-tenue pc:mt-[1em] pc:mb-0 pc:text-left pc:text-[13px]">Toca el texto si estás seguro. Toca «dudo» si dudas.</p>
  </div>
  <!-- En el móvil, la fila es siempre la página, la cuadrícula y el botón grande. Lo que aún no toca queda invisible, y la cuadrícula no se mueve. -->
  <div class="flex gap-2 pc:hidden">
    <button
      class={["ver-pagina flex shrink-0 cursor-pointer items-center gap-1.5 rounded-[10px] border-2 border-linea bg-tarjeta px-3 text-[13px] whitespace-nowrap", !(contestada && corregida) && "invisible"]}
      onclick={onverpagina}
    >
      {@render icono(16, "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8zM14 2v6h6M8 13h8M8 17h8")} pág. {q.pagina}
    </button>
    {@render rejilla()}
    <button
      class={["min-w-0 flex-1 cursor-pointer rounded-[10px] bg-acento px-2 py-3.5 text-[15px] font-semibold whitespace-nowrap text-sobre-acento", !contestada && "invisible"]}
      onclick={onsiguiente}>{etiqueta}</button
    >
  </div>
  <!-- En el PC, la fila es siempre anterior, la cuadrícula y siguiente. El botón grande de una pregunta corregida va en la hoja. -->
  <div class="hidden gap-2 pc:mt-6 pc:grid pc:grid-cols-[1fr_auto_1fr]">
    <button class={flecha} disabled={i === 0} onclick={() => onir(i - 1)}>‹ Anterior</button>
    {@render rejilla()}
    <button class={flecha} disabled={i === total - 1} onclick={() => onir(i + 1)}>Siguiente ›</button>
  </div>
  {#if error}<p class="m-0 mt-1 text-center text-[12px] text-aviso pc:text-left">{error}</p>{/if}
</div>

{#if cuadricula}
  <button class="fixed inset-0 z-10 cursor-default bg-black/50" aria-label="Cerrar" onclick={() => (cuadricula = false)}></button>
  <div
    class="fixed inset-x-0 bottom-0 z-20 mx-auto max-w-[560px] rounded-t-xl bg-fondo p-3.5 pb-[calc(14px+env(safe-area-inset-bottom))] shadow-xl"
    transition:fly={{ y: 300, duration: 200 }}
  >
    <Cuadricula {marcas} {i} onir={ir} />
    {#if !revision}
      {@const faltan = marcas.filter((m) => m === "falta").length}
      <!-- Terminar queda gris hasta que la ronda no tiene preguntas sin contestar. -->
      <button
        class="mt-3.5 w-full cursor-pointer rounded-[10px] bg-acento p-3 text-[14px] font-semibold text-sobre-acento disabled:cursor-default disabled:bg-linea disabled:text-tenue"
        disabled={faltan > 0}
        onclick={() => {
          cuadricula = false;
          onterminar();
        }}>Terminar {simulacro ? "el simulacro" : "la sesión"}</button
      >
      {#if faltan}
        <p class="mt-2 mb-0 text-center text-[12px] text-aviso">
          {aviso(faltan)}
        </p>
      {/if}
    {/if}
  </div>
{/if}

<!-- Los iconos de la app: un trazo de 24 × 24. -->
<!-- El botón de la cuadrícula, con la posición en la ronda. -->
{#snippet rejilla()}
  <button
    class="flex shrink-0 cursor-pointer items-center justify-center gap-2 self-stretch rounded-[10px] border-2 border-linea bg-tarjeta px-3 py-2 text-[15px] font-semibold whitespace-nowrap text-texto"
    title="Las preguntas"
    onclick={() => (cuadricula = true)}
  >
    {@render icono(18, "M3 3h7v7H3zM14 3h7v7h-7zM3 14h7v7H3zM14 14h7v7h-7z")}
    {i + 1} / {total}
  </button>
{/snippet}

{#snippet icono(lado: number, d: string, grosor = 2)}
  <svg width={lado} height={lado} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width={grosor} stroke-linecap="round" stroke-linejoin="round">
    <path {d} />
  </svg>
{/snippet}
