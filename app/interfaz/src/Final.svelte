<script lang="ts">
  import type { NotaDeDesarrollo, NotaDelSimulacro, Parte, PreguntaDeDesarrolloDeSesion } from "./nucleo/nucleo.js";
  import Cuadricula, { type Marca } from "./Cuadricula.svelte";
  import Pomodoro from "./Pomodoro.svelte";

  let { simulacro, nota, preguntas, desarrollo, error, onvolver, marcas, onrevisar, oncorregir }: {
    // La cuadrícula corregida del test: cada número abre su pregunta.
    marcas: Marca[];
    onrevisar: (k: number) => void;
    simulacro: boolean;
    nota: NotaDelSimulacro | null;
    preguntas: number;
    // Las preguntas de desarrollo del simulacro, con su texto y su nota. Sin nota, la respuesta está sin corregir.
    desarrollo: { q: PreguntaDeDesarrolloDeSesion; texto: string; nota: NotaDeDesarrollo | undefined }[];
    error: string;
    onvolver: () => void;
    oncorregir: (k: number, nota: NotaDeDesarrollo) => void;
  } = $props();
  const tarjeta = "my-3 rounded-xl bg-tarjeta p-3.5";
  const calificaciones: [NotaDeDesarrollo, string, string][] = [
    ["Again", "No la sabía", "border-fallo text-fallo"],
    ["Hard", "A medias", "border-aviso text-aviso"],
    ["Good", "La sabía", "border-acierto text-acierto"],
  ];
  const parte = (p: Parte) => `${p.nota} sobre ${p.sobre}`;
</script>

<main class={["mx-auto p-4", desarrollo.length ? "max-w-[1100px]" : "max-w-[560px]"]}>
  <div class="flex items-start justify-between gap-3">
    <h1 class="mt-2 mb-1 text-[24px] font-bold">{simulacro ? "El simulacro ha terminado" : "La sesión ha terminado"}</h1>
    <Pomodoro />
  </div>
  {#if nota}
    {#if simulacro}
      <p class={tarjeta}>Aciertos: {nota.aciertos} · Fallos: {nota.fallos} · En blanco: {nota.en_blanco}</p>
      {#if nota.desarrollo && nota.total}
        <p class={["nota-del-simulacro", tarjeta]}>
          Test: {parte(nota.test)} · Desarrollo: {nota.desarrollo.nota ? parte(nota.desarrollo) : "sin corregir"}<br />
          <b>Nota: {nota.total.nota ? parte(nota.total) : "desarrollo sin corregir"}</b>
        </p>
      {:else}
        <p class={["nota-del-simulacro", tarjeta]}>Nota: {parte(nota.test)}.</p>
      {/if}
    {:else}
      <p class={tarjeta}>Aciertos: {nota.aciertos} de {preguntas}</p>
    {/if}
  {/if}
  {#each desarrollo as d, k (d.q.id)}
    <section class={["correccion", tarjeta]}>
      <p class="m-0 mb-2.5 font-semibold">{k + 1}. {d.q.enunciado}</p>
      <!-- El texto queda al lado de la respuesta modelo en el PC, y encima en el móvil. -->
      <div class="gap-4 pc:grid pc:grid-cols-2">
        <div>
          <p class="m-0 mb-1 text-[12px] text-tenue">Tu respuesta</p>
          <p class="m-0 rounded-[10px] border-2 border-linea p-2.5 whitespace-pre-wrap">{d.texto.trim() || "En blanco"}</p>
        </div>
        <div class="mt-3 pc:mt-0">
          <p class="m-0 mb-1 text-[12px] text-tenue">La respuesta modelo</p>
          <p class="m-0 rounded-[10px] border-2 border-acierto p-2.5">{d.q.modelo}</p>
        </div>
      </div>
      <p class="mt-3 mb-1 text-[12px] text-tenue">Los puntos</p>
      <ul class="m-0 list-none p-0">
        {#each d.q.puntos as p, n (n)}<li class="border-b border-linea py-1.5">{p.idea}</li>{/each}
      </ul>
      {#if d.nota}
        {@const [, etiqueta, color] = calificaciones.find(([n]) => n === d.nota)!}
        <p class={["m-0 mt-3 font-semibold", color.split(" ")[1]]}>{etiqueta}</p>
      {:else}
        <!-- Sin un clic, la respuesta queda sin corregir, y `banco pendientes` la lista. -->
        <div class="mt-3 flex gap-2">
          {#each calificaciones as [n, etiqueta, color] (n)}
            <button class={["min-w-0 flex-1 cursor-pointer rounded-[10px] border-2 bg-tarjeta px-2 py-3 text-[15px] font-semibold", color]} onclick={() => oncorregir(k, n)}
              >{etiqueta}</button
            >
          {/each}
        </div>
      {/if}
    </section>
  {/each}
  <div class="mx-auto max-w-[560px]">
    <p class="mt-4 mb-2 text-[13px] text-tenue">Toca un número para ver su pregunta corregida.</p>
    <Cuadricula {marcas} i={-1} onir={onrevisar} />
    <button class="my-3 block w-full cursor-pointer rounded-xl bg-acento p-4 text-[16px] font-semibold text-sobre-acento" onclick={onvolver}
      >Volver al inicio</button
    >
    {#if error}<p class="my-3 text-aviso">{error}</p>{/if}
  </div>
</main>
