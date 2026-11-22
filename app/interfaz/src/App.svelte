<script lang="ts">
  import * as nucleo from "./nucleo/nucleo.js";
  import type { Columnas, Datos, Guardado, NotaDelSimulacro, PreguntaDeSesion, Respuesta, Resultado, Ronda, Tirador } from "./nucleo/nucleo.js";

  // Todos los exámenes comparten un origen, así que cada clave lleva la carpeta del examen.
  const carpeta = location.pathname.split("/")[1];
  const claveObjetivo = `${carpeta}-objetivo`;
  const claveRonda = `${carpeta}-ronda`;
  const clavePendientes = `${carpeta}-pendientes`;
  const claveColumnas = `${carpeta}-columnas`;
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

  let columnas = $state<Columnas>(JSON.parse(localStorage.getItem(claveColumnas) ?? "null") ?? nucleo.columnas());
  // Las otras opciones de una pregunta contestada, plegadas en el móvil.
  let abiertas = $state(false);
  // La página de la fuente a pantalla completa es una entrada del historial: la ✕ y el botón de atrás la cierran igual.
  let pagina = $state(false);
  let zoom = $state(false);
  function verPagina() {
    history.pushState({ pagina: true }, "");
    pagina = true;
    zoom = false;
  }
  addEventListener("popstate", () => (pagina = false));

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
    abiertas = false;
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
    abiertas = false;
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

