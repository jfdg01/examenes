<script lang="ts">
  import * as nucleo from "./nucleo/nucleo.js";
  import type { Datos, NotaDelSimulacro, PreguntaDeSesion, Respuesta, Resultado, Ronda } from "./nucleo/nucleo.js";

  // Todos los exámenes comparten un origen, así que la clave lleva la carpeta del examen.
  const claveObjetivo = `${location.pathname.split("/")[1]}-objetivo`;
  let objetivo = $state(Number(localStorage.getItem(claveObjetivo) ?? 5));
  let datos = $state.raw<Datos | null>(null);
  const inicio = $derived(datos && nucleo.inicio(datos, objetivo));

  // La página vive en /<carpeta>/, así que la ruta relativa llega al API de su examen.
  async function cargarInicio() {
    try {
      const r = await fetch("api/inicio");
      if (!r.ok) throw new Error(`El servidor responde ${r.status}`);
      datos = await r.json();
      document.title = datos!.examen.titulo;
    } catch (e) {
      error = (e as Error).message;
    }
  }
  cargarInicio();

  let ronda = $state<Ronda>({ estado: "Inicio" });
  let preguntas = $state.raw<PreguntaDeSesion[]>([]);
  let i = $state(0);
  // `opcion` es null con "En blanco".
  let eleccion = $state<{ opcion: number | null; respuesta: Respuesta } | null>(null);
  let error = $state("");
  let simulacro = $state(false);
  let resultados: Resultado[] = [];
  let nota = $state<NotaDelSimulacro | null>(null);
  let empieza = 0;
  let segundos = 0;
  let reloj: ReturnType<typeof setInterval> | undefined;
  let restante = $state(0);

  const q = $derived(preguntas[i]);
  const acierto = $derived(eleccion?.respuesta.resultado === "Acierto");

  function empezar(comoSimulacro: boolean) {
    if (!datos) return;
    empieza = Date.now();
    if (comoSimulacro) ({ preguntas, segundos } = nucleo.simulacro(datos, empieza));
    else preguntas = nucleo.sesion(datos, empieza);
    simulacro = comoSimulacro;
    i = 0;
    resultados = [];
    nota = null;
    eleccion = null;
    error = "";
    ronda = { estado: "Pregunta", contestada: false };
    if (simulacro) {
      contar();
      reloj = setInterval(contar, 1000);
    }
  }

  function contar() {
    restante = nucleo.restante(empieza, segundos, Date.now());
    if (!restante) terminar();
  }

  // Al pasar el límite, `nucleo` cuenta en blanco las preguntas sin contestar.
  function terminar() {
    // El reloj y la última pregunta llegan aquí a la vez si el límite pasa en la última.
    if (ronda.estado === "Final") return;
    clearInterval(reloj);
    ronda = { estado: "Final" };
    nota = nucleo.nota({ preguntas: preguntas.length, resultados }, datos!.examen.penalizacion);
  }

  async function contestar(opcion: number | null, seguro: boolean) {
    const respuesta = nucleo.corregir(q, opcion, seguro, Date.now());
    eleccion = { opcion, respuesta };
    ronda = { estado: "Pregunta", contestada: true };
    resultados.push(respuesta.resultado);
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
    else terminar();
  }

  function volver() {
    clearInterval(reloj);
    cargarInicio();
    ronda = { estado: "Inicio" };
  }

  function abandonar() {
    const que = simulacro ? "el simulacro" : "la sesión";
    if (confirm(`¿Abandonar ${que}? Las respuestas quedan en el historial.`)) volver();
  }
</script>

