import {
  cajas, dominio, imagenDePagina, nota, opciones, pdfEnPagina, porcentajeDudo, proximaFecha, respuesta, resultado, sesion, simulacro, unir, clave,
} from "./logica.js";

const PENDIENTES = "ln-pendientes";
const app = document.getElementById("app");
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
  pintar(
    el("div", { className: "meta", textContent: [ronda.simulacro ? "Simulacro" : null, `${ronda.i + 1} / ${ronda.preguntas.length}`, q.tema, ronda.simulacro ? null : `caja ${caja}`].filter(Boolean).join(" · ") }),
    el("div", { className: "q", textContent: q.pregunta }),
    ...filas, blanco, ayuda,
  );

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
    ayuda.remove();
    document.body.classList.add("con-hoja");
    const siguiente = () => (++ronda.i < ronda.preguntas.length ? pregunta(ronda) : fin(ronda));
    app.append(hoja(q, op, r, siguiente));
  }
}

function hoja(q, op, r, siguiente) {
  const aviso = op !== null && !r.acierto && r.seguro
    ? el("div", { className: "alarma", textContent: "Fallo con «seguro»: una idea falsa. Repásala." })
    : r.acierto && !r.seguro ? el("div", { className: "suerte", textContent: "Acierto con «dudo»: cuenta como un fallo." }) : null;
  const { pdf, pagina } = q.fuente;
  return el("div", { className: "hoja" }, el("div", {},
    aviso,
    el("p", { textContent: q.explicacion }),
    el("p", { className: "fuente", textContent: `${pdf}, página ${pagina} · ${q.concepto} (dominio: caja ${dominio(banco, historial).get(q.concepto)})` }),
    el("details", { className: "fuente" }, el("summary", { textContent: "Ver la página aquí" }),
      el("img", { src: imagenDePagina(q.fuente), alt: `La página ${pagina} de ${pdf}`, loading: "lazy" })),
    el("p", { className: "fuente" }, el("a", { href: pdfEnPagina(q.fuente), target: "_blank", textContent: `Abrir el PDF en la página ${pagina}` })),
    el("button", { className: "grande", textContent: "Siguiente", onclick: siguiente }),
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

addEventListener("online", enviar);
setInterval(enviar, 30000);
cargar();
