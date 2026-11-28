<script lang="ts">
  import Pomodoro from "./Pomodoro.svelte";
  import type { CifraDeDesarrollo, Inicio, Par, TemaDelInicio } from "./nucleo/nucleo.js";

  let { inicio, desarrollo, objetivo, error, onobjetivo, onempezar, ondesarrollo, onseleccion }: {
    inicio: Inicio | null;
    // Sin banco de desarrollo, `undefined`, y el botón no sale.
    desarrollo: CifraDeDesarrollo | undefined;
    objetivo: number;
    error: string;
    onobjetivo: (objetivo: number) => void;
    onempezar: (simulacro: boolean) => void;
    ondesarrollo: () => void;
    onseleccion: (seleccion: Par[]) => void;
  } = $props();

  // Los pares marcados, con los `apartados` del tema `t` en `marcado`.
  function marcar(temas: TemaDelInicio[], t: TemaDelInicio, apartados: string[], marcado: boolean) {
    onseleccion(
      temas.flatMap((u) =>
        u.apartados
          .filter((a) => (u.pdf === t.pdf && apartados.includes(a.apartado) ? marcado : a.marcado))
          .map((a) => ({ pdf: u.pdf, apartado: a.apartado })),
      ),
    );
  }

  // La tarjeta de los temas, plegada o no, se recuerda entre visitas. Un tema empieza con sus apartados plegados.
  const clavePlegados = "temas-plegados";
  let plegados = $state(localStorage.getItem(clavePlegados) === "si");
  function plegar() {
    plegados = !plegados;
    localStorage.setItem(clavePlegados, plegados ? "si" : "no");
  }
  let abiertos = $state<string[]>([]);
  const abrir = (pdf: string) => (abiertos = abiertos.includes(pdf) ? abiertos.filter((p) => p !== pdf) : [...abiertos, pdf]);

  // Las preguntas de desarrollo que sabes y las que hay en los `apartados` del tema `pdf`, o "" si no hay ninguna.
  function deDesarrollo(pdf: string, apartados: string[]): string {
    const pares = desarrollo?.pares.filter((p) => p.pdf === pdf && apartados.includes(p.apartado)) ?? [];
    const hay = pares.reduce((n, p) => n + p.hay, 0);
    return hay ? ` · ${pares.reduce((n, p) => n + p.sabes, 0)}/${hay} desarrollo` : "";
  }
</script>

