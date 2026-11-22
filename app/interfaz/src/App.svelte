<script lang="ts">
  import * as nucleo from "./nucleo/nucleo.js";
  import type { Columnas, Datos, Guardado, NotaDelSimulacro, PreguntaDeSesion, Respuesta, Resultado, Ronda } from "./nucleo/nucleo.js";
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
  let eleccion = $state<Eleccion | null>(null);
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

{#if ronda.estado === "Inicio"}
  <Inicio
    {inicio}
    {objetivo}
    {error}
    onobjetivo={(o) => localStorage.setItem(claveObjetivo, String((objetivo = o)))}
    onempezar={empezar}
  />
{:else if ronda.estado === "Pregunta"}
  <!-- El móvil: la línea de arriba, la barra, el cuerpo con su scroll y el pie fijo.
       El PC: 2 columnas. --l, --s y --r son el borde izquierdo, el separador y el borde derecho. -->
  <div
    class="flex h-dvh flex-col pc:ml-(--l) pc:box-border pc:grid pc:h-auto pc:w-[calc(var(--r)-var(--l))] pc:grid-cols-[calc(var(--s)-var(--l)-48px)_1fr] pc:grid-rows-[auto_auto_auto_1fr] pc:content-start pc:items-start pc:gap-x-12 pc:px-6 pc:py-4 pc:[grid-template-areas:'meta_hoja'_'barra_hoja'_'izq_hoja'_'pie_hoja']"
    style:--l="{columnas.l * 100}vw"
    style:--s="{columnas.s * 100}vw"
    style:--r="{columnas.r * 100}vw"
  >
    <Pregunta
      {q}
      {i}
      total={preguntas.length}
      {simulacro}
      {restante}
      {eleccion}
      {abiertas}
      {error}
      onabandonar={abandonar}
      oncontestar={contestar}
      onabiertas={() => (abiertas = !abiertas)}
      onverpagina={verPagina}
      onsiguiente={siguiente}
    >
      <Hoja {q} respuesta={eleccion?.respuesta} />
    </Pregunta>
    <Tiradores
      onmover={(k, x) => (columnas = nucleo.mover(columnas, k, x))}
      onsoltar={() => localStorage.setItem(claveColumnas, JSON.stringify(columnas))}
    />
  </div>
  {#if pagina}<PaginaCompleta {q} oncerrar={() => history.back()} />{/if}
{:else}
  <Final {simulacro} {nota} preguntas={preguntas.length} {error} onvolver={volver} />
{/if}
