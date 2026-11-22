<script lang="ts" module>
  // La marca de una pregunta de la ronda. "contestada" es una respuesta del simulacro, sin corregir hasta el final.
  export type Marca = "falta" | "contestada" | "acierto" | "fallo" | "blanco";
  export const aviso = (n: number) => `Te ${n === 1 ? "falta 1 pregunta" : `faltan ${n} preguntas`}. Contéstala${n === 1 ? "" : "s"} para terminar.`;
</script>

<script lang="ts">
  // Un número por pregunta de la ronda. Sin pregunta actual, `i` es -1.
  let { marcas, i, onir }: { marcas: Marca[]; i: number; onir: (k: number) => void } = $props();
  const color: Record<Marca, string> = {
    falta: "border-linea bg-tarjeta text-tenue",
    contestada: "border-acento bg-[color-mix(in_srgb,var(--color-acento)_30%,var(--color-fondo))]",
    acierto: "border-acierto bg-[color-mix(in_srgb,var(--color-acierto)_30%,var(--color-fondo))]",
    fallo: "border-fallo bg-[color-mix(in_srgb,var(--color-fallo)_30%,var(--color-fondo))]",
    blanco: "border-dashed border-tenue bg-tarjeta text-tenue",
  };
</script>

<!-- Tantas columnas como filas, más o menos: 12 son 4 × 3, y 20 son 5 × 4. -->
<div class="cuadricula mx-auto grid w-fit gap-2" style:grid-template-columns="repeat({Math.ceil(Math.sqrt(marcas.length))}, 3.25rem)">
  {#each marcas as m, k (k)}
    <button
      class={[
        "aspect-square cursor-pointer rounded-xl border-2 text-[16px] font-semibold",
        color[m],
        k === i && "outline-2 outline-offset-2 outline-texto",
      ]}
      onclick={() => onir(k)}>{k + 1}</button
    >
  {/each}
</div>