<!-- Como en el test, los botones de empezar van en un pie fijo, y lo de arriba corre por encima. -->
<main class="mx-auto flex h-dvh max-w-[560px] flex-col">
  <div class="min-h-0 flex-1 overflow-auto p-4">
    <p class="flex items-center gap-3 text-[13px] text-tenue"><a class="volver mr-auto" href="/">← Exámenes</a><Pomodoro /></p>
    {#if inicio}
      <h1 class="mt-2 mb-1 text-[24px] font-bold">{inicio.titulo}</h1>
      {#if inicio.estado === "BancoVacio"}
        <p class="my-3 rounded-xl bg-tarjeta p-3.5">El banco no tiene preguntas</p>
      {:else}
        {@const t = inicio.tarjeta}
        {@const parte = (n: number) => `${(100 * n) / (t.sabes + t.no_sabes + t.sin_ver)}%`}
        <div class="mt-3.5 flex h-5 overflow-hidden rounded-md">
          <i class="bg-acierto" style:flex={t.sabes}></i>
          <i class="bg-aviso" style:flex={t.no_sabes}></i>
          <i class="bg-linea" style:flex={t.sin_ver}></i>
        </div>
        <p class="mt-1.5 text-[13px] text-tenue">
          Sabes {t.sabes} · No sabes {t.no_sabes} · Sin ver {t.sin_ver}, de {t.sabes + t.no_sabes + t.sin_ver} preguntas{#if inicio.parte}, {inicio.parte}{/if}
        </p>
        {#if desarrollo?.hay}
          <p class="cifra-desarrollo mt-0.5 text-[13px] text-tenue">Desarrollo: sabes {desarrollo.sabes} de {desarrollo.hay}</p>
        {/if}
        <div class="my-3 rounded-xl bg-tarjeta p-3.5">
          <label class="flex items-center gap-3">
            <b class="text-[16px] whitespace-nowrap">Sacar más de <span class="inline-block w-[3ch] tabular-nums">{objetivo.toLocaleString("es")}</span></b>
            <input
              class="min-w-0 flex-1 accent-acento"
              type="range"
              min="0.5"
              max="10"
              step="0.5"
              value={objetivo}
              oninput={(e) => onobjetivo(Number(e.currentTarget.value))}
            />
          </label>
          <div class="mt-3.5 grid grid-cols-[auto_1fr_auto] items-center gap-2.5 text-center">
            {@render cifra(`${t.hoy} %`, t.sabes)}
            <div>
              <span class="text-[14px] font-bold text-aviso">{t.sesiones}</span>
              <!-- La flecha: una línea con su punta a la derecha. -->
              <i
                class="relative mt-1 block h-0.5 bg-aviso after:absolute after:-top-[5px] after:-right-0.5 after:border-[6px] after:border-r-0 after:border-transparent after:border-l-aviso"
              ></i>
            </div>
            {#if t.meta > t.sabes}{@render cifra("95 %", t.meta)}{:else}<div></div>{/if}
          </div>
          <div class="relative mt-3.5 h-5 overflow-hidden rounded-md bg-linea">
            <i class="absolute inset-y-0 left-0 bg-acierto" style:width={parte(t.sabes)}></i>
            <i class="absolute inset-y-0 bg-aviso" style:left={parte(t.sabes)} style:width={parte(t.meta - t.sabes)}></i>
            <i class="absolute inset-y-0 -ml-px w-0.5 bg-texto" style:left={parte(t.meta)}></i>
          </div>
        </div>
        {@const enteros = inicio.temas.filter((u) => u.apartados.every((a) => a.marcado)).length}
        <fieldset class="my-3 rounded-xl bg-tarjeta p-3.5 text-[14px]">
          <button class="plegar-temas flex w-full cursor-pointer items-center gap-2.5 text-left" aria-expanded={!plegados} onclick={plegar}>
            <b class="flex-1">Temas</b>
            <span class="text-[13px] text-tenue tabular-nums">{enteros} de {inicio.temas.length} marcados</span>
            {@render flecha(!plegados)}
          </button>
          {#each plegados ? [] : inicio.temas as tema (tema.pdf)}
            {@const marcados = tema.apartados.filter((a) => a.marcado).length}
            <label class="tema flex items-center gap-2.5 py-1.5">
              <input
                class="size-4 accent-acento"
                type="checkbox"
                checked={marcados === tema.apartados.length}
                indeterminate={marcados > 0 && marcados < tema.apartados.length}
                onchange={(e) => marcar(inicio.temas, tema, tema.apartados.map((a) => a.apartado), e.currentTarget.checked)}
              />
              <!-- El espacio duro tras el número deja "1." en la línea de su primera palabra. -->
              <span class="min-w-0 flex-1">
                <span class="block font-semibold">{tema.nombre.replace(/^(\S+\.) /, "$1\u00a0")}</span>
                <span class="block text-[13px] text-tenue tabular-nums">{tema.sabes}/{tema.preguntas} preguntas{deDesarrollo(
                    tema.pdf,
                    tema.apartados.map((a) => a.apartado),
                  )}</span>
              </span>
              <a class="self-start text-[13px] text-acento underline" href="pdfs/{tema.pdf}" target="_blank">PDF</a>
              {#if tema.apartados.length > 1}
                <button
                  class="abrir-tema cursor-pointer self-start"
                  title="Apartados"
                  aria-expanded={abiertos.includes(tema.pdf)}
                  onclick={(e) => {
                    e.preventDefault();
                    abrir(tema.pdf);
                  }}>{@render flecha(abiertos.includes(tema.pdf))}</button
                >
              {/if}
            </label>
            {#if tema.apartados.length > 1 && abiertos.includes(tema.pdf)}
              {#each tema.apartados as a (a.apartado)}
                <label class="apartado ml-6.5 flex items-center gap-2.5 py-0.5 text-tenue">
                  <input
                    class="size-4 accent-acento"
                    type="checkbox"
                    checked={a.marcado}
                    onchange={(e) => marcar(inicio.temas, tema, [a.apartado], e.currentTarget.checked)}
                  />
                  <span class="min-w-0 flex-1">
                    <span class="block">{a.nombre}</span>
                    <span class="block text-[13px] tabular-nums">{a.sabes}/{a.preguntas} preguntas{deDesarrollo(tema.pdf, [a.apartado])}</span>
                  </span>
                </label>
              {/each}
            {/if}
          {/each}
        </fieldset>
      {/if}
    {/if}
    {#if error}<p class="my-3 text-aviso">{error}</p>{/if}
  </div>
  {#if inicio && inicio.estado !== "BancoVacio"}
    <div class="pie flex flex-col gap-2.5 border-t-2 border-linea px-4 pt-3 pb-[calc(12px+env(safe-area-inset-bottom))]">
      <!-- Un aviso sin bloqueo: `banco pendientes` lista las respuestas, y el agente las corrige. -->
      {#if desarrollo?.sin_corregir}
        <p class="sin-corregir m-0 text-[13px] text-aviso">
          {desarrollo.sin_corregir === 1 ? "1 respuesta sin corregir" : `${desarrollo.sin_corregir} respuestas sin corregir`}
        </p>
      {/if}
      <div class="flex gap-2.5">
        <button
          class="flex-1 cursor-pointer rounded-xl bg-acento p-4 text-[16px] font-semibold text-sobre-acento disabled:cursor-default disabled:opacity-50"
          disabled={inicio.sesion === 0}
          onclick={() => onempezar(false)}>Tipo test ({inicio.sesion})</button
        >
        {#if desarrollo}
          <button
            class="desarrollo flex-1 cursor-pointer rounded-xl bg-acento p-4 text-[16px] font-semibold text-sobre-acento disabled:cursor-default disabled:opacity-50"
            disabled={desarrollo.sesion === 0}
            onclick={ondesarrollo}>Desarrollo ({desarrollo.sesion})</button
          >
        {/if}
      </div>
      <button
        class="block w-full cursor-pointer rounded-xl border-2 border-linea bg-tarjeta p-4 text-[16px] font-semibold disabled:cursor-default disabled:opacity-50"
        disabled={inicio.simulacro === 0}
        onclick={() => onempezar(true)}
        >Simulacro ({#if desarrollo?.simulacro}{desarrollo.simulacro} de desarrollo y {inicio.simulacro} de test{:else}{inicio.simulacro} preguntas{/if})</button
      >
    </div>
  {/if}
</main>

{#snippet flecha(abierta: boolean)}
  <i class={["block w-4 text-center text-tenue not-italic transition-transform", abierta && "rotate-90"]}>›</i>
{/snippet}

{#snippet cifra(cifra: string, n: number)}
  <div>
    <b class="block text-[26px]/[1.1] text-acento">{cifra}</b>
    <span class="text-[12px] text-tenue">con {n} preguntas</span>
  </div>
{/snippet}
