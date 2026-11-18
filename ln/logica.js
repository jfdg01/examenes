export function cajas(historial) {
  const estado = new Map();
  for (const r of [...historial].sort((a, b) => a.ts.localeCompare(b.ts))) {
    const caja = estado.get(r.id)?.caja ?? 1;
    estado.set(r.id, { caja: r.acierto && r.seguro ? Math.min(caja + 1, 3) : 1, ts: r.ts });
  }
  return estado;
}

const TAMANO_SESION = 12;
const PAUSA = 3 * 3600 * 1000;
const PESO = { 2: 1, 3: 0.5 };

export function barajar(lista, azar = Math.random) {
  const copia = [...lista];
  for (let i = copia.length - 1; i > 0; i--) {
    const j = Math.floor(azar() * (i + 1));
    [copia[i], copia[j]] = [copia[j], copia[i]];
  }
  return copia;
}

function principioDelBloque(historial, ahora) {
  let principio = ahora.getTime();
  for (const t of historial.map((r) => Date.parse(r.ts)).sort((a, b) => b - a)) {
    if (t > principio) continue;
    if (principio - t >= PAUSA) break;
    principio = t;
  }
  return principio;
}

export function sesion(banco, historial, ahora, azar = Math.random) {
  const estado = cajas(historial);
  const bloque = principioDelBloque(historial, ahora);
  const vistos = new Set(banco.filter((q) => estado.has(q.id)).map((q) => q.concepto));
  const caja1 = [], sinVer = new Map(), sorteo = [];
  for (const q of barajar(banco, azar)) {
    const e = estado.get(q.id);
    if (e?.caja === 1) caja1.push(q);
    else if (!vistos.has(q.concepto)) { if (!sinVer.has(q.concepto)) sinVer.set(q.concepto, q); }
    else if (!e || Date.parse(e.ts) < bloque) sorteo.push({ q, clave: -Math.log(azar()) / (e ? PESO[e.caja] : 1) });
  }
  const elegidas = sorteo.sort((a, b) => a.clave - b.clave).map((x) => x.q);
  return barajar([...caja1, ...sinVer.values(), ...elegidas].slice(0, TAMANO_SESION), azar);
}

export function conceptos(banco, historial) {
  const concepto = new Map(banco.map((q) => [q.id, q.concepto]));
  const estado = new Map(banco.map((q) => [q.concepto, "sinVer"]));
  for (const r of [...historial].sort((a, b) => a.ts.localeCompare(b.ts)))
    if (concepto.has(r.id)) estado.set(concepto.get(r.id), r.acierto && r.seguro ? "sabes" : "noSabes");
  const cuenta = { sabes: 0, noSabes: 0, sinVer: 0 };
  for (const e of estado.values()) cuenta[e]++;
  return cuenta;
}

export const opciones = (pregunta, azar = Math.random) => barajar([pregunta.correcta, ...pregunta.distractores], azar);

export const textoParaCopiar = (pregunta, ops) => `${pregunta.pregunta}\n\n${ops.map((op, k) => `${String.fromCharCode(65 + k)}) ${op}`).join("\n")}`;

export function respuesta(pregunta, eleccion, seguro, ahora) {
  const ts = ahora.toISOString().slice(0, 19) + "Z";
  if (eleccion === null) return { id: pregunta.id, ts, acierto: false, seguro: false };
  return { id: pregunta.id, ts, acierto: eleccion === pregunta.correcta, seguro };
}

export function porcentajeDudo(historial) {
  const dudas = historial.filter((r) => !r.seguro);
  if (!dudas.length) return null;
  const porcentaje = (100 * dudas.filter((r) => r.acierto).length) / dudas.length;
  return { porcentaje, total: dudas.length, contestar: porcentaje > 20 };
}

export function dominio(banco, historial) {
  const estado = cajas(historial);
  const minimo = new Map();
  for (const q of banco) {
    const caja = estado.get(q.id)?.caja ?? 1;
    minimo.set(q.concepto, Math.min(caja, minimo.get(q.concepto) ?? 3));
  }
  return minimo;
}

export const clave = (r) => r.id + "|" + r.ts;

export function unir(servidor, locales) {
  const vistas = new Map();
  for (const r of [...servidor, ...locales]) if (!vistas.has(clave(r))) vistas.set(clave(r), r);
  return [...vistas.values()];
}

const TAMANO_SIMULACRO = 20;
export const MINUTOS_SIMULACRO = 25;

export const simulacro = (banco, azar = Math.random) => barajar(banco, azar).slice(0, TAMANO_SIMULACRO);

export const PENALIZACION = 1 / 4;

const binomial = (n, k, p) => {
  let c = 1;
  for (let i = 0; i < k; i++) c = (c * (n - i)) / (i + 1);
  return c * p ** k * (1 - p) ** (n - k);
};

// acierto es la probabilidad de acertar una pregunta de un concepto que no sabes; null la deja en blanco.
// objetivo es la nota sobre 10, y pide objetivo * n / 10 aciertos netos.
export function probabilidadDeLaNota(sabes, acierto, objetivo, n = TAMANO_SIMULACRO) {
  const netos = (objetivo * n) / 10;
  let total = 0;
  for (let k = 0; k <= n; k++) {
    const resto = n - k;
    if (acierto === null) { if (k >= netos) total += binomial(n, k, sabes); continue; }
    for (let a = 0; a <= resto; a++) if (k + a - PENALIZACION * (resto - a) >= netos) total += binomial(n, k, sabes) * binomial(resto, a, acierto);
  }
  return total;
}

export const resultado =(pregunta, eleccion) => (eleccion === null ? "blanco" : eleccion === pregunta.correcta ? "acierto" : "fallo");

export function nota(resultados, penalizacion) {
  const cuenta = (tipo) => resultados.filter((x) => x === tipo).length;
  return ((cuenta("acierto") - penalizacion * cuenta("fallo")) / resultados.length) * 10;
}

export const imagenDePagina = ({ pdf, pagina }) => encodeURI(`paginas/${pdf.replace(/\.pdf$/, "")}-${pagina}.png`);

export const pdfEnPagina = ({ pdf, pagina }) => `${encodeURI(pdf)}#page=${pagina}`;

// L es el borde izquierdo, S el separador y R el borde derecho de las 2 columnas: de izquierda a derecha.
export const COLUMNAS = { L: 0.08, S: 0.5, R: 0.92 };
const COLUMNA_MINIMA = 0.15;

export function mover(pos, tirador, x) {
  const [min, max] = { L: [0, pos.S - COLUMNA_MINIMA], S: [pos.L + COLUMNA_MINIMA, pos.R - COLUMNA_MINIMA], R: [pos.S + COLUMNA_MINIMA, 1] }[tirador];
  return { ...pos, [tirador]: Math.min(max, Math.max(min, x)) };
}
