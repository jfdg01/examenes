<script lang="ts">
  import type { Inicio } from "./nucleo/nucleo.js";

  let { inicio, objetivo, error, onobjetivo, onempezar }: {
    inicio: Inicio | null;
    objetivo: number;
    error: string;
    onobjetivo: (objetivo: number) => void;
    onempezar: (simulacro: boolean) => void;
  } = $props();
</script>

<main class="mx-auto max-w-[560px] p-4">
  {#if inicio}
    <h1>{inicio.titulo}</h1>
    {#if inicio.estado === "BancoVacio"}
      <p>El banco no tiene preguntas</p>
    {:else}
      {@const t = inicio.tarjeta}
      {@const parte = (n: number) => `${(100 * n) / (t.sabes + t.no_sabes + t.sin_ver)}%`}
      <div class="flex h-2 overflow-hidden rounded-[4px] bg-linea">
        <i class="bg-hecho" style:flex={t.sabes}></i>
        <i class="bg-mal" style:flex={t.no_sabes}></i>
        <i style:flex={t.sin_ver}></i>
      </div>
      <p>Sabes {t.sabes} · No sabes {t.no_sabes} · Sin ver {t.sin_ver}, de {t.sabes + t.no_sabes + t.sin_ver} conceptos</p>
      <div>
        <label>
          Sacar más de {objetivo.toLocaleString("es")}
          <input type="range" min="0.5" max="10" step="0.5" value={objetivo} oninput={(e) => onobjetivo(Number(e.currentTarget.value))} />
        </label>
        <p>
          <b>{t.hoy} %</b> con {t.sabes} conceptos · {t.sesiones}
          {#if t.meta > t.sabes}· <b>95 %</b> con {t.meta} conceptos{/if}
        </p>
        <div class="relative h-5 overflow-hidden rounded-md bg-linea">
          <i class="absolute inset-y-0 left-0 bg-hecho" style:width={parte(t.sabes)}></i>
          <i class="absolute inset-y-0 bg-falta" style:left={parte(t.sabes)} style:width={parte(t.meta - t.sabes)}></i>
          <i class="absolute inset-y-0 -ml-px w-0.5 bg-black" style:left={parte(t.meta)}></i>
        </div>
        <details>
          <summary><b>{t.consejo}</b>: {t.si_contestas} % si contestas · {t.en_blanco} % en blanco</summary>
          <small>aciertos en una duda</small>
          {#each t.dudas as d (d.texto)}
            <p>
              {d.texto} · {d.conceptos} conceptos · <b>{d.porcentaje} %</b>
              {#if d.detalle}· <em>{d.detalle}</em>{/if}
            </p>
          {/each}
        </details>
      </div>
      <button onclick={() => onempezar(false)}>Empezar la sesión ({inicio.sesion} preguntas)</button>
      <button onclick={() => onempezar(true)}>Simulacro ({inicio.simulacro} preguntas)</button>
    {/if}
  {/if}
  {#if error}<p class="text-mal">{error}</p>{/if}
</main>
