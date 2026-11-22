<script lang="ts">
  import * as nucleo from "./nucleo/nucleo.js";
  import type { Columnas, Datos, Guardado, Nota, NotaDelSimulacro, PreguntaDeSesion, Respuesta, Resultado, Ronda } from "./nucleo/nucleo.js";
  import Final from "./Final.svelte";
  import Hoja from "./Hoja.svelte";
  import Inicio from "./Inicio.svelte";
  import PaginaCompleta from "./PaginaCompleta.svelte";
  import Pregunta, { type Eleccion } from "./Pregunta.svelte";
  import Tiradores from "./Tiradores.svelte";

  // Todos los exámenes comparten un origen, así que cada clave lleva la carpeta del examen.
  const carpeta = location.pathname.split("/")[1];
  const claveObjetivo = `${carpeta}-objetivo`;
  const claveRonda = `${carpeta}-ronda`;
  const clavePendientes = `${carpeta}-pendientes`;
  const claveColumnas = `${carpeta}-columnas`;
  const claveNotasVisibles = `${carpeta}-notas-visibles`;
  let objetivo = $state(Number(localStorage.getItem(claveObjetivo) ?? 5));
  let datos = $state.raw<Datos | null>(null);
  let error = $state("");
  const inicio = $derived(datos && nucleo.inicio(datos, objetivo));

  // PROTOTIPO de navegar-preguntas y editar-pregunta: nada sale al servidor, y un fetch que no es GET no se envía.
  const fetch = (url: string, opciones?: RequestInit) =>
    opciones?.method && opciones.method !== "GET" ? Promise.resolve(new Response("{}")) : window.fetch(url, opciones);

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
    const sinRonda: Guardado = { guardada: new Date(0).toISOString(), ronda: undefined };
    const servidor: Guardado = await fetch("api/ronda")
      .then((r) => (r.ok ? r.json() : sinRonda))
      .catch(() => sinRonda);
    // Un clic en "Empezar" antes de la respuesta ya empezó una ronda nueva, que gana.
    if (ronda.estado !== "Inicio") return;
    const local = JSON.parse(localStorage.getItem(claveRonda) ?? "null") ?? sinRonda;
    // Un guardado de este dispositivo con la hora en milisegundos pasa a ISO, y no pierde la ronda.
    if (typeof local.guardada === "number") local.guardada = new Date(local.guardada).toISOString();
    const g = nucleo.gana(servidor, local);
    enviar(g);
    const r = nucleo.restaurar(datos, g, Date.now());
    if (!r) return;
    ({ preguntas, resultados, simulacro } = r);
    limite = r.limite ?? 0;
    seguir();
  }
  // PROTOTIPO: la ronda a medias del servidor no se restaura.
  void arrancar;
  cargarInicio();

  // La nota de cada pregunta, por su id. Sin la clave, la nota se ve en el PC y no en el móvil.
  let notas = $state<Record<string, string>>({});
  fetch("api/notas")
    .then((r) => r.json())
    .then((n) => (notas = n))
    .catch(() => {});
  let notasVisibles = $state(
    (localStorage.getItem(claveNotasVisibles) ?? (matchMedia("(min-width: 900px)").matches ? "si" : "no")) === "si",
  );
  function alternarNotas() {
    notasVisibles = !notasVisibles;
    localStorage.setItem(claveNotasVisibles, notasVisibles ? "si" : "no");
  }

  // La nota sale al servidor medio segundo después de la última tecla en su pregunta.
  const temporizadores: Record<string, ReturnType<typeof setTimeout>> = {};
  function anotar(texto: string) {
    const id = q.id;
    notas[id] = texto;
    clearTimeout(temporizadores[id]);
    temporizadores[id] = setTimeout(async () => {
      const r = await fetch("api/notas", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id, texto } satisfies Nota),
      }).catch(() => null);
      // Un envío correcto no borra el aviso de otra cosa, como el de una respuesta sin red.
      if (!r?.ok) error = "La nota no llegó al servidor, y sigue solo en esta pantalla.";
    }, 500);
  }

  function enviar(g: Guardado) {
    localStorage.setItem(claveRonda, JSON.stringify(g));
    fetch("api/ronda", { method: "PUT", headers: { "Content-Type": "application/json" }, body: JSON.stringify(g) }).catch(() => {});
  }

  // La ronda a medias, o ninguna en el inicio y el final.
  function guardar() {
    const aMedias = ronda.estado === "Pregunta";
    const ids = preguntas.map((q) => q.id);
    const hechos = resultados.filter((r) => r !== null);
    enviar({ guardada: new Date().toISOString(), ronda: aMedias ? { ids, resultados: hechos, simulacro, limite: simulacro ? limite : undefined } : undefined });
  }

  let ronda = $state<Ronda>({ estado: "Inicio" });
  let preguntas = $state.raw<PreguntaDeSesion[]>([]);
  let i = $state(0);
  // PROTOTIPO: la elección y el resultado van por posición, porque la ronda salta entre preguntas.
  let elecciones = $state<(Eleccion | null)[]>([]);
  const eleccion = $derived(elecciones[i] ?? null);
  let simulacro = $state(false);
  let resultados = $state<(Resultado | null)[]>([]);
  // La revisión de una pregunta desde el final.
  let revisando = $state(false);
  const final = $derived(ronda.estado === "Final");
  const marcas = $derived(
    preguntas.map((_, k) => {
      const r = resultados[k];
      if (r == null) return final ? "blanco" : "falta";
      if (simulacro && !final) return "contestada";
      return r === "Acierto" ? "acierto" : r === "Fallo" ? "fallo" : "blanco";
    }),
  );
  let nota = $state<NotaDelSimulacro | null>(null);
  let limite = 0;
  let reloj: ReturnType<typeof setInterval> | undefined;
  let restante = $state(0);

  let columnas = $state<Columnas>(JSON.parse(localStorage.getItem(claveColumnas) ?? "null") ?? nucleo.columnas());
  // Las otras opciones de una pregunta contestada, plegadas en el móvil.
  let abiertas = $state(false);
  // La página de la fuente a pantalla completa es una entrada del historial: la ✕ y el botón de atrás la cierran igual.
  let pagina = $state(false);
  function verPagina() {
    history.pushState({ pagina: true }, "");
    pagina = true;
  }
  addEventListener("popstate", () => (pagina = false));

  const q = $derived(preguntas[i]);

  function empezar(comoSimulacro: boolean) {
    if (!datos) return;
    const ahora = Date.now();
    if (comoSimulacro) {
      const s = nucleo.simulacro(datos, ahora);
      preguntas = s.preguntas;
      limite = ahora + s.segundos * 1000;
    } else preguntas = nucleo.sesion(datos, ahora);
    simulacro = comoSimulacro;
    resultados = preguntas.map(() => null);
    elecciones = preguntas.map(() => null);
    revisando = false;
    seguir();
    guardar();
  }

  // La ronda sigue en la primera pregunta sin resultado.
  function seguir() {
    i = resultados.findIndex((r) => r === null);
    if (i < 0) i = preguntas.length;
    nota = null;
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
    nota = nucleo.nota({ preguntas: preguntas.length, resultados: resultados.filter((r) => r !== null) }, datos!.examen.penalizacion);
  }

  async function contestar(opcion: number | null, seguro: boolean) {
    const correccion = nucleo.corregir(q, opcion, seguro, Date.now());
    const respuesta = correccion.respuesta;
    elecciones[i] = { opcion, correccion };
    ronda = { estado: "Pregunta", contestada: true };
    resultados[i] = respuesta.resultado;
    guardar();
    localStorage.setItem(clavePendientes, JSON.stringify([...pendientes(), respuesta]));
    datos = { ...datos!, historial: nucleo.unir(datos!.historial, [respuesta]) };
    error = await enviarPendientes();
  }

  function ir(k: number) {
    i = k;
    abiertas = false;
    if (!revisando) ronda = { estado: "Pregunta", contestada: elecciones[k] !== null };
  }

  function siguiente() {
    if (i + 1 < preguntas.length) ir(i + 1);
    else if (revisando) revisando = false;
    else intentarTerminar();
  }

  // La ronda no termina con preguntas sin contestar: lleva a la primera que falta.
  function intentarTerminar() {
    const falta = resultados.findIndex((r) => r === null);
    if (falta < 0) return terminar();
    const n = resultados.filter((r) => r === null).length;
    error = `Te ${n === 1 ? "falta 1 pregunta" : `faltan ${n} preguntas`}. Contéstala${n === 1 ? "" : "s"} para terminar.`;
    ir(falta);
  }

  function editar(cambios: { pregunta: string; opciones: string[]; explicacion: string }) {
    preguntas = preguntas.map((p, k) => (k === i ? { ...p, ...cambios } : p));
  }

  function volver() {
    clearInterval(reloj);
    cargarInicio();
    ronda = { estado: "Inicio" };
    guardar();
  }

  function abandonar() {
    if (revisando) return void (revisando = false);
    const que = simulacro ? "el simulacro" : "la sesión";
    if (confirm(`¿Abandonar ${que}? Las respuestas quedan en el historial.`)) volver();
  }