{#snippet aviso()}
  {#if error}<p class="error">{error}</p>{/if}
{/snippet}

{#if ronda.estado === "Inicio"}
  <main>
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
  {@render aviso()}
  </main>
{:else if ronda.estado === "Pregunta"}
  {@const otras = q.opciones.length - (eleccion?.opcion == null || eleccion.opcion === q.correcta ? 1 : 2)}
  <div class="ronda" style:--l="{columnas.l * 100}vw" style:--s="{columnas.s * 100}vw" style:--r="{columnas.r * 100}vw">
    <p class="meta">
      <span>
        {simulacro ? "Simulacro · " : ""}{i + 1} / {preguntas.length} · {q.tema} ·
        {simulacro ? `${Math.floor(restante / 60)}:${String(restante % 60).padStart(2, "0")}` : q.paso}
      </span>
      <button title="Abandonar" onclick={abandonar}>✕</button>
    </p>
    <div class="barra"><i style:width="{(100 * (i + 1)) / preguntas.length}%"></i></div>
    <div class="cuerpo">
      <div class="izq">
        <p class="enunciado">{q.pregunta}</p>
        {#each q.opciones as opcion, k (k)}
          {@const marca = !ronda.contestada ? "" : k === q.correcta ? "✓" : k === eleccion?.opcion ? "✗" : ""}
          <div class="opcion" class:resto={ronda.contestada && !marca} class:plegada={ronda.contestada && !marca && !abiertas}>
            <button disabled={ronda.contestada} onclick={() => contestar(k, true)}>{opcion}</button>
            {#if !ronda.contestada}
              <button onclick={() => contestar(k, false)}>dudo</button>
            {:else}
              {marca}
            {/if}
          </div>
        {/each}
        {#if ronda.contestada}
          <button class="chip" onclick={() => (abiertas = !abiertas)}>
            {abiertas ? "ocultar las otras" : `ver las otras ${otras} opciones`}
          </button>
          {#if eleccion?.respuesta.resultado === "Fallo" && eleccion.respuesta.seguro}
            <p>Fallo con «seguro»: una idea falsa. Repásala.</p>
          {:else if acierto && !eleccion?.respuesta.seguro}
            <p>Acierto con «dudo»: cuenta como un fallo.</p>
          {/if}
        {/if}
      </div>
      <div class="hoja">
        {#if !ronda.contestada}
          <p class="vacia">Elige una opción. La explicación sale aquí.</p>
        {:else}
          <p>{q.explicacion}</p>
          <p>{q.concepto} ({acierto && eleccion?.respuesta.seguro ? q.paso_tras_seguro : q.paso_tras_otra})</p>
          <p>{q.fuente} · <a href={q.pdf} target="_blank">Abrir el PDF en la página {q.pagina}</a></p>
          <img src={q.imagen} alt="La página {q.pagina} de la fuente" />
        {/if}
      </div>
    </div>
    <div class="pie">
      {#if !ronda.contestada}
        {#if simulacro}<button onclick={() => contestar(null, false)}>En blanco</button>{/if}
        <p>Toca el texto si estás seguro. Toca «dudo» si dudas.</p>
      {:else}
        <button class="ver-pagina" onclick={verPagina}>pág. {q.pagina}</button>
        <button onclick={siguiente}>Siguiente</button>
      {/if}
      {@render aviso()}
    </div>
    {#each ["l", "s", "r"] as const as k (k)}
      <div
        class="tirador"
        class:separador={k === "s"}
        role="separator"
        aria-orientation="vertical"
        style:left="var(--{k})"
        onpointerdown={(e) => {
          // Sin `preventDefault`, el navegador arrastra el texto seleccionado y cancela el puntero.
          e.preventDefault();
          e.currentTarget.setPointerCapture(e.pointerId);
        }}
        onpointermove={(e) => {
          if (e.currentTarget.hasPointerCapture(e.pointerId)) columnas = nucleo.mover(columnas, k.toUpperCase() as Tirador, e.clientX / innerWidth);
        }}
        onlostpointercapture={() => localStorage.setItem(claveColumnas, JSON.stringify(columnas))}
      ></div>
    {/each}
  </div>
  {#if pagina}
    <div class="pagina">
      <p class="meta">
        <span>{q.fuente}</span>
        <a href={q.pdf} target="_blank">PDF</a>
        <button title="Cerrar" onclick={() => history.back()}>✕</button>
      </p>
      <div class="cuerpo">
        <button class:zoom onclick={() => (zoom = !zoom)}><img src={q.imagen} alt="La página {q.pagina} de la fuente" /></button>
      </div>
      <p class="pie">Toca la página para ampliarla</p>
    </div>
  {/if}
{:else}
  <main>
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
  {@render aviso()}
  </main>
{/if}

<style>
  :global(body) {
    margin: 0;
    font: 15px/1.5 system-ui, sans-serif;
  }
  main {
    max-width: 560px;
    margin: 0 auto;
    padding: 16px;
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    padding: 8px 14px;
    font-size: 13px;
  }
  .meta span {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .barra {
    height: 3px;
    background: #ddd;
  }
  .barra i {
    display: block;
    height: 100%;
    background: #36c;
  }
  .opcion {
    display: flex;
    gap: 6px;
    align-items: center;
    margin-bottom: 8px;
  }
  .opcion button:first-child {
    flex: 1;
    text-align: left;
    padding: 10px;
  }
  .resto {
    opacity: 0.5;
  }
  .hoja img {
    display: block;
    width: 100%;
  }
  .error {
    color: #c33;
  }

  /* El móvil, por defecto: la línea de arriba, la barra, el cuerpo con su scroll y el pie fijo. */
  .ronda {
    display: flex;
    flex-direction: column;
    height: 100dvh;
  }
  .cuerpo {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 12px 14px;
  }
  .pie {
    padding: 10px 14px calc(10px + env(safe-area-inset-bottom));
    border-top: 2px solid #ddd;
  }
  .pie p {
    margin: 4px 0 0;
    font-size: 12px;
    text-align: center;
  }
  .plegada,
  .vacia,
  .hoja img,
  .tirador {
    display: none;
  }
  .chip {
    width: 100%;
    margin-bottom: 8px;
    border: 1px dashed #aaa;
    background: none;
  }
  .pagina {
    position: fixed;
    inset: 0;
    z-index: 10;
    display: flex;
    flex-direction: column;
    background: #fff;
  }
  .pagina .cuerpo {
    padding: 0;
  }
  .pagina .cuerpo button {
    display: block;
    width: 100%;
    padding: 0;
    border: 0;
    background: none;
  }
  .pagina img {
    display: block;
    width: 100%;
  }
  .pagina .zoom {
    width: 200%;
  }

  /* El PC: 2 columnas. --l, --s y --r son el borde izquierdo, el separador y el borde derecho. */
  @media (min-width: 900px) {
    .ronda {
      display: grid;
      grid-template-columns: calc(var(--s) - var(--l) - 48px) 1fr;
      grid-template-areas: "meta hoja" "barra hoja" "izq hoja" "pie hoja";
      grid-template-rows: auto auto auto 1fr;
      align-content: start;
      align-items: start;
      column-gap: 48px;
      box-sizing: border-box;
      height: auto;
      margin-left: var(--l);
      width: calc(var(--r) - var(--l));
      padding: 16px 24px;
    }
    .cuerpo {
      display: contents;
    }
    .meta {
      grid-area: meta;
      padding: 0 0 8px;
    }
    .barra {
      grid-area: barra;
    }
    .izq {
      grid-area: izq;
      padding-top: 12px;
    }
    .hoja {
      grid-area: hoja;
      position: sticky;
      top: 16px;
      max-height: calc(100vh - 32px);
      overflow: auto;
    }
    .pie {
      grid-area: pie;
      padding: 0;
      border: 0;
    }
    .plegada {
      display: flex;
    }
    .vacia {
      display: block;
      color: #888;
    }
    .hoja img {
      display: block;
    }
    .chip,
    .ver-pagina {
      display: none;
    }
    /* Cada tirador es una franja de 24 px. El separador muestra su línea, y el asa sale al pasar el ratón. */
    .tirador {
      display: block;
      position: fixed;
      top: 0;
      bottom: 0;
      width: 24px;
      margin-left: -12px;
      cursor: col-resize;
      touch-action: none;
      z-index: 9;
    }
    .tirador::after {
      content: "";
      position: absolute;
      top: 16px;
      bottom: 16px;
      left: 11px;
      width: 2px;
      background: #ddd;
    }
    .tirador:not(.separador)::after {
      background: none;
    }
    .tirador::before {
      content: "";
      display: none;
      position: absolute;
      z-index: 1;
      top: 50%;
      left: 5px;
      width: 14px;
      height: 40px;
      margin-top: -20px;
      border: 2px solid #36c;
      border-radius: 7px;
      background: radial-gradient(circle, #888 1.5px, transparent 2px) center / 6px 8px repeat-y, #fff;
      box-sizing: border-box;
    }
    .tirador:hover::before,
    .tirador:active::before {
      display: block;
    }
    .tirador:hover::after,
    .tirador:active::after {
      background: #36c;
    }
  }
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