{#if ronda.estado === "Inicio"}
  {#if inicio}
    <h1>{inicio.titulo}</h1>
    {#if inicio.estado === "BancoVacio"}
      <p>El banco no tiene preguntas</p>
    {:else}
      {@const t = inicio.tarjeta}
      {@const parte = (n: number) => `${(100 * n) / (t.sabes + t.no_sabes + t.sin_ver)}%`}
      <div class="tramos">
        <i class="hecho" style:flex={t.sabes}></i>
        <i class="mal" style:flex={t.no_sabes}></i>
        <i style:flex={t.sin_ver}></i>
      </div>
      <p>Sabes {t.sabes} · No sabes {t.no_sabes} · Sin ver {t.sin_ver}, de {t.sabes + t.no_sabes + t.sin_ver} conceptos</p>
      <div>
        <label>
          Sacar más de {objetivo.toLocaleString("es")}
          <input type="range" min="0.5" max="10" step="0.5" bind:value={objetivo} oninput={() => localStorage.setItem(claveObjetivo, String(objetivo))} />
        </label>
        <p>
          <b>{t.hoy} %</b> con {t.sabes} conceptos · {t.sesiones}
          {#if t.meta > t.sabes}· <b>95 %</b> con {t.meta} conceptos{/if}
        </p>
        <div class="camino">
          <i class="hecho" style:width={parte(t.sabes)}></i>
          <i class="falta" style:left={parte(t.sabes)} style:width={parte(t.meta - t.sabes)}></i>
          <i class="marca" style:left={parte(t.meta)}></i>
        </div>
        <details>
          <summary><b>{t.consejo}</b>: {t.si_contestas} % si contestas · {t.en_blanco} % en blanco</summary>
          <small>aciertos en una duda</small>
          {#each t.dudas as d (d.texto)}
            <p>
              {d.texto} · {d.conceptos} conceptos · <b>{d.porcentaje} %</b>
              {#if d.detalle}· <em>{d.detalle}</em>{/if}
            </p>
          {/each}
        </details>
      </div>
      <button onclick={() => empezar(false)}>Empezar la sesión ({inicio.sesion} preguntas)</button>
      <button onclick={() => empezar(true)}>Simulacro ({inicio.simulacro} preguntas)</button>
    {/if}
  {/if}
{:else if ronda.estado === "Pregunta"}
  <p>
    {simulacro ? "Simulacro · " : ""}{i + 1} / {preguntas.length} · {q.tema} ·
    {simulacro ? `${Math.floor(restante / 60)}:${String(restante % 60).padStart(2, "0")}` : q.paso}
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
    {#if simulacro}<button onclick={() => contestar(null, false)}>En blanco</button>{/if}
    <p>Toca el texto si estás seguro. Toca «dudo» si dudas.</p>
  {:else}
    {#if eleccion?.respuesta.resultado === "Fallo" && eleccion.respuesta.seguro}
      <p>Fallo con «seguro»: una idea falsa. Repásala.</p>
    {:else if acierto && !eleccion?.respuesta.seguro}
      <p>Acierto con «dudo»: cuenta como un fallo.</p>
    {/if}
    <p>{q.explicacion}</p>
    <p>{q.concepto} ({acierto && eleccion?.respuesta.seguro ? q.paso_tras_seguro : q.paso_tras_otra})</p>
    <button onclick={siguiente}>Siguiente</button>
  {/if}
{:else}
  {#if simulacro}
    <h1>El simulacro ha terminado</h1>
    {#if nota}
      <p>Aciertos: {nota.aciertos} · Fallos: {nota.fallos} · En blanco: {nota.en_blanco}</p>
      <p>Nota: {nota.nota} sobre 10.</p>
    {/if}
  {:else}
    <h1>La sesión ha terminado</h1>
    {#if nota}<p>Aciertos: {nota.aciertos} de {preguntas.length}</p>{/if}
  {/if}
  <button onclick={volver}>Volver al inicio</button>
{/if}
{#if error}
  <p>{error}</p>
{/if}

<style>
  .camino {
    position: relative;
    height: 20px;
    border-radius: 6px;
    background: #ddd;
    overflow: hidden;
  }
  .camino i {
    position: absolute;
    top: 0;
    bottom: 0;
  }
  .tramos {
    display: flex;
    height: 8px;
    border-radius: 4px;
    background: #ddd;
    overflow: hidden;
  }
  .mal {
    background: #c33;
  }
  .hecho {
    background: #3a3;
  }
  .camino .hecho {
    left: 0;
  }
  .falta {
    background: #e93;
  }
  .marca {
    width: 2px;
    margin-left: -1px;
    background: #000;
  }
</style>
