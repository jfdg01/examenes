const TAMANO_SESION = 12;
const HUECOS_SIN_VER = 4;
const HUECO = 10 * 60 * 1000;

export function pasos(banco, historial) {
  const concepto = new Map(banco.map((q) => [q.id, q.concepto]));
  const estado = new Map();
  for (const r of [...historial].sort((a, b) => a.ts.localeCompare(b.ts))) {
    const c = concepto.get(r.id);
    if (c === undefined) continue;
    const paso = r.acierto && r.seguro ? (estado.get(c)?.paso ?? 0) + 1 : 0;
    estado.set(c, { paso, vence: Date.parse(r.ts) + (paso && HUECO * 3 ** (paso - 1)) });
  }
  return estado;
}

export function barajar(lista, azar = Math.random) {
  const copia = [...lista];
  for (let i = copia.length - 1; i > 0; i--) {
    const j = Math.floor(azar() * (i + 1));
    [copia[i], copia[j]] = [copia[j], copia[i]];
  }
  return copia;
}

export function sesion(banco, historial, ahora, azar = Math.random) {
  const estado = pasos(banco, historial);
  const ultima = new Map();
  for (const r of historial) if (!(r.ts <= ultima.get(r.id))) ultima.set(r.id, r.ts);
  const pregunta = new Map();
  for (const q of barajar(banco, azar)) {
    const otra = pregunta.get(q.concepto);
    if (!otra || (ultima.has(otra.id) && (!ultima.has(q.id) || ultima.get(q.id) < ultima.get(otra.id)))) pregunta.set(q.concepto, q);
  }
  const conceptos = [...pregunta.keys()];
  const sinVer = conceptos.filter((c) => !estado.has(c));
  const vistos = conceptos.filter((c) => estado.has(c)).sort((a, b) => estado.get(a).vence - estado.get(b).vence);
  const vencidos = vistos.filter((c) => estado.get(c).vence <= ahora.getTime()).slice(0, TAMANO_SESION - (sinVer.length && HUECOS_SIN_VER));
  const resto = vistos.slice(vencidos.length);
  return barajar([...vencidos, ...sinVer, ...resto].slice(0, TAMANO_SESION).map((c) => pregunta.get(c)), azar);
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
  const aciertos = dudas.filter((r) => r.acierto).length;
  return { porcentaje: (100 * aciertos) / dudas.length, total: dudas.length, aciertos };
}

export const clave = (r) => r.id + "|" + r.ts;

export function unir(servidor, locales) {
  const vistas = new Map();
  for (const r of [...servidor, ...locales]) if (!vistas.has(clave(r))) vistas.set(clave(r), r);
  return [...vistas.values()];
}

// Un examen es el examen.json de su carpeta: titulo, preguntas, opciones, minutos, penalizacion y temas.
// penalizacion es la parte de un acierto que resta un fallo.
export const simulacro = (banco, n, azar = Math.random) => barajar(banco, azar).slice(0, n);

const binomial = (n, k, p) => {
  let c = 1;
  for (let i = 0; i < k; i++) c = (c * (n - i)) / (i + 1);
  return c * p ** k * (1 - p) ** (n - k);
};

// acierto es la probabilidad de acertar una pregunta de un concepto que no sabes; null la deja en blanco.
// objetivo es la nota sobre 10, y pide objetivo * n / 10 aciertos netos.
export function probabilidadDeLaNota(sabes, acierto, objetivo, { preguntas: n, penalizacion }) {
  const netos = (objetivo * n) / 10;
  let total = 0;
  for (let k = 0; k <= n; k++) {
    const resto = n - k;
    if (acierto === null) { if (k >= netos) total += binomial(n, k, sabes); continue; }
    for (let a = 0; a <= resto; a++) if (k + a - penalizacion * (resto - a) >= netos) total += binomial(n, k, sabes) * binomial(resto, a, acierto);
  }
  return total;
}

const META = 0.95;
const VENTANA = 60;

