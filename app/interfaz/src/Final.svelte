<script lang="ts">
  import type { NotaDelSimulacro } from "./nucleo/nucleo.js";

  let { simulacro, nota, preguntas, error, onvolver }: {
    simulacro: boolean;
    nota: NotaDelSimulacro | null;
    preguntas: number;
    error: string;
    onvolver: () => void;
  } = $props();
</script>

<main class="mx-auto max-w-[560px] p-4">
  {#if simulacro}
    <h1>El simulacro ha terminado</h1>
    {#if nota}
      <p>Aciertos: {nota.aciertos} · Fallos: {nota.fallos} · En blanco: {nota.en_blanco}</p>
      <p>Nota: {nota.nota} sobre 10.</p>
    {/if}
  {:else}
    <h1>La sesión ha terminado</h1>
    {#if nota}<p>Aciertos: {nota.aciertos} de {preguntas}</p>{/if}
  {/if}
  <button onclick={onvolver}>Volver al inicio</button>
  {#if error}<p class="text-mal">{error}</p>{/if}
</main>
