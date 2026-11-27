<script lang="ts">
  import { fly } from "svelte/transition";
  import Cuadricula, { aviso, type Marca } from "./Cuadricula.svelte";
  import type { PreguntaDeDesarrolloDeSesion } from "./nucleo/nucleo.js";

  // Una pregunta de desarrollo del simulacro: el enunciado y el cuadro de texto. El modelo y los puntos salen al final.
  let { q, i, total, restante, texto, marcas, ontexto, onir, onterminar, onabandonar }: {
    q: PreguntaDeDesarrolloDeSesion;
    i: number;
    total: number;
    restante: number;
    texto: string;
    marcas: Marca[];
    ontexto: (texto: string) => void;
    onir: (k: number) => void;
    onterminar: () => void;
    onabandonar: () => void;
  } = $props();

  const flecha = "cursor-pointer rounded-[10px] border-2 border-linea bg-tarjeta whitespace-nowrap px-3 py-2.5 text-[14px] text-texto disabled:cursor-default disabled:opacity-30";
  let cuadricula = $state(false);
  const faltan = $derived(marcas.filter((m) => m === "falta").length);
</script>

<div class="desarrollo-del-simulacro flex h-dvh flex-col text-[14px]/[1.45] pc:mx-auto pc:h-auto pc:max-w-[1100px] pc:px-6 pc:py-4 pc:text-[15px]/[1.5]">
  <p class="m-0 flex items-center gap-2 px-3.5 pt-2 pb-1.5 text-[12px] whitespace-nowrap text-tenue pc:px-0 pc:pt-0 pc:pb-2.5 pc:text-[13px]">
    <span class="donde min-w-0 flex-1 truncate">{Math.floor(restante / 60)}:{String(restante % 60).padStart(2, "0")} · {q.apartado} · desarrollo</span>
    <button
      class="flex h-7 w-8 shrink-0 cursor-pointer items-center justify-center rounded-md border border-fallo bg-[color-mix(in_srgb,var(--color-fallo)_18%,var(--color-fondo))] text-fallo"
      title="Abandonar"
      onclick={onabandonar}
    >
      {@render icono(14, "M6 6l12 12M18 6L6 18", 2.5)}
    </button>
  </p>
  <div class="h-[3px] bg-linea"><i class="block h-full bg-acento" style:width="{(100 * (i + 1)) / total}%"></i></div>
  <div class="min-h-0 flex-1 overflow-auto px-3.5 py-3 pc:overflow-visible pc:px-0">
    <p class="mt-1 mb-3.5 text-[16px]/[1.4] font-semibold pc:text-[19px]/[1.5]">{q.enunciado}</p>
    <textarea
      class="respuesta block w-full resize-y rounded-[10px] border-2 border-linea bg-tarjeta p-2.5 text-[14px] pc:text-[15px]"
      rows="12"
      placeholder="Escribe tu respuesta. Un cuadro vacío cuenta como «No la sabía»."
      value={texto}
      oninput={(e) => ontexto(e.currentTarget.value)}
    ></textarea>
  </div>
  <div class="grid grid-cols-[1fr_auto_1fr] gap-2 border-t-2 border-linea p-3 pb-[calc(12px+env(safe-area-inset-bottom))] pc:border-t-0 pc:px-0">
    <button class={flecha} disabled={i === 0} onclick={() => onir(i - 1)}>‹ Anterior</button>
    <button
      class="flex shrink-0 cursor-pointer items-center justify-center gap-2 rounded-[10px] border-2 border-linea bg-tarjeta px-3 py-2 text-[15px] font-semibold whitespace-nowrap"
      title="Las preguntas"
      onclick={() => (cuadricula = true)}
    >
      {@render icono(18, "M3 3h7v7H3zM14 3h7v7h-7zM3 14h7v7H3zM14 14h7v7h-7z")}
      {i + 1} / {total}
    </button>
    <button class={flecha} disabled={i === total - 1} onclick={() => onir(i + 1)}>Siguiente ›</button>
  </div>
</div>

{#if cuadricula}
  <button class="fixed inset-0 z-10 cursor-default bg-black/50" aria-label="Cerrar" onclick={() => (cuadricula = false)}></button>
  <div
    class="fixed inset-x-0 bottom-0 z-20 mx-auto max-w-[560px] rounded-t-xl bg-fondo p-3.5 pb-[calc(14px+env(safe-area-inset-bottom))] shadow-xl"
    transition:fly={{ y: 300, duration: 200 }}
  >
    <Cuadricula
      {marcas}
      {i}
      onir={(k) => {
        cuadricula = false;
        onir(k);
      }}
    />
    <button
      class="mt-3.5 w-full cursor-pointer rounded-[10px] bg-acento p-3 text-[14px] font-semibold text-sobre-acento disabled:cursor-default disabled:bg-linea disabled:text-tenue"
      disabled={faltan > 0}
      onclick={() => {
        cuadricula = false;
        onterminar();
      }}>Terminar el simulacro</button
    >
    {#if faltan}<p class="mt-2 mb-0 text-center text-[12px] text-aviso">{aviso(faltan)}</p>{/if}
  </div>
{/if}

{#snippet icono(lado: number, d: string, grosor = 2)}
  <svg width={lado} height={lado} viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width={grosor} stroke-linecap="round" stroke-linejoin="round"><path {d} /></svg>
{/snippet}
