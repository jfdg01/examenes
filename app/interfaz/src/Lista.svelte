<script lang="ts">
  import type { ExamenDeLaLista } from "./nucleo/nucleo.js";

  let examenes = $state.raw<ExamenDeLaLista[] | null>(null);
  let error = $state("");
  fetch("/api/examenes")
    .then((r) => (r.ok ? r.json() : Promise.reject(new Error(`El servidor responde ${r.status}`))))
    .then((e) => (examenes = e))
    .catch((e) => (error = e.message));
</script>

<main class="mx-auto max-w-[560px] p-4">
  <a class="text-[13px] text-tenue" href="/?plan">Plan de estudio →</a>
  <h1 class="mt-2 mb-1 text-[24px] font-bold">Exámenes</h1>
  {#if examenes?.length === 0}
    <p class="my-3 rounded-xl bg-tarjeta p-3.5">No hay exámenes</p>
  {/if}
  {#each examenes ?? [] as e (e.carpeta)}
    <a class="examen my-3 block rounded-xl bg-tarjeta p-4 text-[16px] font-semibold" href="/{e.carpeta}/">{e.titulo}</a>
  {/each}
  {#if error}<p class="my-3 text-aviso">{error}</p>{/if}
</main>
