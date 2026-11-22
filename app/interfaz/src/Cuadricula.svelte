<script lang="ts">
  // PROTOTIPO de navegar-preguntas: un número por pregunta de la ronda.
  // La marca es "falta", "contestada" (sin corrección), "acierto", "fallo" o "blanco".
  let { marcas, i, onir }: { marcas: string[]; i: number; onir: (k: number) => void } = $props();
  const color: Record<string, string> = {
    falta: "border-linea bg-tarjeta text-tenue",
    contestada: "border-acento bg-[color-mix(in_srgb,var(--color-acento)_30%,var(--color-fondo))]",
    acierto: "border-acierto bg-[color-mix(in_srgb,var(--color-acierto)_30%,var(--color-fondo))]",
    fallo: "border-fallo bg-[color-mix(in_srgb,var(--color-fallo)_30%,var(--color-fondo))]",
    blanco: "border-dashed border-tenue bg-tarjeta text-tenue",
  };
</script>

<div class="grid grid-cols-[repeat(auto-fill,minmax(42px,1fr))] gap-1.5">
  {#each marcas as m, k (k)}
    <button
      class={[
        "h-10 cursor-pointer rounded-lg border-2 text-[14px] font-semibold",
        color[m],
        k === i && "outline-2 outline-offset-2 outline-texto",
      ]}
      onclick={() => onir(k)}>{k + 1}</button
    >
  {/each}
</div>
