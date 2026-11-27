<script lang="ts">
  import type { Pagina } from "./nucleo/nucleo.js";

  let { pagina, oncerrar }: { pagina: Pagina; oncerrar: () => void } = $props();
  let zoom = $state(false);
</script>

<div class="pagina fixed inset-0 z-10 flex flex-col bg-fondo">
  <p class="m-0 flex items-center gap-3 border-b-2 border-linea px-3.5 py-2.5 text-[12px]">
    <span class="flex-1 truncate text-tenue">{pagina.fuente}</span>
    <a class="text-acento underline" href={pagina.pdf} target="_blank">PDF</a>
    <button class="cursor-pointer rounded-md border border-linea bg-tarjeta px-2.5 py-[3px]" title="Cerrar" onclick={oncerrar}>✕</button>
  </p>
  <div class="min-h-0 flex-1 overflow-auto bg-white">
    <button class={["block cursor-pointer", zoom ? "w-[200%]" : "w-full"]} onclick={() => (zoom = !zoom)}>
      <img class="block w-full" src={pagina.imagen} alt="La página {pagina.numero} de la fuente" />
    </button>
  </div>
  <p class="m-0 border-t-2 border-linea p-1.5 text-center text-[11px] text-tenue">Toca la página para ampliarla</p>
</div>
