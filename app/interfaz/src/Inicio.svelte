<script lang="ts">
  import type { Inicio, Par, TemaDelInicio } from "./nucleo/nucleo.js";

  let { inicio, objetivo, error, onobjetivo, onempezar, onseleccion }: {
    inicio: Inicio | null;
    objetivo: number;
    error: string;
    onobjetivo: (objetivo: number) => void;
    onempezar: (simulacro: boolean) => void;
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
</script>

<main class="mx-auto max-w-[560px] p-4">
  <a class="volver text-[13px] text-tenue" href="/">← Exámenes</a>
  {#if inicio}
    <h1 class="mt-2 mb-1 text-[24px] font-bold">{inicio.titulo}</h1>
    {#if inicio.estado === "BancoVacio"}
      <p class="my-3 rounded-xl bg-tarjeta p-3.5">El banco no tiene preguntas</p>
    {:else}
      {@const t = inicio.tarjeta}
      {@const parte = (n: number) => `${(100 * n) / (t.sabes + t.no_sabes + t.sin_ver)}%`}
      <div class="mt-3.5 flex h-3.5 gap-0.5 overflow-hidden rounded-[7px]">
        <i class="bg-acierto" style:flex={t.sabes}></i>
        <i class="bg-fallo" style:flex={t.no_sabes}></i>
        <i class="bg-linea" style:flex={t.sin_ver}></i>
      </div>
      <p class="mt-1.5 text-[13px] text-tenue">
        Sabes {t.sabes} · No sabes {t.no_sabes} · Sin ver {t.sin_ver}, de {t.sabes + t.no_sabes + t.sin_ver} conceptos{#if inicio.parte}, {inicio.parte}{/if}
      </p>
      <div class="my-3 rounded-xl bg-tarjeta p-3.5">
        <label class="flex items-center gap-3">
          <b class="text-[16px] whitespace-nowrap">Sacar más de {objetivo.toLocaleString("es")}</b>
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
        <details class="group">
          <summary
            class="mt-3.5 grid cursor-pointer list-none grid-cols-[1fr_auto] items-center gap-x-2.5 rounded-lg bg-[color-mix(in_srgb,var(--color-acento)_20%,var(--color-fondo))] px-2.5 py-2 [&::-webkit-details-marker]:hidden"
          >
            <b class="text-acento">{t.consejo}</b>
            <span class="order-1 col-span-full text-[13px] font-bold text-acierto">{t.si_contestas} % si contestas · {t.en_blanco} % en blanco</span>
            <i
              class="grid size-5 shrink-0 place-items-center rounded-full border-[1.5px] border-acento text-[12px] font-bold text-acento not-italic group-open:bg-acento group-open:text-sobre-acento"
              >?</i
            >
          </summary>
          <small class="mt-3.5 block text-[11px] tracking-[1px] text-tenue uppercase">aciertos en una duda</small>
          {#each t.dudas as d, k (d.texto)}
            <!-- La última fila es la media de las otras. -->
            <p
              class={[
                "grid grid-cols-[1fr_auto_3.5em] gap-x-2.5 py-[3px] text-[13px] text-tenue",
                k === t.dudas.length - 1 && "mt-0.5 border-t border-linea pt-1 text-texto",
              ]}
            >
              <span>{d.texto}</span>
              <span class="text-right">{d.conceptos} conceptos</span>
              <b class="text-right">{d.porcentaje} %</b>
              {#if d.detalle}<em class="col-span-full -mt-0.5 text-[11px] not-italic opacity-75">{d.detalle}</em>{/if}
            </p>
          {/each}
        </details>
      </div>
      <fieldset class="my-3 rounded-xl bg-tarjeta p-3.5 text-[14px]">
        {#each inicio.temas as tema (tema.pdf)}
          {@const marcados = tema.apartados.filter((a) => a.marcado).length}
          <label class="tema flex items-center gap-2.5 py-1">
            <input
              class="size-4 accent-acento"
              type="checkbox"
              checked={marcados === tema.apartados.length}
              indeterminate={marcados > 0 && marcados < tema.apartados.length}
              onchange={(e) => marcar(inicio.temas, tema, tema.apartados.map((a) => a.apartado), e.currentTarget.checked)}
            />
            <span class="flex-1 font-semibold">{tema.nombre}</span>
            <span class="text-[13px] text-tenue">{tema.conceptos} conceptos</span>
          </label>
          {#if tema.apartados.length > 1}
            {#each tema.apartados as a (a.apartado)}
              <label class="apartado ml-6.5 flex items-center gap-2.5 py-0.5 text-tenue">
                <input
                  class="size-4 accent-acento"
                  type="checkbox"
                  checked={a.marcado}
                  onchange={(e) => marcar(inicio.temas, tema, [a.apartado], e.currentTarget.checked)}
                />
                <span class="flex-1">{a.nombre}</span>
                <span class="text-[13px]">{a.conceptos}</span>
              </label>
            {/each}
          {/if}
        {/each}
      </fieldset>
      <button
        class="my-3 block w-full cursor-pointer rounded-xl bg-acento p-4 text-[16px] font-semibold text-sobre-acento disabled:cursor-default disabled:opacity-50"
        disabled={inicio.sesion === 0}
        onclick={() => onempezar(false)}>Empezar la sesión ({inicio.sesion} preguntas)</button
      >
      <button
        class="my-3 block w-full cursor-pointer rounded-xl border-2 border-linea bg-tarjeta p-4 text-[16px] font-semibold disabled:cursor-default disabled:opacity-50"
        disabled={inicio.simulacro === 0}
        onclick={() => onempezar(true)}>Simulacro ({inicio.simulacro} preguntas)</button
      >
    {/if}
  {/if}
  {#if error}<p class="my-3 text-aviso">{error}</p>{/if}
</main>

{#snippet cifra(cifra: string, n: number)}
  <div>
    <b class="block text-[26px]/[1.1] text-acento">{cifra}</b>
    <span class="text-[12px] text-tenue">con {n} conceptos</span>
  </div>
{/snippet}
