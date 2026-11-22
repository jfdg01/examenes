<script lang="ts">
  import type { NotaDelSimulacro } from "./nucleo/nucleo.js";

  let { simulacro, nota, preguntas, error, onvolver }: {
    simulacro: boolean;
    nota: NotaDelSimulacro | null;
    preguntas: number;
    error: string;
    onvolver: () => void;
  } = $props();
  const tarjeta = "my-3 rounded-xl bg-tarjeta p-3.5";
</script>

<main class="mx-auto max-w-[560px] p-4">
  <h1 class="mt-2 mb-1 text-[24px] font-bold">{simulacro ? "El simulacro ha terminado" : "La sesión ha terminado"}</h1>
  {#if nota}
    {#if simulacro}
      <p class={tarjeta}>Aciertos: {nota.aciertos} · Fallos: {nota.fallos} · En blanco: {nota.en_blanco}</p>
      <p class={tarjeta}>Nota: {nota.nota} sobre 10.</p>
    {:else}
      <p class={tarjeta}>Aciertos: {nota.aciertos} de {preguntas}</p>
    {/if}
  {/if}
  <button class="my-3 block w-full cursor-pointer rounded-xl bg-acento p-4 text-[16px] font-semibold text-sobre-acento" onclick={onvolver}
    >Volver al inicio</button
  >
  {#if error}<p class="my-3 text-aviso">{error}</p>{/if}
</main>
