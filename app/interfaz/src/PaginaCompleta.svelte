<script lang="ts">
  import type { PreguntaDeSesion } from "./nucleo/nucleo.js";

  let { q, oncerrar }: { q: PreguntaDeSesion; oncerrar: () => void } = $props();
  let zoom = $state(false);
</script>

<div class="pagina fixed inset-0 z-10 flex flex-col bg-white">
  <p class="m-0 flex items-center gap-2 px-3.5 py-2 text-[13px]">
    <span class="flex-1 truncate">{q.fuente}</span>
    <a href={q.pdf} target="_blank">PDF</a>
    <button title="Cerrar" onclick={oncerrar}>✕</button>
  </p>
  <div class="min-h-0 flex-1 overflow-auto">
    <button class={["block border-0 bg-transparent p-0", zoom ? "w-[200%]" : "w-full"]} onclick={() => (zoom = !zoom)}>
      <img class="block w-full" src={q.imagen} alt="La página {q.pagina} de la fuente" />
    </button>
  </div>
  <p class="border-t-2 border-linea px-3.5 pt-2.5 pb-[calc(10px+env(safe-area-inset-bottom))]">Toca la página para ampliarla</p>
</div>
