<script lang="ts">
  import * as nucleo from "./nucleo/nucleo.js";
  import type { Datos, Guardado, NotaDelSimulacro, PreguntaDeSesion, Respuesta, Resultado, Ronda } from "./nucleo/nucleo.js";

  // Todos los exámenes comparten un origen, así que cada clave lleva la carpeta del examen.
  const carpeta = location.pathname.split("/")[1];
  const claveObjetivo = `${carpeta}-objetivo`;
  const claveRonda = `${carpeta}-ronda`;
  const clavePendientes = `${carpeta}-pendientes`;
  let objetivo = $state(Number(localStorage.getItem(claveObjetivo) ?? 5));
  let datos = $state.raw<Datos | null>(null);
  let error = $state("");
  const inicio = $derived(datos && nucleo.inicio(datos, objetivo));

  // Una respuesta sin red espera en `localStorage` hasta que llega al servidor.
  const pendientes = (): Respuesta[] => JSON.parse(localStorage.getItem(clavePendientes) ?? "[]");

  // Los envíos van en cola, y una respuesta nueva solo entra al final de las pendientes.
  // Así las enviadas son las primeras. El error es "" cuando todas llegaron.
  let cola = Promise.resolve("");
  function enviarPendientes() {
    return (cola = cola.then(async () => {
      const enviando = pendientes();
      if (!enviando.length) return "";
      const r = await fetch("api/respuestas", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(enviando),
      }).catch(() => null);
      if (!r) return "Sin red: la respuesta se envía al volver la red.";
      if (!r.ok) return `El servidor responde ${r.status}, y la respuesta sigue en este dispositivo.`;
      localStorage.setItem(clavePendientes, JSON.stringify(pendientes().slice(enviando.length)));
      return "";
    }));
  }
  // Con la red de vuelta, las pendientes salen y el historial llega con las respuestas de otros dispositivos.
  addEventListener("online", cargarInicio);

  // La página vive en /<carpeta>/, así que la ruta relativa llega al API de su examen.
  async function cargarInicio() {
    error = await enviarPendientes();
    try {
      const r = await fetch("api/inicio");
      if (!r.ok) throw new Error(`El servidor responde ${r.status}`);
      const d: Datos = await r.json();
      // Una respuesta que no llegó al servidor sigue en el historial de la pantalla.
      datos = { ...d, historial: nucleo.unir(d.historial, pendientes()) };
      document.title = datos.examen.titulo;
    } catch (e) {
      error = datos ? "Sin red: el inicio usa el historial de este dispositivo." : (e as Error).message;
    }
  }

  // Al cargar, gana el guardado más nuevo de `localStorage` y del servidor, y los 2 quedan con él.
  async function arrancar() {
    await cargarInicio();
    if (!datos) return;
    const sinRonda: Guardado = { guardada: 0, ronda: undefined };
    const servidor: Guardado = await fetch("api/ronda")
      .then((r) => (r.ok ? r.json() : sinRonda))
      .catch(() => sinRonda);
    // Un clic en "Empezar" antes de la respuesta ya empezó una ronda nueva, que gana.
    if (ronda.estado !== "Inicio") return;
    const g = nucleo.gana(servidor, JSON.parse(localStorage.getItem(claveRonda) ?? "null") ?? sinRonda);
    enviar(g);
    const r = nucleo.restaurar(datos, g, Date.now());
    if (!r) return;
    ({ preguntas, resultados, simulacro } = r);
    limite = r.limite ?? 0;
    seguir();
  }
  arrancar();

  function enviar(g: Guardado) {
    localStorage.setItem(claveRonda, JSON.stringify(g));
    fetch("api/ronda", { method: "PUT", headers: { "Content-Type": "application/json" }, body: JSON.stringify(g) }).catch(() => {});
  }

  // La ronda a medias, o ninguna en el inicio y el final.
  function guardar() {
    const aMedias = ronda.estado === "Pregunta";
    const ids = preguntas.map((q) => q.id);
    enviar({ guardada: Date.now(), ronda: aMedias ? { ids, resultados, simulacro, limite: simulacro ? limite : undefined } : undefined });
  }

  let ronda = $state<Ronda>({ estado: "Inicio" });
  let preguntas = $state.raw<PreguntaDeSesion[]>([]);
  let i = $state(0);
  // `opcion` es null con "En blanco".
  let eleccion = $state<{ opcion: number | null; respuesta: Respuesta } | null>(null);
  let simulacro = $state(false);
  let resultados: Resultado[] = [];
  let nota = $state<NotaDelSimulacro | null>(null);
  let limite = 0;
  let reloj: ReturnType<typeof setInterval> | undefined;
  let restante = $state(0);

  const q = $derived(preguntas[i]);
  const acierto = $derived(eleccion?.respuesta.resultado === "Acierto");

  function empezar(comoSimulacro: boolean) {
    if (!datos) return;
    const ahora = Date.now();
    if (comoSimulacro) {
      const s = nucleo.simulacro(datos, ahora);
      preguntas = s.preguntas;
      limite = ahora + s.segundos * 1000;
    } else preguntas = nucleo.sesion(datos, ahora);
    simulacro = comoSimulacro;
    resultados = [];
    seguir();
    guardar();
  }

  // La ronda sigue en la primera pregunta sin resultado.
  function seguir() {
    i = resultados.length;
    nota = null;
    eleccion = null;
    error = "";
    if (i >= preguntas.length) return terminar();
    ronda = { estado: "Pregunta", contestada: false };
    if (simulacro) {
      // El intervalo va antes, porque contar() con el límite pasado llama a terminar(), que lo para.
      reloj = setInterval(contar, 1000);
      contar();
    }
  }

  function contar() {
    restante = nucleo.restante(limite, Date.now());
    if (!restante) terminar();
  }

  // Al pasar el límite, `nucleo` cuenta en blanco las preguntas sin contestar.
  function terminar() {
    // El reloj y la última pregunta llegan aquí a la vez si el límite pasa en la última.
    if (ronda.estado === "Final") return;
    clearInterval(reloj);
    ronda = { estado: "Final" };
    guardar();
    nota = nucleo.nota({ preguntas: preguntas.length, resultados }, datos!.examen.penalizacion);
  }

  async function contestar(opcion: number | null, seguro: boolean) {
    const respuesta = nucleo.corregir(q, opcion, seguro, Date.now());
    eleccion = { opcion, respuesta };
    ronda = { estado: "Pregunta", contestada: true };
    resultados.push(respuesta.resultado);
    guardar();
    localStorage.setItem(clavePendientes, JSON.stringify([...pendientes(), respuesta]));
    datos = { ...datos!, historial: nucleo.unir(datos!.historial, [respuesta]) };
    error = await enviarPendientes();
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
    guardar();
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
