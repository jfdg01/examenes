import {
  cajas, dominio, textoParaCopiar, imagenDePagina, mover, COLUMNAS, FILAS, nota, opciones, pdfEnPagina, porcentajeDudo, proximaFecha, respuesta, resultado, sesion, simulacro, unir, clave,
} from "./logica.js";

const PENDIENTES = "ln-pendientes";
const ICONO_COPIAR = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15V5a2 2 0 0 1 2-2h10"/></svg>';
const circulo = (marca) => `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="${marca}"/></svg>`;
const ICONO_OK = circulo("M7.5 12.5l3 3 6-6.5");
const ICONO_KO = circulo("M8.5 8.5l7 7M15.5 8.5l-7 7");
const app = document.getElementById("app");
const editar = document.getElementById("editar");
let banco = [];
let historial = [];
let alDia = true;
let notas = {};

const pendientes = () => JSON.parse(localStorage.getItem(PENDIENTES) ?? "[]");
const guardar = (lista) => localStorage.setItem(PENDIENTES, JSON.stringify(lista));
const numero = (n) => n.toLocaleString("es", { maximumFractionDigits: 2 });

const NOTAS_VISIBLES = "ln-notas-visibles";
const guardarNota = (id, texto) => fetch("api/notas", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ id, texto }) }).catch(() => {});

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
  scrollTo(0, 0);
  const q = ronda.preguntas[ronda.i];
  const ops = opciones(q);
  const caja = cajas(historial).get(q.id)?.caja ?? 1;
  const filas = ops.map((op) => el("div", { className: "fila" },
    el("button", { className: "texto", textContent: op, onclick: () => contestar(op, true) }),
    el("button", { className: "dudo", textContent: "dudo", onclick: () => contestar(op, false) })));
  const copiar = el("button", { className: "copiar", title: "Copiar la pregunta y las opciones", innerHTML: ICONO_COPIAR, onclick: async () => {
    await navigator.clipboard.writeText(textoParaCopiar(q, ops));
    copiar.classList.add("hecho");
    setTimeout(() => copiar.classList.remove("hecho"), 1000);
  } });
  let espera;
  const campo = el("textarea", { className: "notas", rows: 4, placeholder: "Notas de esta pregunta", value: notas[q.id] ?? "", oninput: () => {
    notas[q.id] = campo.value;
    clearTimeout(espera);
    espera = setTimeout(() => guardarNota(q.id, campo.value), 500);
  } });
  campo.hidden = localStorage.getItem(NOTAS_VISIBLES) === "no";
  const botonNotas = el("button", { className: "copiar", textContent: "Notas", onclick: () => {
    campo.hidden = !campo.hidden;
    localStorage.setItem(NOTAS_VISIBLES, campo.hidden ? "no" : "si");
    botonNotas.classList.toggle("on", !campo.hidden);
  } });
  botonNotas.classList.toggle("on", !campo.hidden);
  const blanco = ronda.simulacro ? el("button", { className: "blanco", textContent: "En blanco", onclick: () => contestar(null, false) }) : null;
  const ayuda = el("p", { className: "dim", textContent: "Toca el texto si estás seguro. Toca «dudo» si dudas." });
  const derecha = el("div", { className: "hoja vacia" }, el("div", {}, el("p", { textContent: "Elige una opción. La explicación sale aquí." })));
  pintar(el("div", { className: "dos" },
    el("div", { className: "izq" },
      el("div", { className: "meta", textContent: [ronda.simulacro ? "Simulacro" : null, `${ronda.i + 1} / ${ronda.preguntas.length}`, q.tema, ronda.simulacro ? null : `caja ${caja}`].filter(Boolean).join(" · ") }, el("span", { className: "acciones" }, botonNotas, copiar, editar)),
      el("div", { className: "q", textContent: q.pregunta }),
      ...filas, blanco, ayuda, campo),
    derecha));

  function contestar(op, seguro) {
    const r = respuesta(q, op, seguro, new Date());
    historial.push(r);
    guardar([...pendientes(), r]);
    enviar();
    ronda.resultados.push(resultado(q, op));
    filas.forEach((fila, k) => {
      const ok = ops[k] === q.correcta;
      if (ok || ops[k] === op) {
        fila.classList.add(ok ? "ok" : "ko");
        fila.append(el("span", { className: "icono", innerHTML: ok ? ICONO_OK : ICONO_KO }));
      }
      fila.querySelector(".dudo").remove();
      fila.querySelector(".texto").disabled = true;
    });
    blanco?.remove();
    const aviso = op !== null && !r.acierto && r.seguro
      ? el("p", { className: "aviso-respuesta alarma", textContent: "Fallo con «seguro»: una idea falsa. Repásala." })
      : r.acierto && !r.seguro ? el("p", { className: "aviso-respuesta suerte", textContent: "Acierto con «dudo»: cuenta como un fallo." }) : null;
    aviso ? ayuda.replaceWith(aviso) : ayuda.remove();
    const siguiente = () => (++ronda.i < ronda.preguntas.length ? pregunta(ronda) : fin(ronda));
    derecha.replaceWith(hoja(q, siguiente));
  }
}

