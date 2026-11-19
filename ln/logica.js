const enOrden = (historial) => [...historial].sort((a, b) => a.ts.localeCompare(b.ts));
const sabeTras = (r) => r.acierto && r.seguro;

// La caja es del concepto. Una respuesta huérfana no cuenta.
export function cajas(banco, historial) {
  const concepto = new Map(banco.map((q) => [q.id, q.concepto]));
  const estado = new Map();
  for (const r of enOrden(historial)) {
    const k = concepto.get(r.id);
    if (k === undefined) continue;
    const caja = estado.get(k)?.caja ?? 1;
    estado.set(k, { caja: sabeTras(r) ? Math.min(caja + 1, 3) : 1, ts: r.ts });
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

// Cada concepto sale con la pregunta que lleva más tiempo sin salir. Entre las preguntas que nunca salieron, gana una al azar.
export function sesion(banco, historial, ahora, azar = Math.random) {
  const estado = cajas(banco, historial);
  const bloque = principioDelBloque(historial, ahora);
  const ultima = new Map();
  for (const r of historial) if (!(ultima.get(r.id) >= r.ts)) ultima.set(r.id, r.ts);
  const pregunta = new Map();
  for (const q of barajar(banco, azar)) {
    const otra = pregunta.get(q.concepto);
    if (!otra || (ultima.get(q.id) ?? "") < (ultima.get(otra.id) ?? "")) pregunta.set(q.concepto, q);
  }
  const caja1 = [], sinVer = [], sorteo = [];
  for (const [k, q] of pregunta) {
    const e = estado.get(k);
    if (!e) sinVer.push(q);
    else if (e.caja === 1) caja1.push(q);
    else if (Date.parse(e.ts) < bloque) sorteo.push({ q, clave: -Math.log(azar()) / PESO[e.caja] });
  }
  const elegidas = sorteo.sort((a, b) => a.clave - b.clave).map((x) => x.q);
  return barajar([...caja1, ...sinVer, ...elegidas].slice(0, TAMANO_SESION), azar);
}

export function conceptos(banco, historial) {
  const concepto = new Map(banco.map((q) => [q.id, q.concepto]));
  const estado = new Map(banco.map((q) => [q.concepto, "sinVer"]));
  for (const r of enOrden(historial))
    if (concepto.has(r.id)) estado.set(concepto.get(r.id), sabeTras(r) ? "sabes" : "noSabes");
  const cuenta = { sabes: 0, noSabes: 0, sinVer: 0 };
  for (const e of estado.values()) cuenta[e]++;
  return cuenta;
}

export const opciones = (pregunta, azar = Math.random) => barajar([pregunta.correcta, ...pregunta.distractores], azar);

export const textoParaCopiar = (pregunta, ops) => `${pregunta.pregunta}\n\n${ops.map((op, k) => `${String.fromCharCode(65 + k)}) ${op}`).join("\n")}`;

export function respuesta(pregunta, eleccion, seguro, ahora) {
  const ts = ahora.toISOString().slice(0, 19) + "Z";
  if (eleccion === null) return { id: pregunta.id, ts, acierto: false, seguro: false, blanco: true };
  return { id: pregunta.id, ts, acierto: eleccion === pregunta.correcta, seguro };
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

const comb = (n, k) => {
  let c = 1;
  for (let i = 0; i < k; i++) c = (c * (n - i)) / (i + 1);
  return c;
};

// acierto y fallo son las probabilidades de +1 y de −1/4 en una pregunta. El resto de cada pregunta vale 0.
// objetivo es la nota sobre 10, y pide objetivo * n / 10 aciertos netos.
export function probabilidadDeLaNota(acierto, fallo, objetivo, n = TAMANO_SIMULACRO) {
  const netos = (objetivo * n) / 10, cero = Math.max(0, 1 - acierto - fallo);
  let total = 0;
  for (let a = 0; a <= n; a++)
    for (let f = 0; f <= n - a && a - PENALIZACION * f >= netos; f++)
      total += comb(n, a) * comb(n - a, f) * acierto ** a * fallo ** f * cero ** (n - a - f);
  return total;
}

// Los 5 resultados de una prueba, en este orden: acierto y fallo con "seguro", acierto y fallo con "dudo", y blanco.
const resultadoDe = (r) => (r.blanco ? 4 : r.seguro ? (r.acierto ? 0 : 1) : r.acierto ? 2 : 3);
const PUNTOS = { contesta: [1, -PENALIZACION, 1, -PENALIZACION, 0], blanco: [1, -PENALIZACION, 0, 0, 0] };
const ESTADOS = ["sabes", "noSabes", "sinVer"];
const PRIOR = { sabes: [2, 0, 0, 0, 0], noSabes: [0, 0, 0.5, 1.5, 0], sinVer: [0, 0, 0.5, 1.5, 0] };

// Una prueba es una respuesta a una pregunta distinta de la anterior de su concepto, contada en el estado del concepto antes de la respuesta.
export function pruebas(banco, historial) {
  const concepto = new Map(banco.map((q) => [q.id, q.concepto]));
  const estado = new Map(), anterior = new Map();
  const cuenta = Object.fromEntries(ESTADOS.map((e) => [e, [0, 0, 0, 0, 0]]));
  for (const r of enOrden(historial)) {
    const k = concepto.get(r.id);
    if (k === undefined) continue;
    if (anterior.get(k) !== r.id) cuenta[estado.get(k) ?? "sinVer"][resultadoDe(r)]++;
    estado.set(k, sabeTras(r) ? "sabes" : "noSabes");
    anterior.set(k, r.id);
  }
  return cuenta;
}

export function semilla(n) {
  return () => {
    n = (n + 0x6d2b79f5) | 0;
    let t = Math.imul(n ^ (n >>> 15), 1 | n);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const normal = (azar) => Math.sqrt(-2 * Math.log(1 - azar())) * Math.cos(2 * Math.PI * azar());

// Marsaglia y Tsang. Una forma menor que 1 usa gamma(a + 1) · U^(1/a).
function gamma(a, azar) {
  if (a <= 0) return 0;
  if (a < 1) return gamma(a + 1, azar) * azar() ** (1 / a);
  const d = a - 1 / 3, c = 1 / Math.sqrt(9 * d);
  for (;;) {
    let x, v;
    do { x = normal(azar); v = 1 + c * x; } while (v <= 0);
    v = v ** 3;
    if (Math.log(azar()) < 0.5 * x * x + d - d * v + d * Math.log(v)) return d * v;
  }
}

const dirichlet = (alfa, azar) => {
  const g = alfa.map((a) => gamma(a, azar));
  const s = g.reduce((x, y) => x + y);
  return g.map((x) => x / s);
};

const META = 0.95;
const VENTANA = 60;
const MUESTRAS = 500;

// La tarjeta de la nota: la probabilidad de hoy y de la forma con más probabilidad, los conceptos y las sesiones que faltan para un 95 %.
export function tarjetaDeLaNota(banco, historial, objetivo) {
  const c = conceptos(banco, historial);
  const n = c.sabes + c.noSabes + c.sinVer, total = n || 1, resto = c.noSabes + c.sinVer || 1;
  const cuenta = pruebas(banco, historial);
  const alfa = Object.fromEntries(ESTADOS.map((e) => [e, PRIOR[e].map((x, i) => x + cuenta[e][i])]));
  const azar = semilla(1);
  const muestras = Array.from({ length: MUESTRAS }, () => ESTADOS.map((e) => dirichlet(alfa[e], azar)));
  // Con s conceptos en "lo sabes", los demás quedan en "no lo sabes" y "sin ver" con la proporción de hoy.
  const probabilidad = (s, forma) => {
    const peso = [s / total, ((total - s) / total) * (c.noSabes / resto), ((total - s) / total) * (c.sinVer / resto)];
    let suma = 0;
    for (const m of muestras) {
      let acierto = 0, fallo = 0;
      m.forEach((theta, e) => theta.forEach((p, i) => {
        if (PUNTOS[forma][i] === 1) acierto += peso[e] * p;
        else if (PUNTOS[forma][i] < 0) fallo += peso[e] * p;
      }));
      suma += probabilidadDeLaNota(acierto, fallo, objetivo);
    }
    return suma / MUESTRAS;
  };
  const mejor = (s) => Math.max(probabilidad(s, "blanco"), probabilidad(s, "contesta"));
  let meta = c.sabes;
  while (meta < n && mejor(meta) < META) meta++;
  const alcanza = n === 0 || mejor(meta) >= META;
  const faltan = meta - c.sabes;
  const sesionesDeLaVentana = Math.min(VENTANA, historial.length) / TAMANO_SESION || 1;
  const ritmo = (c.sabes - conceptos(banco, enOrden(historial).slice(0, -VENTANA)).sabes) / sesionesDeLaVentana;
  const blanco = probabilidad(c.sabes, "blanco"), contesta = probabilidad(c.sabes, "contesta");
  const suma = (xs) => xs.reduce((x, y) => x + y, 0);
  // aciertos es la media de la Dirichlet: el prior y las pruebas juntos.
  const filas = Object.fromEntries(ESTADOS.map((e) => [e, {
    aciertos: (alfa[e][0] + alfa[e][2]) / suma(alfa[e]), vistos: cuenta[e][0] + cuenta[e][2], pruebas: suma(cuenta[e]) }]));
  return { ...c, total, filas, blanco, contesta, contestar: contesta > blanco,
    puntos: Math.round(100 * Math.abs(contesta - blanco)), hoy: Math.max(blanco, contesta),
    meta, alcanza, faltan, ritmo, sesiones: !alcanza ? null : faltan <= 0 ? 0 : ritmo > 0 ? Math.ceil(faltan / ritmo) : null };
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
