<script lang="ts">
  import * as nucleo from "./nucleo/nucleo.js";
  import type { Columnas, Datos, Edicion, Guardado, LineaDeDesarrollo, Nota, NotaDeDesarrollo, NotaDelSimulacro, Pagina, Par, PreguntaDeDesarrolloDeSesion, PreguntaDeSesion, Respuesta, Resultado, Ronda } from "./nucleo/nucleo.js";
  import { aviso, type Marca } from "./Cuadricula.svelte";
  import Desarrollo from "./Desarrollo.svelte";
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
  const clavePendientesDeDesarrollo = `${carpeta}-pendientes-desarrollo`;
  const claveColumnas = `${carpeta}-columnas`;
  const claveNotasVisibles = `${carpeta}-notas-visibles`;
  let objetivo = $state(Number(localStorage.getItem(claveObjetivo) ?? 5));
  let datos = $state.raw<Datos | null>(null);
  let error = $state("");
  // Los pares marcados del inicio. Sin selección, todo está marcado.
  let seleccion = $state.raw<Par[] | undefined>(undefined);
  const inicio = $derived(datos && nucleo.inicio(datos, seleccion, objetivo, Date.now()));
  const cifraDeDesarrollo = $derived(datos ? nucleo.cifra_de_desarrollo(datos, seleccion, Date.now()) : undefined);

  // Una respuesta sin red espera en `localStorage` hasta que llega al servidor. El test y el desarrollo tienen cada uno su lista.
  const pendientes = <T,>(clave: string): T[] => JSON.parse(localStorage.getItem(clave) ?? "[]");

  // Los envíos van en cola, y una respuesta nueva solo entra al final de las pendientes.
  // Así las enviadas son las primeras. El error es "" cuando todas llegaron.
  let cola = Promise.resolve("");
  function enviarPendientes() {
    return (cola = cola.then(
      async () => (await enviarLista(clavePendientes, "api/respuestas")) || (await enviarLista(clavePendientesDeDesarrollo, "api/desarrollo")),
    ));
  }
  async function enviarLista(clave: string, ruta: string) {
    const enviando = pendientes(clave);
    if (!enviando.length) return "";
    const r = await fetch(ruta, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(enviando),
    }).catch(() => null);
    if (!r) return "Sin red: la respuesta se envía al volver la red.";
    if (!r.ok) return `El servidor responde ${r.status}, y la respuesta sigue en este dispositivo.`;
    localStorage.setItem(clave, JSON.stringify(pendientes(clave).slice(enviando.length)));
    return "";
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
      datos = {
        ...d,
        historial: nucleo.unir(d.historial, pendientes(clavePendientes)),
        historial_de_desarrollo: nucleo.unir_desarrollo(d.historial_de_desarrollo, pendientes(clavePendientesDeDesarrollo)),
      };
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
    if (ronda.estado !== "Inicio" || desarrollo) return;
    const local = JSON.parse(localStorage.getItem(claveRonda) ?? "null") ?? sinRonda;
    // Un guardado de este dispositivo con la hora en milisegundos pasa a ISO, y no pierde la ronda.
    if (typeof local.guardada === "number") local.guardada = new Date(local.guardada).toISOString();
    // Una ronda de este dispositivo sin las respuestas del simulacro las tiene ya en el historial.
    if (local.ronda && !local.ronda.respuestas) local.ronda.respuestas = [];
    const g = nucleo.gana(servidor, local);
    enviar(g);
    seleccion = g.seleccion;
    const d = nucleo.restaurar_desarrollo(datos, g, Date.now());
    if (d) {
      textos = d.textos;
      notasDeDesarrollo = d.notas;
      desarrollo = d.preguntas;
      return;
    }
    const r = nucleo.restaurar(datos, g, Date.now());
    if (!r) return;
    ({ preguntas, resultados, simulacro } = r);
    elecciones = preguntas.map(() => null);
    respuestas = preguntas.map((q) => r.respuestas.find((x) => x.id === q.id) ?? null);
    limite = r.limite ?? 0;
    seguir();
  }
  arrancar();

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
  function anotar(id: string, texto: string) {
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

  function enviar(g: Guardado, alServidor = true) {
    localStorage.setItem(claveRonda, JSON.stringify(g));
    if (alServidor) fetch("api/ronda", { method: "PUT", headers: { "Content-Type": "application/json" }, body: JSON.stringify(g) }).catch(() => {});
  }

  // La ronda a medias, o ninguna en el inicio y el final.
  function guardar(alServidor = true) {
    if (alServidor) clearTimeout(guardado);
    const guardada = new Date().toISOString();
    if (desarrollo) {
      const ids = desarrollo.map((q) => q.id);
      return enviar({ guardada, ronda: { ids, resultados: [], respuestas: [], simulacro: false, limite: undefined, desarrollo: { textos, notas: notasDeDesarrollo } }, seleccion }, alServidor);
    }
    const aMedias = ronda.estado === "Pregunta";
    const ids = preguntas.map((q) => q.id);
    enviar({ guardada, ronda: aMedias ? { ids, resultados, respuestas: respuestas.filter((r) => r !== null), simulacro, limite: simulacro ? limite : undefined } : undefined, seleccion });
  }

  let ronda = $state<Ronda>({ estado: "Inicio" });
  let preguntas = $state.raw<PreguntaDeSesion[]>([]);
  let i = $state(0);
  // La elección y el resultado de cada pregunta van en su posición, porque la ronda salta entre preguntas.
  let elecciones = $state<(Eleccion | null)[]>([]);
  // La última respuesta de cada pregunta del simulacro, que entra en el historial al terminar.
  let respuestas: (Respuesta | null)[] = [];
  const eleccion = $derived(elecciones[i] ?? null);
  let simulacro = $state(false);
  let resultados = $state<(Resultado | undefined)[]>([]);
  // La revisión de una pregunta desde el final.
  let revisando = $state(false);
  const final = $derived(ronda.estado === "Final");
  const contestada = $derived(revisando || (ronda.estado === "Pregunta" && ronda.contestada));
  const marcas: Marca[] = $derived(
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
      const s = nucleo.simulacro(datos, seleccion, ahora);
      preguntas = s.preguntas;
      limite = ahora + s.segundos * 1000;
    } else preguntas = nucleo.sesion(datos, seleccion, ahora);
    simulacro = comoSimulacro;
    resultados = preguntas.map(() => undefined);
    elecciones = preguntas.map(() => null);
    respuestas = preguntas.map(() => null);
    revisando = false;
    seguir();
    guardar();
  }

  // La ronda sigue en la primera pregunta sin resultado.
  function seguir() {
    i = resultados.findIndex((r) => r === undefined);
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
    entregar();
    guardar();
    nota = nucleo.nota({ preguntas: preguntas.length, resultados: resultados.filter((r) => r !== undefined) }, datos!.examen.penalizacion);
  }

  async function contestar(opcion: number | null, seguro: boolean) {
    const correccion = nucleo.corregir(q, opcion, seguro, Date.now());
    const respuesta = correccion.respuesta;
    elecciones[i] = { opcion, correccion };
    ronda = { estado: "Pregunta", contestada: true };
    resultados[i] = respuesta.resultado;
    if (simulacro) respuestas[i] = respuesta;
    guardar();
    if (!simulacro) await alHistorial([respuesta]);
  }

  async function alHistorial(nuevas: Respuesta[]) {
    if (!nuevas.length) return;
    localStorage.setItem(clavePendientes, JSON.stringify([...pendientes<Respuesta>(clavePendientes), ...nuevas]));
    datos = { ...datos!, historial: nucleo.unir(datos!.historial, nuevas) };
    error = await enviarPendientes();
  }

  // Las respuestas del simulacro entran en el historial al terminar o al abandonar, cada una una vez.
  function entregar() {
    const nuevas = respuestas.filter((r) => r !== null);
    respuestas = respuestas.map(() => null);
    alHistorial(nuevas);
  }

  function ir(k: number) {
    i = k;
    avisar = false;
    abiertas = false;
    if (!revisando) ronda = { estado: "Pregunta", contestada: resultados[k] !== undefined };
  }

  // Con todas contestadas, el botón grande termina desde cualquier pregunta. En la revisión, vuelve al resultado.
  const todas = $derived(resultados.every((r) => r !== undefined));
  const etiqueta = $derived(revisando ? "Resultado" : todas || i === preguntas.length - 1 ? "Terminar" : "Siguiente");
  function siguiente() {
    if (revisando) revisando = false;
    else if (todas || i + 1 >= preguntas.length) intentarTerminar();
    else ir(i + 1);
  }

  // La ronda no termina con preguntas sin contestar: lleva a la primera que falta, con el aviso hasta la pregunta siguiente.
  let avisar = $state(false);
  const faltan = $derived(resultados.filter((r) => r === undefined).length);
  function intentarTerminar() {
    const falta = resultados.findIndex((r) => r === undefined);
    if (falta < 0) return terminar();
    ir(falta);
    avisar = true;
  }

  // La edición entra en `preguntas.json`, en el banco de la pantalla y en la pregunta de la ronda, con el mismo `id`.
  async function editar(edicion: Edicion): Promise<string> {
    const e = nucleo.editar(datos!, q, edicion);
    if (!e) return "La pregunta ya no está en el banco: recarga la página.";
    const r = await fetch("api/preguntas", {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(e.banco),
    }).catch(() => null);
    if (!r) return "Sin red: la edición no llegó al servidor.";
    if (!r.ok) return `El servidor responde ${r.status}, y la pregunta sigue como estaba.`;
    const k = i;
    datos = { ...datos!, banco: datos!.banco.map((b) => (b.id === e.banco.id ? e.banco : b)) };
    preguntas = preguntas.map((p, j) => (j === k ? e.ronda : p));
    return "";
  }

  // La sesión de desarrollo. Sin ella, `desarrollo` es null.
  let desarrollo = $state.raw<PreguntaDeDesarrolloDeSesion[] | null>(null);
  let textos = $state<string[]>([]);
  let notasDeDesarrollo = $state<(NotaDeDesarrollo | undefined)[]>([]);
  // La ronda sigue en la primera pregunta sin nota.
  const j = $derived(notasDeDesarrollo.findIndex((n) => n === undefined));
  let solucion = $state(false);
  let paginaDeDesarrollo = $state.raw<Pagina | null>(null);

  function empezarDesarrollo() {
    if (!datos) return;
    const preguntas = nucleo.sesion_de_desarrollo(datos, seleccion, Date.now());
    if (!preguntas.length) return;
    textos = preguntas.map(() => "");
    notasDeDesarrollo = preguntas.map(() => undefined);
    solucion = false;
    error = "";
    desarrollo = preguntas;
    guardar();
  }

  // El texto entra en `localStorage` con cada tecla, y en el servidor medio segundo después de la última.
  let guardado: ReturnType<typeof setTimeout> | undefined;
  function escribir(texto: string) {
    textos[j] = texto;
    guardar(false);
    clearTimeout(guardado);
    guardado = setTimeout(() => guardar(), 500);
  }

  async function calificar(nota: NotaDeDesarrollo) {
    const lineas = nucleo.calificar(desarrollo![j].id, textos[j], nota, Date.now());
    notasDeDesarrollo[j] = nota;
    solucion = false;
    // Tras la última nota, la sesión vuelve al inicio.
    if (j < 0) desarrollo = null;
    guardar();
    localStorage.setItem(clavePendientesDeDesarrollo, JSON.stringify([...pendientes<LineaDeDesarrollo>(clavePendientesDeDesarrollo), ...lineas]));
    datos = { ...datos!, historial_de_desarrollo: nucleo.unir_desarrollo(datos!.historial_de_desarrollo, lineas) };
    error = await enviarPendientes();
  }

  function volver() {
    clearInterval(reloj);
    entregar();
    cargarInicio();
    ronda = { estado: "Inicio" };
    guardar();
  }

  function abandonar() {
    if (desarrollo) {
      if (!confirm("¿Abandonar la sesión de desarrollo? Las notas quedan en el historial.")) return;
      desarrollo = null;
      guardar();
      return;
    }
    if (revisando) return void (revisando = false);
    const que = simulacro ? "el simulacro" : "la sesión";
    if (confirm(`¿Abandonar ${que}? Las respuestas quedan en el historial.`)) volver();
  }
