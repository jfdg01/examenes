<script lang="ts">
  import type { Cierre } from "./tipos/Cierre";
  import type { Inicio } from "./tipos/Inicio";
  import type { NotaDelSimulacro } from "./tipos/NotaDelSimulacro";
  import type { Resultado } from "./tipos/Resultado";
  import type { Simulacro } from "./tipos/Simulacro";
  import type { PreguntaDeSesion } from "./tipos/PreguntaDeSesion";
  import type { Ronda } from "./tipos/Ronda";

  // La página vive en /<carpeta>/, así que la ruta relativa llega al API de su examen.
  const pedir = <T,>(ruta: string, opciones?: RequestInit): Promise<T> =>
    fetch(ruta, opciones).then((r) => (r.ok ? r.json() : Promise.reject(new Error(`El servidor responde ${r.status}`))));

  // Todos los exámenes comparten un origen, así que la clave lleva la carpeta del examen.
  const claveObjetivo = `${location.pathname.split("/")[1]}-objetivo`;
  let objetivo = $state(Number(localStorage.getItem(claveObjetivo) ?? 5));
  let inicio = $state<Inicio | null>(null);

  async function cargarInicio() {
    const pedido = objetivo;
    try {
      const datos = await pedir<Inicio>(`api/inicio?objetivo=${pedido}`);
      // Con el slider en marcha, una respuesta vieja llega tarde y no pinta.
      if (pedido !== objetivo) return;
      inicio = datos;
      document.title = datos.titulo;
    } catch (e) {
      error = (e as Error).message;
    }
  }
  cargarInicio();

  function cambiarObjetivo() {
    localStorage.setItem(claveObjetivo, String(objetivo));
    cargarInicio();
  }
  let ronda = $state<Ronda>({ estado: "Inicio" });
  let preguntas = $state<PreguntaDeSesion[]>([]);
  let i = $state(0);
  let aciertos = $state(0);
  // `opcion` es null con "En blanco".
  let eleccion = $state<{ opcion: number | null; seguro: boolean } | null>(null);
  let error = $state("");
  let simulacro = $state(false);
  let resultados: Resultado[] = [];
  let nota = $state<NotaDelSimulacro | null>(null);
  let limite = 0;
  let reloj: ReturnType<typeof setInterval> | undefined;
  let restante = $state(0);

  const q = $derived(preguntas[i]);
  const acierto = $derived(eleccion?.opcion === q?.correcta);

  async function empezar(comoSimulacro: boolean) {
    try {
      if (comoSimulacro) {
        const s = await pedir<Simulacro>("api/simulacro");
        preguntas = s.preguntas;
        limite = Date.now() + s.segundos * 1000;
      } else preguntas = await pedir<PreguntaDeSesion[]>("api/sesion");
    } catch (e) {
      error = (e as Error).message;
      return;
    }
    simulacro = comoSimulacro;
    i = 0;
    aciertos = 0;
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
    restante = Math.max(0, Math.round((limite - Date.now()) / 1000));
    if (!restante) terminar();
  }

  // El final del simulacro pide la nota a Rust, que cuenta en blanco las preguntas sin contestar.
  async function terminar() {
    // El reloj y la última pregunta llegan aquí a la vez si el límite pasa en la última.
    if (ronda.estado === "Final") return;
    clearInterval(reloj);
    ronda = { estado: "Final" };
    if (!simulacro) return;
    const cierre: Cierre = { preguntas: preguntas.length, resultados };
    try {
      nota = await pedir<NotaDelSimulacro>("api/cierre", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(cierre),
      });
    } catch (e) {
      error = (e as Error).message;
    }
  }

  async function contestar(opcion: number | null, seguro: boolean) {
    eleccion = { opcion, seguro };
    ronda = { estado: "Pregunta", contestada: true };
    if (acierto) aciertos++;
    resultados.push(opcion === null ? "Blanco" : acierto ? "Acierto" : "Fallo");
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
          <input type="range" min="0.5" max="10" step="0.5" bind:value={objetivo} oninput={cambiarObjetivo} />
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
  {#if simulacro}
    <h1>El simulacro ha terminado</h1>
    {#if nota}
      <p>Aciertos: {nota.aciertos} · Fallos: {nota.fallos} · En blanco: {nota.en_blanco}</p>
      <p>Nota: {nota.nota} sobre 10.</p>
    {/if}
  {:else}
    <h1>La sesión ha terminado</h1>
    <p>Aciertos: {aciertos} de {preguntas.length}</p>
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
