import {
  cajas, dominio, imagenDePagina, mover, COLUMNAS, nota, opciones, pdfEnPagina, porcentajeDudo, proximaFecha, respuesta, resultado, sesion, simulacro, unir, clave,
} from "./logica.js";

const PENDIENTES = "ln-pendientes";
const POSICIONES = "ln-columnas";
const app = document.getElementById("app");
const editar = document.getElementById("editar");
let banco = [];
let historial = [];
let alDia = true;

const pendientes = () => JSON.parse(localStorage.getItem(PENDIENTES) ?? "[]");
const guardar = (lista) => localStorage.setItem(PENDIENTES, JSON.stringify(lista));
const numero = (n) => n.toLocaleString("es", { maximumFractionDigits: 2 });

const pintar = (...nodos) => app.replaceChildren(...nodos.filter(Boolean));

function el(etiqueta, propiedades = {}, ...hijos) {
  const nodo = Object.assign(document.createElement(etiqueta), propiedades);
  nodo.append(...hijos.filter(Boolean));
  return nodo;
}

async function enviar() {
  const lote = pendientes();
  if (!lote.length) return;
  try {
    const r = await fetch("api/respuestas", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(lote) });
    if (!r.ok) return;
    const enviadas = new Set(lote.map(clave));
    guardar(pendientes().filter((x) => !enviadas.has(clave(x))));
  } catch {}
}

function inicio() {
  document.body.classList.remove("con-hoja");
  const lista = sesion(banco, historial, new Date());
  const dudo = porcentajeDudo(historial);
  const sinEnviar = pendientes().length;
  const proxima = proximaFecha(banco, historial);
  pintar(
    el("h1", { textContent: "LN: el test" }),
    el("p", { className: "dim", textContent: `${banco.length} preguntas en el banco · ${historial.length} respuestas en el historial` }),
    alDia ? null : el("p", { className: "aviso", textContent: "Sin conexión con el servidor: el historial no está al día." }),
    sinEnviar ? el("p", { className: "aviso", textContent: `${sinEnviar} respuestas sin enviar al servidor.` }) : null,
    dudo && el("div", { className: "tarjeta", textContent:
      `Aciertos con «dudo»: ${numero(dudo.porcentaje)} % de ${dudo.total}. ` +
      (dudo.contestar ? "Una pregunta dudosa se contesta." : "Una pregunta dudosa se deja en blanco.") }),
    lista.length
      ? el("button", { className: "grande", textContent: `Empezar la sesión (${lista.length} preguntas)`, onclick: () => pregunta({ preguntas: lista, simulacro: false, i: 0, resultados: [] }) })
      : el("p", { className: "tarjeta", textContent: proxima
        ? `No hay preguntas pendientes. La próxima vuelve el ${proxima.toLocaleDateString("es", { weekday: "long", day: "numeric", month: "long" })}.`
        : "El banco no tiene preguntas." }),
    el("button", { className: "grande otro", textContent: `Simulacro (${simulacro(banco).length} preguntas)`, onclick: () => pregunta({ preguntas: simulacro(banco), simulacro: true, i: 0, resultados: [] }) }),
  );
}

function pregunta(ronda) {
  document.body.classList.remove("con-hoja");
  scrollTo(0, 0);
  const q = ronda.preguntas[ronda.i];
  const ops = opciones(q);
  const caja = cajas(historial).get(q.id)?.caja ?? 1;
  const filas = ops.map((op) => el("div", { className: "fila" },
    el("button", { className: "texto", textContent: op, onclick: () => contestar(op, true) }),
    el("button", { className: "dudo", textContent: "dudo", onclick: () => contestar(op, false) })));
  const blanco = ronda.simulacro ? el("button", { className: "blanco", textContent: "En blanco", onclick: () => contestar(null, false) }) : null;
  const ayuda = el("p", { className: "dim", textContent: "Toca el texto si estás seguro. Toca «dudo» si dudas." });
  const derecha = el("div", { className: "hoja vacia" }, el("div", {}, el("p", { textContent: "Elige una opción. La explicación sale aquí." })));
  pintar(el("div", { className: "dos" },
    el("div", { className: "izq" },
      el("div", { className: "meta", textContent: [ronda.simulacro ? "Simulacro" : null, `${ronda.i + 1} / ${ronda.preguntas.length}`, q.tema, ronda.simulacro ? null : `caja ${caja}`].filter(Boolean).join(" · ") }, editar),
      el("div", { className: "q", textContent: q.pregunta }),
      ...filas, blanco, ayuda),
    derecha));

  function contestar(op, seguro) {
    const r = respuesta(q, op, seguro, new Date());
    historial.push(r);
    guardar([...pendientes(), r]);
    enviar();
    ronda.resultados.push(resultado(q, op));
    filas.forEach((fila, k) => {
      if (ops[k] === q.correcta) fila.classList.add("ok");
      else if (ops[k] === op) fila.classList.add("ko");
      fila.querySelector(".dudo").remove();
      fila.querySelector(".texto").disabled = true;
    });
    blanco?.remove();
    const aviso = op !== null && !r.acierto && r.seguro
      ? el("p", { className: "aviso-respuesta alarma", textContent: "Fallo con «seguro»: una idea falsa. Repásala." })
      : r.acierto && !r.seguro ? el("p", { className: "aviso-respuesta suerte", textContent: "Acierto con «dudo»: cuenta como un fallo." }) : null;
    aviso ? ayuda.replaceWith(aviso) : ayuda.remove();
    document.body.classList.add("con-hoja");
    const siguiente = () => (++ronda.i < ronda.preguntas.length ? pregunta(ronda) : fin(ronda));
    derecha.replaceWith(hoja(q, siguiente));
  }
}

