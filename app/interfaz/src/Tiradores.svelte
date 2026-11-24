<script lang="ts">
  import type { Tirador } from "./nucleo/nucleo.js";

  let { onmover, onsoltar }: { onmover: (tirador: Tirador, x: number) => void; onsoltar: () => void } = $props();
</script>

<!-- Cada tirador es una franja de 24 px en el PC. El separador muestra su línea, y el asa sale al pasar el ratón. -->
{#each ["L", "S", "R"] as const as k (k)}
  <div
    class={[
      "tirador hidden pc:fixed pc:inset-y-0 pc:z-9 pc:-ml-3 pc:block pc:w-6 pc:cursor-col-resize pc:touch-none",
      "after:absolute after:inset-y-4 after:left-[11px] after:w-0.5 hover:after:bg-acento active:after:bg-acento",
      k === "S" && "after:bg-linea",
      "before:absolute before:top-1/2 before:left-[5px] before:z-1 before:-mt-5 before:box-border before:hidden before:h-10 before:w-3.5",
      "before:rounded-[7px] before:border-2 before:border-acento hover:before:block active:before:block",
      "before:[background:radial-gradient(circle,var(--color-tenue)_1.5px,transparent_2px)_center/6px_8px_repeat-y,var(--color-tarjeta)]",
    ]}
    role="separator"
    aria-orientation="vertical"
    style:left="var(--{k.toLowerCase()})"
    onpointerdown={(e) => {
      // Sin `preventDefault`, el navegador arrastra el texto seleccionado y cancela el puntero.
      e.preventDefault();
      e.currentTarget.setPointerCapture(e.pointerId);
    }}
    onpointermove={(e) => {
      if (e.currentTarget.hasPointerCapture(e.pointerId)) onmover(k, e.clientX / innerWidth);
    }}
    onlostpointercapture={onsoltar}
  ></div>
{/each}