</script>

{#if ronda.estado === "Inicio" || (ronda.estado === "Final" && !revisando)}
  <!-- PROTOTIPO: el cambio de variante recarga la página, y la ronda se pierde. -->
  <nav class="fixed right-2 bottom-2 z-30 flex gap-1 rounded-full bg-tarjeta p-1 text-[12px] shadow-xl">
    <span class="px-2 py-1 text-tenue">PROTOTIPO · la navegación:</span>
    {#each [["a", "arriba"], ["b", "en el pie"]] as [v, nombre] (v)}
      <a
        class={["rounded-full px-2.5 py-1", (new URLSearchParams(location.search).get("v") ?? "a") === v ? "bg-acento text-sobre-acento" : "text-texto"]}
        href="?v={v}">{nombre}</a
      >
    {/each}
  </nav>
{/if}
{#if ronda.estado === "Inicio"}
  <Inicio
    {inicio}
    {objetivo}
    {error}
    onobjetivo={(o) => localStorage.setItem(claveObjetivo, String((objetivo = o)))}
    onempezar={empezar}
  />
{:else if ronda.estado === "Pregunta" || revisando}
  <!-- El móvil: la línea de arriba, la barra, el cuerpo con su scroll y el pie fijo.
       El PC: 2 columnas. --l, --s y --r son el borde izquierdo, el separador y el borde derecho. -->
  <div
    class="flex h-dvh flex-col text-[14px]/[1.45] [-webkit-tap-highlight-color:transparent] pc:ml-(--l) pc:text-[15px]/[1.5] pc:box-border pc:grid pc:h-auto pc:w-[calc(var(--r)-var(--l))] pc:grid-cols-[calc(var(--s)-var(--l)-48px)_1fr] pc:grid-rows-[auto_auto_auto_1fr] pc:content-start pc:items-start pc:gap-x-12 pc:px-6 pc:py-4 pc:[grid-template-areas:'meta_hoja'_'barra_hoja'_'izq_hoja'_'pie_hoja']"
    style:--l="{columnas.l * 100}vw"
    style:--s="{columnas.s * 100}vw"
    style:--r="{columnas.r * 100}vw"
  >
    <!-- PROTOTIPO: la edición a medias se cierra al cambiar de pregunta. -->
    {#key i}
    <Pregunta
      {q}
      {i}
      total={preguntas.length}
      {simulacro}
      {restante}
      {eleccion}
      {abiertas}
      nota={notas[q.id] ?? ""}
      {notasVisibles}
      {error}
      onabandonar={abandonar}
      oncontestar={contestar}
      onabiertas={() => (abiertas = !abiertas)}
      onnota={anotar}
      onnotasvisibles={alternarNotas}
      onverpagina={verPagina}
      onsiguiente={siguiente}
      corregida={!simulacro || final}
      revision={revisando}
      {marcas}
      oneditar={editar}
      onanterior={() => ir(i - 1)}
      onir={ir}
      onterminar={intentarTerminar}
    >
      <Hoja {q} paso={!simulacro || final ? (eleccion?.correccion.paso ?? (revisando ? q.paso : undefined)) : undefined} onsiguiente={siguiente} />
    </Pregunta>
    {/key}
    <Tiradores
      onmover={(k, x) => (columnas = nucleo.mover(columnas, k, x))}
      onsoltar={() => localStorage.setItem(claveColumnas, JSON.stringify(columnas))}
    />
  </div>
  {#if pagina}<PaginaCompleta {q} oncerrar={() => history.back()} />{/if}
{:else}
  <Final
    {simulacro}
    {nota}
    preguntas={preguntas.length}
    {error}
    onvolver={volver}
    {marcas}
    onrevisar={(k) => {
      revisando = true;
      i = k;
    }}
  />
{/if}