function hoja(q, siguiente) {
  const { pdf, pagina } = q.fuente;
  return el("div", { className: "hoja" }, el("div", {},
    el("p", { textContent: q.explicacion }),
    el("button", { className: "grande", textContent: "Siguiente", onclick: siguiente }),
    el("details", { className: "fuente" }, el("summary", { textContent: "Ver la página aquí" }),
      el("img", { src: imagenDePagina(q.fuente), alt: `La página ${pagina} de ${pdf}`, loading: "lazy" })),
    el("details", { className: "fuente" }, el("summary", { textContent: "La fuente" }),
      el("p", { textContent: `${pdf}, página ${pagina} · ${q.concepto} (dominio: caja ${dominio(banco, historial).get(q.concepto)})` }),
      el("p", {}, el("a", { href: pdfEnPagina(q.fuente), target: "_blank", textContent: `Abrir el PDF en la página ${pagina}` }))),
  ));
}

function fin(ronda) {
  document.body.classList.remove("con-hoja");
  const cuenta = (tipo) => ronda.resultados.filter((x) => x === tipo).length;
  const resumen = ronda.simulacro
    ? [
      el("p", { className: "tarjeta", textContent: `Aciertos: ${cuenta("acierto")} · Fallos: ${cuenta("fallo")} · En blanco: ${cuenta("blanco")}` }),
      el("p", { className: "tarjeta", textContent: `Nota con una penalización de 1/3: ${numero(nota(ronda.resultados, 1 / 3))} sobre 10.` }),
      el("p", { className: "tarjeta", textContent: `Nota con una penalización de 1/2: ${numero(nota(ronda.resultados, 1 / 2))} sobre 10.` }),
    ]
    : [el("p", { className: "tarjeta", textContent: `Aciertos: ${cuenta("acierto")} de ${ronda.resultados.length}` })];
  pintar(
    el("h1", { textContent: ronda.simulacro ? "El simulacro ha terminado" : "La sesión ha terminado" }),
    ...resumen,
    el("button", { className: "grande", textContent: "Volver al inicio", onclick: inicio }),
  );
}

async function cargar() {
  banco = await (await fetch("preguntas.json")).json();
  await enviar();
  try {
    const r = await fetch("api/respuestas");
    if (!r.ok) throw new Error(r.status);
    historial = unir(await r.json(), pendientes());
  } catch {
    historial = pendientes();
    alDia = false;
  }
  inicio();
}

function columnas() {
  let pos = { ...COLUMNAS, ...JSON.parse(localStorage.getItem(POSICIONES) ?? "{}") };
  const aplicar = () => {
    const ancho = document.documentElement.clientWidth;
    for (const k in pos) {
      document.documentElement.style.setProperty(`--${k}`, `${pos[k] * ancho}px`);
      document.getElementById(`tirador-${k}`).style.left = `${pos[k] * ancho}px`;
    }
  };
  for (const k in pos) {
    const t = document.getElementById(`tirador-${k}`);
    t.onpointerdown = (e) => { t.setPointerCapture(e.pointerId); t.classList.add("activo"); };
    t.onpointermove = (e) => {
      if (!t.hasPointerCapture(e.pointerId)) return;
      pos = mover(pos, k, e.clientX / document.documentElement.clientWidth);
      aplicar();
    };
    t.onlostpointercapture = () => { t.classList.remove("activo"); localStorage.setItem(POSICIONES, JSON.stringify(pos)); };
  }
  editar.onclick = () => {
    editar.textContent = document.body.classList.toggle("editando") ? "Listo" : "Ajustar las columnas";
  };
  addEventListener("resize", aplicar);
  aplicar();
}

columnas();
addEventListener("online", enviar);
setInterval(enviar, 30000);
cargar();
