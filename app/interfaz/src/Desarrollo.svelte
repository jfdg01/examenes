<script lang="ts">
  import type { NotaDeDesarrollo, Pagina, PreguntaDeDesarrolloDeSesion } from "./nucleo/nucleo.js";
  import Pomodoro from "./Pomodoro.svelte";

  let {
    q,
    i,
    total,
    texto,
    solucion,
    nota,
    notasVisibles,
    error,
    ontexto,
    onsolucion,
    oncalificar,
    onnota,
    onnotasvisibles,
    onverpagina,
    onabandonar,
  }: {
    q: PreguntaDeDesarrolloDeSesion;
    i: number;
    total: number;
    texto: string;
    // La respuesta modelo está abierta.
    solucion: boolean;
    nota: string;
    notasVisibles: boolean;
    error: string;
    ontexto: (texto: string) => void;
    onsolucion: () => void;
    oncalificar: (nota: NotaDeDesarrollo) => void;
    onnota: (texto: string) => void;
    onnotasvisibles: () => void;
    onverpagina: (pagina: Pagina) => void;
    onabandonar: () => void;
  } = $props();

  const accion = "flex h-7 w-8 shrink-0 cursor-pointer items-center justify-center rounded-md border bg-tarjeta text-[12px]";
  const calificaciones: [NotaDeDesarrollo, string, string][] = [
    ["Again", "No la sabía", "border-fallo text-fallo"],
    ["Hard", "A medias", "border-aviso text-aviso"],
    ["Good", "La sabía", "border-acierto text-acierto"],
  ];
</script>

<div class="desarrollo flex h-dvh flex-col text-[14px]/[1.45] pc:mx-auto pc:h-auto pc:max-w-[1100px] pc:px-6 pc:py-4 pc:text-[15px]/[1.5]">
  <p class="m-0 flex items-center gap-2 px-3.5 pt-2 pb-1.5 text-[12px] whitespace-nowrap text-tenue pc:px-0 pc:pt-0 pc:pb-2.5 pc:text-[13px]">
    <span class="donde min-w-0 flex-1 truncate">{i + 1} / {total} · {q.apartado} · {q.probabilidad}</span>
    <Pomodoro />
    <button class={["ver-nota", accion, notasVisibles ? "border-acento text-acento" : "border-linea text-tenue"]} title="Mostrar u ocultar la nota" onclick={onnotasvisibles}>
      {@render icono("M15.5 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V8.5zM15 3v4a2 2 0 0 0 2 2h4M7 13h7M7 17h4")}
    </button>
    <!-- Cerrar va en la esquina, en rojo, como en la pregunta de test. -->
    <button class={[accion, "border-fallo text-fallo bg-[color-mix(in_srgb,var(--color-fallo)_18%,var(--color-fondo))]!"]} title="Abandonar" onclick={onabandonar}>
      {@render icono("M6 6l12 12M18 6L6 18", 2.5)}
    </button>
  </p>
  <div class="h-[3px] bg-linea"><i class="block h-full bg-acento" style:width="{(100 * (i + 1)) / total}%"></i></div>
  <div class="min-h-0 flex-1 overflow-auto px-3.5 py-3 pc:overflow-visible pc:px-0">
    <p class="mt-1 mb-3.5 text-[16px]/[1.4] font-semibold pc:text-[19px]/[1.5]">{q.enunciado}</p>
    {#if notasVisibles}
      <textarea
        class="nota mb-3 block w-full resize-y rounded-[10px] border-2 border-linea bg-tarjeta p-2.5 text-[13px] pc:text-[14px]"
        rows="2"
        placeholder="La nota de esta pregunta"
        value={nota}
        oninput={(e) => onnota(e.currentTarget.value)}
      ></textarea>
    {/if}
    <!-- Con la solución abierta, el texto queda al lado de la respuesta modelo en el PC, y encima en el móvil. -->
    <div class={["gap-4", solucion && "pc:grid pc:grid-cols-2"]}>
      <div>
        {#if solucion}<p class="m-0 mb-1 text-[12px] text-tenue">Tu respuesta</p>{/if}
        <textarea
          class="respuesta block w-full resize-y rounded-[10px] border-2 border-linea bg-tarjeta p-2.5 text-[14px] pc:text-[15px]"
          rows={solucion ? 8 : 12}
          placeholder="Escribe tu respuesta, o piénsala"
          value={texto}
          oninput={(e) => ontexto(e.currentTarget.value)}
        ></textarea>
      </div>
      {#if solucion}
        <div class="modelo mt-3 pc:mt-0">
          <p class="m-0 mb-1 text-[12px] text-tenue">La respuesta modelo</p>
          <p class="m-0 rounded-[10px] border-2 border-acierto bg-tarjeta p-2.5">{q.modelo}</p>
        </div>
      {/if}
    </div>
    {#if solucion}
      <p class="mt-4 mb-1 text-[12px] text-tenue">Los puntos</p>
      <ul class="puntos m-0 list-none p-0">
        {#each q.puntos as p, k (k)}
          <li class="flex items-start gap-2.5 border-b border-linea py-1.5">
            <span class="flex-1">{p.idea}</span>
            {#if p.pagina}
              {@const pagina = p.pagina}
              <button class="ver-pagina shrink-0 cursor-pointer text-[13px] text-acento underline" onclick={() => onverpagina(pagina)}>pág. {pagina.numero}</button>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
    {#if error}<p class="my-3 text-[12px] text-aviso">{error}</p>{/if}
  </div>
  <div class="flex gap-2 border-t-2 border-linea p-3 pc:border-t-0 pc:px-0">
    {#if solucion}
      {#each calificaciones as [nota, etiqueta, color] (nota)}
        <button class={["min-w-0 flex-1 cursor-pointer rounded-[10px] border-2 bg-tarjeta px-2 py-3.5 text-[15px] font-semibold", color]} onclick={() => oncalificar(nota)}>{etiqueta}</button>
      {/each}
    {:else}
      <button class="flex-1 cursor-pointer rounded-[10px] bg-acento p-3.5 text-[15px] font-semibold text-sobre-acento" onclick={onsolucion}>Ver respuesta modelo</button>
    {/if}
  </div>
</div>

{#snippet icono(d: string, grosor = 2)}
  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width={grosor} stroke-linecap="round" stroke-linejoin="round"><path {d} /></svg>
{/snippet}
