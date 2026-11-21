<script lang="ts">
  import type { Inicio } from "./tipos/Inicio";

  // La página vive en /<carpeta>/, así que la ruta relativa llega al API de su examen.
  const inicio: Promise<Inicio> = fetch("api/inicio")
    .then((r) => (r.ok ? r.json() : Promise.reject(new Error(`El servidor responde ${r.status}`))))
    .then((datos: Inicio) => {
      document.title = datos.titulo;
      return datos;
    });
</script>

{#await inicio then { titulo, banco_vacio }}
  <h1>{titulo}</h1>
  {#if banco_vacio}
    <p>El banco no tiene preguntas</p>
  {/if}
{:catch error}
  <p>{error.message}</p>
{/await}