// La tarjeta de la nota: la probabilidad de hoy y de la forma con más probabilidad, los conceptos y las sesiones que faltan para un 95 %.
export function tarjetaDeLaNota(banco, historial, objetivo, examen) {
  const aCiegas = 1 / examen.opciones;
  const c = conceptos(banco, historial);
  const n = c.sabes + c.noSabes + c.sinVer, total = n || 1;
  const dudo = porcentajeDudo(historial);
  const tasaDudo = dudo ? dudo.porcentaje / 100 : aCiegas;
  const tasa = (c.noSabes * tasaDudo + c.sinVer * aCiegas) / (c.noSabes + c.sinVer || 1);
  const probabilidad = (s, acierto) => probabilidadDeLaNota(s / total, acierto, objetivo, examen);
  const mejor = (s) => Math.max(probabilidad(s, null), probabilidad(s, tasa));
  let meta = c.sabes;
  while (meta < n && mejor(meta) < META) meta++;
  const faltan = meta - c.sabes;
  const ordenado = [...historial].sort((a, b) => a.ts.localeCompare(b.ts));
  const sesionesDeLaVentana = Math.min(VENTANA, historial.length) / TAMANO_SESION || 1;
  const ritmo = (c.sabes - conceptos(banco, ordenado.slice(0, -VENTANA)).sabes) / sesionesDeLaVentana;
  const blanco = probabilidad(c.sabes, null), contesta = probabilidad(c.sabes, tasa);
  return { ...c, total, dudo, tasaDudo, tasa, blanco, contesta, contestar: contesta > blanco,
    puntos: Math.round(100 * Math.abs(contesta - blanco)), hoy: Math.max(blanco, contesta),
    meta, faltan, ritmo, sesiones: faltan <= 0 ? 0 : ritmo > 0 ? Math.ceil(faltan / ritmo) : null };
}

export const resultado =(pregunta, eleccion) => (eleccion === null ? "blanco" : eleccion === pregunta.correcta ? "acierto" : "fallo");

export function nota(resultados, penalizacion) {
  const cuenta = (tipo) => resultados.filter((x) => x === tipo).length;
  return ((cuenta("acierto") - penalizacion * cuenta("fallo")) / resultados.length) * 10;
}

export const imagenDePagina = ({ pdf, pagina }) => encodeURI(`paginas/${pdf.replace(/\.pdf$/, "")}-${pagina}.png`);

export const pdfEnPagina = ({ pdf, pagina }) => `pdfs/${encodeURI(pdf)}#page=${pagina}`;

// L es el borde izquierdo, S el separador y R el borde derecho de las 2 columnas: de izquierda a derecha.
export const COLUMNAS = { L: 0.08, S: 0.5, R: 0.92 };
const COLUMNA_MINIMA = 0.15;

export function mover(pos, tirador, x) {
  const [min, max] = { L: [0, pos.S - COLUMNA_MINIMA], S: [pos.L + COLUMNA_MINIMA, pos.R - COLUMNA_MINIMA], R: [pos.S + COLUMNA_MINIMA, 1] }[tirador];
  return { ...pos, [tirador]: Math.min(max, Math.max(min, x)) };
}

// Un guardado es la ronda a medias en 4 campos y la hora del guardado. Sin ronda, ronda es null.
export const guardado = (ronda, ahora) => ({
  guardada: ahora,
  ronda: ronda && { ids: ronda.preguntas.map((q) => q.id), resultados: ronda.resultados, simulacro: ronda.simulacro, limite: ronda.limite ?? null },
});

export const masNueva = (a, b) => (b.guardada > a.guardada ? b : a);

// Una ronda con un id que no está en el banco se descarta. La pregunta actual es la primera sin resultado.
export function restaurar({ ronda }, banco) {
  if (!ronda) return null;
  const porId = new Map(banco.map((q) => [q.id, q]));
  if (!ronda.ids.every((id) => porId.has(id))) return null;
  return { preguntas: ronda.ids.map((id) => porId.get(id)), simulacro: ronda.simulacro, limite: ronda.limite, resultados: [...ronda.resultados], i: ronda.resultados.length };
}