</script>

{#if desarrollo && j >= 0}
  {@const p = desarrollo[j]}
  {#key j}
    <Desarrollo
      q={p}
      i={j}
      total={desarrollo.length}
      texto={textos[j]}
      {solucion}
      nota={notas[p.id] ?? ""}
      {notasVisibles}
      {error}
      ontexto={escribir}
      onsolucion={() => (solucion = true)}
      oncalificar={calificar}
      onnota={(t) => anotar(p.id, t)}
      onnotasvisibles={alternarNotas}
      onverpagina={(g) => {
        paginaDeDesarrollo = g;
        verPagina();
      }}
      onabandonar={abandonar}
    />
  {/key}
  {#if pagina && paginaDeDesarrollo}<PaginaCompleta pagina={paginaDeDesarrollo} oncerrar={() => history.back()} />{/if}
{:else if ronda.estado === "Inicio"}
  <Inicio
    {inicio}
    desarrollo={cifraDeDesarrollo}
    ondesarrollo={empezarDesarrollo}
    {objetivo}
    {error}
    onobjetivo={(o) => localStorage.setItem(claveObjetivo, String((objetivo = o)))}
    onempezar={empezar}
    onseleccion={(s) => {
      seleccion = s;
      guardar();
    }}
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
    <!-- Una edición a medias se cierra al cambiar de pregunta. -->
    {#key i}
    <Pregunta
      {q}
      {i}
      total={preguntas.length}
      {simulacro}
      {restante}
      {eleccion}
      {contestada}
      {abiertas}
      nota={notas[q.id] ?? ""}
      {notasVisibles}
      error={error || (avisar && faltan ? aviso(faltan) : "")}
      onabandonar={abandonar}
      oncontestar={contestar}
      onabiertas={() => (abiertas = !abiertas)}
      onnota={(t) => anotar(q.id, t)}
      onnotasvisibles={alternarNotas}
      onverpagina={verPagina}
      onsiguiente={siguiente}
      corregida={!simulacro || final}
      revision={revisando}
      {marcas}
      oneditar={editar}
      onir={ir}
      onterminar={intentarTerminar}
      {etiqueta}
    >
      <Hoja {q} vuelve={contestada && (!simulacro || final) ? (eleccion?.correccion.vuelve ?? q.vuelve) : undefined} onsiguiente={siguiente} {etiqueta} />
    </Pregunta>
    {/key}
    <Tiradores
      onmover={(k, x) => (columnas = nucleo.mover(columnas, k, x))}
      onsoltar={() => localStorage.setItem(claveColumnas, JSON.stringify(columnas))}
    />
  </div>
  {#if pagina && q.pagina}<PaginaCompleta pagina={q.pagina} oncerrar={() => history.back()} />{/if}
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