function hoja(q, siguiente) {
  return el("div", { className: "hoja" }, el("div", {},
    el("p", { textContent: q.explicacion }),
    el("button", { className: "grande", textContent: "Siguiente", onclick: siguiente }),
    fuentes(q)));
}

function fuentes(q) {
  const { pdf, pagina } = q.fuente;
  const paneles = [
    ["Ver la página aquí", () => el("img", { src: imagenDePagina(q.fuente), alt: `La página ${pagina} de ${pdf}` })],
    ["La fuente", () => el("div", {},
      el("p", { textContent: `${pdf}, página ${pagina} · ${q.concepto} (dominio: caja ${dominio(banco, historial).get(q.concepto)})` }),
      el("p", {}, el("a", { href: pdfEnPagina(q.fuente), target: "_blank", textContent: `Abrir el PDF en la página ${pagina}` })))],
  ];
  const panel = el("div", { className: "panel" });
  const botones = paneles.map(([texto, crear]) => {
    const boton = el("button", { className: "fuente", textContent: texto, onclick: () => {
      const abrir = !boton.classList.contains("abierto");
      botones.forEach((b) => b.classList.remove("abierto"));
      panel.replaceChildren(...(abrir ? [crear()] : []));
      boton.classList.toggle("abierto", abrir);
    } });
    return boton;
  });
  return el("div", {}, el("div", { className: "fuentes" }, ...botones), panel);
}

function fin(ronda) {
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
  try { notas = await (await fetch("api/notas")).json(); } catch {}
  inicio();
}

function columnas() {
  const ancha = matchMedia("(min-width:900px)");
  const raiz = document.documentElement;
  const horizontal = (k) => ancha.matches || "IF".includes(k);
  const medida = (k) => horizontal(k) ? raiz.clientWidth : raiz.clientHeight;
  let pos;
  const aplicar = () => {
    for (const k in pos) {
      const px = `${pos[k] * medida(k)}px`;
      raiz.style.setProperty(`--${k}`, px);
      const t = document.getElementById(`tirador-${k}`);
      t.style.left = horizontal(k) ? px : "";
      t.style.top = horizontal(k) ? "" : px;
    }
  };
  const guardado = () => ancha.matches ? "ln-columnas" : "ln-filas";
  const leer = () => {
    pos = { ...(ancha.matches ? COLUMNAS : FILAS), ...JSON.parse(localStorage.getItem(guardado()) ?? "{}") };
    aplicar();
    rotular();
  };
  const rotular = () => {
    const editando = document.body.classList.contains("editando");
    editar.textContent = editando ? "Listo" : "Ajustar";
  };
  for (const k of ["L", "S", "R", "I", "F"]) {
    const t = document.getElementById(`tirador-${k}`);
    t.onpointerdown = (e) => { t.setPointerCapture(e.pointerId); t.classList.add("activo"); };
    t.onpointermove = (e) => {
      if (!t.hasPointerCapture(e.pointerId)) return;
      pos = mover(pos, k, (horizontal(k) ? e.clientX : e.clientY) / medida(k));
      aplicar();
    };
    t.onlostpointercapture = () => { t.classList.remove("activo"); localStorage.setItem(guardado(), JSON.stringify(pos)); };
  }
  editar.onclick = () => { document.body.classList.toggle("editando"); rotular(); };
  ancha.onchange = leer;
  addEventListener("resize", aplicar);
  leer();
}

columnas();
addEventListener("online", enviar);
setInterval(enviar, 30000);
cargar();
