import { opciones, imagenDePagina, pdfEnPagina, textoParaCopiar } from "/logica.js";

const svg = (c, n = 14) => `<svg width="${n}" height="${n}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${c}</svg>`;
const circulo = (m) => svg(`<circle cx="12" cy="12" r="10"/><path d="${m}"/>`, 22);
const OK = circulo("M7.5 12.5l3 3 6-6.5");
const KO = circulo("M8.5 8.5l7 7M15.5 8.5l-7 7");
export const ICONO_COPIAR = svg('<rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15V5a2 2 0 0 1 2-2h10"/>');
export const ICONO_NOTAS = svg('<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.1 2.1 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>');
export const ICONO_DOC = svg('<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6M8 13h8M8 17h8"/>', 16);

export function el(etiqueta, props = {}, ...hijos) {
  const n = Object.assign(document.createElement(etiqueta), props);
  n.append(...hijos.filter(Boolean));
  return n;
}

// El banco real, con la pregunta de la captura la primera. ?q=<id> fuerza otra.
export async function cargar() {
  const banco = await (await fetch("/preguntas.json")).json();
  const forzada = new URLSearchParams(location.search).get("q");
  const k = banco.findIndex((q) => (forzada ? q.id === forzada : q.pregunta.includes("Declaración de Aplicabilidad")));
  const orden = k < 0 ? banco : [banco[k], ...banco.filter((_, i) => i !== k)];
  return orden.slice(0, 25).map((q) => ({ q, ops: opciones(q) }));
}

export function aviso(q, op, seguro) {
  const acierto = op === q.correcta;
  if (!acierto && seguro) return el("p", { className: "aviso alarma", textContent: "Fallo con «seguro»: una idea falsa. Repásala." });
  if (acierto && !seguro) return el("p", { className: "aviso suerte", textContent: "Acierto con «dudo»: cuenta como un fallo." });
  return null;
}

export function verFuente(q) {
  const { pdf, pagina } = q.fuente;
  const img = el("img", { src: "/" + imagenDePagina(q.fuente), onclick: () => img.classList.toggle("zoom") });
  const ov = el("div", { className: "overlay" },
    el("div", { className: "ov-cab" }, el("span", { className: "t", textContent: `${pdf} · pág. ${pagina}` }),
      el("a", { href: "/" + pdfEnPagina(q.fuente), target: "_blank", textContent: "PDF" }),
      el("button", { textContent: "✕", onclick: () => history.back() })),
    el("div", { className: "ov-cuerpo" }, img),
    el("div", { className: "ov-pie", textContent: "Toca la página para ampliarla" }));
  // El overlay es una entrada del historial: el botón de atrás y la ✕ lo cierran igual.
  history.pushState({ fuente: true }, "");
  const alAtras = () => { ov.remove(); removeEventListener("popstate", alAtras); };
  addEventListener("popstate", alAtras);
  document.body.append(ov);
}

export function marcar(filas, ops, q, op) {
  filas.forEach((f, k) => {
    const ok = ops[k] === q.correcta;
    f.classList.remove("sel");
    f.querySelector(".dudo")?.remove();
    f.querySelector(".texto").disabled = true;
    if (ok || ops[k] === op) {
      f.classList.add(ok ? "ok" : "ko");
      f.append(el("span", { className: "icono", innerHTML: ok ? OK : KO }));
    }
  });
}

// entrada(q, ops, contestar) -> { filas, antes?: nodos sobre las filas, pie?: nodo de abajo }
export function montar(banco, entrada) {
  const app = document.getElementById("app");
  let i = 0;
  const siguiente = () => { i = (i + 1) % banco.length; ver(); };

  function ver() {
    const { q, ops } = banco[i];
    const { filas, antes = [], pie } = entrada(q, ops, contestar);
    const notas = el("textarea", { className: "notas", rows: 3, placeholder: "Notas de esta pregunta", hidden: true });
    const bNotas = el("button", { className: "icono-b", innerHTML: ICONO_NOTAS, onclick: () => { notas.hidden = !notas.hidden; bNotas.classList.toggle("on", !notas.hidden); } });
    const bCopiar = el("button", { className: "icono-b", innerHTML: ICONO_COPIAR, onclick: async () => {
      await navigator.clipboard.writeText(textoParaCopiar(q, ops));
      bCopiar.classList.add("hecho");
      setTimeout(() => bCopiar.classList.remove("hecho"), 1000);
    } });
    const cuerpo = el("div", { className: "cuerpo" }, el("div", { className: "q", textContent: q.pregunta }), ...antes, ...filas, notas);
    const zonaPie = el("div", { className: "pie" }, pie);
    app.replaceChildren(el("div", { className: "pant" },
      el("div", { className: "meta" }, el("span", { className: "t", textContent: `${i + 1} / ${banco.length} · ${q.tema} · caja 1` }), bNotas, bCopiar),
      el("div", { className: "barra" }, el("i", { style: `width:${((i + 1) / banco.length) * 100}%` })),
      cuerpo, zonaPie));
    scrollTo(0, 0);

    function contestar(op, seguro) {
      marcar(filas, ops, q, op);
      antes.forEach((n) => n.remove());
      const otras = filas.filter((f) => !f.classList.contains("ok") && !f.classList.contains("ko"));
      otras.forEach((f) => f.classList.add("resto"));
      notas.before(...[aviso(q, op, seguro), el("p", { className: "expl", textContent: q.explicacion })].filter(Boolean));
      // ponytail: pliega las otras solo si el contenido no cabe en la pantalla.
      if (cuerpo.scrollHeight > cuerpo.clientHeight) {
        otras.forEach((f) => f.classList.add("plegada"));
        const mas = el("button", { className: "chip", textContent: `ver las otras ${otras.length} opciones`, onclick: () => {
          const abrir = otras[0].classList.contains("plegada");
          otras.forEach((f) => f.classList.toggle("plegada", !abrir));
          mas.textContent = abrir ? "ocultar las otras" : `ver las otras ${otras.length} opciones`;
        } });
        filas.at(-1).after(mas);
      }
      zonaPie.className = "pie acciones";
      zonaPie.replaceChildren(
        el("button", { className: "fuente-b", innerHTML: `${ICONO_DOC} pág. ${q.fuente.pagina}`, onclick: () => verFuente(q) }),
        el("button", { className: "grande", textContent: "Siguiente", onclick: siguiente }));
      cuerpo.scrollTo({ top: 0 });
    }
  }
  ver();
}
