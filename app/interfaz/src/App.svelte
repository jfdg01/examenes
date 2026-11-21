<script lang="ts">
  import type { Inicio } from "./tipos/Inicio";
  import type { PreguntaDeSesion } from "./tipos/PreguntaDeSesion";
  import type { Ronda } from "./tipos/Ronda";

  // La página vive en /<carpeta>/, así que la ruta relativa llega al API de su examen.
  const pedir = <T,>(ruta: string, opciones?: RequestInit): Promise<T> =>
    fetch(ruta, opciones).then((r) => (r.ok ? r.json() : Promise.reject(new Error(`El servidor responde ${r.status}`))));

  const cargarInicio = () =>
    pedir<Inicio>("api/inicio").then((datos) => {
      document.title = datos.titulo;
      return datos;
    });

  let inicio = $state(cargarInicio());
  let ronda = $state<Ronda>({ estado: "Inicio" });
  let preguntas = $state<PreguntaDeSesion[]>([]);
  let i = $state(0);
  let aciertos = $state(0);
  let eleccion = $state<{ opcion: number; seguro: boolean } | null>(null);
  let error = $state("");

  const q = $derived(preguntas[i]);
  const acierto = $derived(eleccion?.opcion === q?.correcta);

  async function empezar() {
    try {
      preguntas = await pedir<PreguntaDeSesion[]>("api/sesion");
    } catch (e) {
      error = (e as Error).message;
      return;
    }
    i = 0;
    aciertos = 0;
    eleccion = null;
    error = "";
    ronda = { estado: "Pregunta", contestada: false };
  }

  async function contestar(opcion: number, seguro: boolean) {
    eleccion = { opcion, seguro };
    ronda = { estado: "Pregunta", contestada: true };
    if (acierto) aciertos++;
    const respuesta = { id: q.id, ts: new Date().toISOString().slice(0, 19) + "Z", acierto, seguro };
    const r = await fetch("api/respuestas", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify([respuesta]),
    }).catch(() => null);
    error = r?.ok ? "" : "La respuesta no llegó al servidor.";
  }

  function siguiente() {
    eleccion = null;
    if (++i < preguntas.length) ronda = { estado: "Pregunta", contestada: false };
    else ronda = { estado: "Final" };
  }

  function volver() {
    inicio = cargarInicio();
    ronda = { estado: "Inicio" };
  }

  function abandonar() {
    if (confirm("¿Abandonar la sesión? Las respuestas quedan en el historial.")) volver();
  }
</script>

{#if ronda.estado === "Inicio"}
  {#await inicio then datos}
    <h1>{datos.titulo}</h1>
    {#if datos.estado === "BancoVacio"}
      <p>El banco no tiene preguntas</p>
    {:else}
      <button onclick={empezar}>Empezar la sesión ({datos.sesion} preguntas)</button>
    {/if}
  {:catch e}
    <p>{e.message}</p>
  {/await}
{:else if ronda.estado === "Pregunta"}
  <p>
    {i + 1} / {preguntas.length} · {q.tema} · {q.paso}
    <button title="Abandonar" onclick={abandonar}>✕</button>
  </p>
  <p>{q.pregunta}</p>
  {#each q.opciones as opcion, k (k)}
    <div>
      <button disabled={ronda.contestada} onclick={() => contestar(k, true)}>{opcion}</button>
      {#if !ronda.contestada}
        <button onclick={() => contestar(k, false)}>dudo</button>
      {:else if k === q.correcta}
        ✓
      {:else if k === eleccion?.opcion}
        ✗
      {/if}
    </div>
  {/each}
  {#if !ronda.contestada}
    <p>Toca el texto si estás seguro. Toca «dudo» si dudas.</p>
  {:else}
    {#if !acierto && eleccion?.seguro}
      <p>Fallo con «seguro»: una idea falsa. Repásala.</p>
    {:else if acierto && !eleccion?.seguro}
      <p>Acierto con «dudo»: cuenta como un fallo.</p>
    {/if}
    <p>{q.explicacion}</p>
    <p>{q.concepto} ({acierto && eleccion?.seguro ? q.paso_tras_seguro : q.paso_tras_otra})</p>
    <button onclick={siguiente}>Siguiente</button>
  {/if}
{:else}
  <h1>La sesión ha terminado</h1>
  <p>Aciertos: {aciertos} de {preguntas.length}</p>
  <button onclick={volver}>Volver al inicio</button>
{/if}
{#if error}
  <p>{error}</p>
{/if}
