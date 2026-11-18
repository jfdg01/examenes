import {
  cajas, dominio, textoParaCopiar, imagenDePagina, mover, COLUMNAS, MINUTOS_SIMULACRO, nota, opciones, pdfEnPagina, porcentajeDudo, respuesta, resultado, sesion, simulacro, unir, clave,
} from "./logica.js";

const PENDIENTES = "ln-pendientes";
const ICONO_COPIAR = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15V5a2 2 0 0 1 2-2h10"/></svg>';
const circulo = (marca) => `<svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="${marca}"/></svg>`;
const ICONO_NOTAS = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.1 2.1 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/></svg>';
const ICONO_DOC = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6M8 13h8M8 17h8"/></svg>';
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
      : el("p", { className: "tarjeta", textContent: banco.length
        ? "No hay preguntas pendientes. El repaso vuelve después de una pausa de 3 horas."
        : "El banco no tiene preguntas." }),
    el("button", { className: "grande otro", textContent: `Simulacro (${simulacro(banco).length} preguntas)`, onclick: () => pregunta({ preguntas: simulacro(banco), simulacro: true, i: 0, resultados: [], limite: Date.now() + MINUTOS_SIMULACRO * 60000 }) }),
  );
}

let reloj;
function correr(ronda, nodo) {
  clearInterval(reloj);
  const pinta = () => {
    const s = Math.max(0, Math.round((ronda.limite - Date.now()) / 1000));
    nodo.textContent = `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
    if (!s) fin(ronda);
  };
  pinta();
  reloj = setInterval(pinta, 1000);
}

function pregunta(ronda) {
  scrollTo(0, 0);
  const q = ronda.preguntas[ronda.i];
  const movil = !matchMedia("(min-width:900px)").matches;
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
  const campo = el("textarea", { className: "notas", rows: movil ? 3 : 4, placeholder: "Notas de esta pregunta", value: notas[q.id] ?? "", oninput: () => {
    notas[q.id] = campo.value;
    clearTimeout(espera);
    espera = setTimeout(() => guardarNota(q.id, campo.value), 500);
  } });
  campo.hidden = (localStorage.getItem(NOTAS_VISIBLES) ?? (movil ? "no" : "si")) === "no";
  const botonNotas = el("button", { className: "copiar", title: "Mostrar u ocultar las notas", innerHTML: ICONO_NOTAS, onclick: () => {
    campo.hidden = !campo.hidden;
    localStorage.setItem(NOTAS_VISIBLES, campo.hidden ? "no" : "si");
    botonNotas.classList.toggle("on", !campo.hidden);
  } });
  botonNotas.classList.toggle("on", !campo.hidden);
  const blanco = ronda.simulacro ? el("button", { className: "blanco", textContent: "En blanco", onclick: () => contestar(null, false) }) : null;
  const ayuda = el("p", { className: "dim ayuda", textContent: "Toca el texto si estás seguro. Toca «dudo» si dudas." });
  const derecha = el("div", { className: "hoja vacia" }, el("div", {}, el("p", { textContent: "Elige una opción. La explicación sale aquí." })));
  const titulo = [ronda.simulacro ? "Simulacro" : null, `${ronda.i + 1} / ${ronda.preguntas.length}`, q.tema, ronda.simulacro ? null : `caja ${caja}`].filter(Boolean).join(" · ");
  const enunciado = el("div", { className: "q", textContent: q.pregunta });
  const cuerpo = el("div", { className: "cuerpo" }, enunciado, ...filas, campo);
  const pie = el("div", { className: "pie" }, blanco, ayuda);
  const tiempo = ronda.simulacro ? el("span", { className: "reloj" }) : null;
  pintar(movil
    ? el("div", { className: "pant" },
      el("div", { className: "meta" }, el("span", { className: "t", textContent: titulo }), tiempo, botonNotas, copiar),
      el("div", { className: "barra" }, el("i", { style: `width:${((ronda.i + 1) / ronda.preguntas.length) * 100}%` })),
      cuerpo, pie)
    : el("div", { className: "dos" },
      el("div", { className: "izq" },
        el("div", { className: "meta", textContent: titulo }, el("span", { className: "acciones" }, tiempo, botonNotas, copiar, editar)),
        enunciado, ...filas, blanco, ayuda, campo),
      derecha));
  if (tiempo) correr(ronda, tiempo);

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
    const siguiente = () => (++ronda.i < ronda.preguntas.length ? pregunta(ronda) : fin(ronda));
    if (movil) return solucionMovil(aviso, siguiente);
    aviso ? ayuda.replaceWith(aviso) : ayuda.remove();
    derecha.replaceWith(hoja(q, siguiente));
  }

  // Las otras opciones se atenúan, y se pliegan solo si el cuerpo no cabe en la pantalla.
  function solucionMovil(aviso, siguiente) {
    const otras = filas.filter((f) => !f.classList.contains("ok") && !f.classList.contains("ko"));
    otras.forEach((f) => f.classList.add("resto"));
    campo.before(...[aviso, el("p", { className: "expl", textContent: q.explicacion })].filter(Boolean));
    if (cuerpo.scrollHeight > cuerpo.clientHeight) {
      otras.forEach((f) => f.classList.add("plegada"));
      const mas = el("button", { className: "chip", textContent: `ver las otras ${otras.length} opciones`, onclick: () => {
        const abrir = otras[0].classList.contains("plegada");
        otras.forEach((f) => f.classList.toggle("plegada", !abrir));
        mas.textContent = abrir ? "ocultar las otras" : `ver las otras ${otras.length} opciones`;
      } });
      filas.at(-1).after(mas);
    }
    pie.className = "pie acciones";
    pie.replaceChildren(
      el("button", { className: "fuente-b", innerHTML: `${ICONO_DOC} pág. ${q.fuente.pagina}`, onclick: () => verFuente(q) }),
      el("button", { className: "grande", textContent: "Siguiente", onclick: siguiente }));
    cuerpo.scrollTo({ top: 0 });
  }
}

// La fuente es una entrada del historial: la ✕ y el botón de atrás del móvil la cierran igual.
function verFuente(q) {
  const { pdf, pagina } = q.fuente;
  const img = el("img", { src: imagenDePagina(q.fuente), alt: `La página ${pagina} de ${pdf}`, onclick: () => img.classList.toggle("zoom") });
  const ov = el("div", { className: "overlay" },
    el("div", { className: "ov-cab" }, el("span", { className: "t", textContent: `${pdf} · pág. ${pagina}` }),
      el("a", { href: pdfEnPagina(q.fuente), target: "_blank", textContent: "PDF" }),
      el("button", { textContent: "✕", onclick: () => history.back() })),
    el("div", { className: "ov-cuerpo" }, img),
    el("div", { className: "ov-pie", textContent: "Toca la página para ampliarla" }));
  history.pushState({ fuente: true }, "");
  const alAtras = () => { ov.remove(); removeEventListener("popstate", alAtras); };
  addEventListener("popstate", alAtras);
  document.body.append(ov);
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
  clearInterval(reloj);
  while (ronda.resultados.length < ronda.preguntas.length) ronda.resultados.push("blanco");
  const cuenta = (tipo) => ronda.resultados.filter((x) => x === tipo).length;
  const resumen = ronda.simulacro
    ? [
      el("p", { className: "tarjeta", textContent: `Aciertos: ${cuenta("acierto")} · Fallos: ${cuenta("fallo")} · En blanco: ${cuenta("blanco")}` }),
      el("p", { className: "tarjeta", textContent: `Nota: ${numero(nota(ronda.resultados, 1 / 4))} sobre 10.` }),
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
  const ids = new Set(banco.map((q) => q.id));
  historial = historial.filter((r) => ids.has(r.id));
  try { notas = await (await fetch("api/notas")).json(); } catch {}
  inicio();
}

// Los tiradores son de la pantalla ancha: mueven los bordes y el separador de las 2 columnas.
function columnas() {
  const raiz = document.documentElement;
  let pos;
  const aplicar = () => {
    for (const k in pos) {
      const px = `${pos[k] * raiz.clientWidth}px`;
      raiz.style.setProperty(`--${k}`, px);
      document.getElementById(`tirador-${k}`).style.left = px;
    }
  };
  pos = { ...COLUMNAS, ...JSON.parse(localStorage.getItem("ln-columnas") ?? "{}") };
  aplicar();
  for (const k of ["L", "S", "R"]) {
    const t = document.getElementById(`tirador-${k}`);
    t.onpointerdown = (e) => { t.setPointerCapture(e.pointerId); t.classList.add("activo"); };
    t.onpointermove = (e) => {
      if (!t.hasPointerCapture(e.pointerId)) return;
      pos = mover(pos, k, e.clientX / raiz.clientWidth);
      aplicar();
    };
    t.onlostpointercapture = () => { t.classList.remove("activo"); localStorage.setItem("ln-columnas", JSON.stringify(pos)); };
  }
  editar.onclick = () => document.body.classList.toggle("editando");
  addEventListener("resize", aplicar);
}

columnas();
addEventListener("online", enviar);
setInterval(enviar, 30000);
cargar();
