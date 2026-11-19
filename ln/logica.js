// El BKT (Bayesian Knowledge Tracing) da a cada concepto la probabilidad de saberlo, P(L).
// Una respuesta tiene 3 salidas: 0 = lo sé y acierto (acierto con seguro), 1 = creo que lo sé y fallo
// (fallo con seguro) y 2 = no lo sé (dudo o blanco). Un acierto con dudo es una adivinanza.
const salida = (r) => (!r.seguro ? 2 : r.acierto ? 0 : 1);

export function semilla(n) {
  return () => {
    n = (n + 0x6d2b79f5) | 0;
    let t = Math.imul(n ^ (n >>> 15), 1 | n);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

// Una prueba es una respuesta a una pregunta distinta de la anterior del concepto. Una pregunta repetida
// enseña su respuesta, y no prueba el concepto.
function pruebas(banco, historial) {
  const concepto = new Map(banco.map((q) => [q.id, q.concepto]));
  const lista = new Map(banco.map((q) => [q.concepto, []])), ultima = new Map();
  for (const r of [...historial].sort((a, b) => a.ts.localeCompare(b.ts))) {
    const c = concepto.get(r.id);
    if (c === undefined) continue;
    if (ultima.get(c) !== r.id) lista.get(c).push(salida(r));
    ultima.set(c, r.id);
  }
  return { lista, ultima };
}

// El ajuste es un EM (Baum-Welch con 2 estados) con parámetros globales: L0 (saberlo antes de verlo),
// T (aprenderlo en una prueba), e1 (las salidas si lo sabes) y e0 (si no lo sabes).
// Los pseudo-counts del prior pesan 10 pruebas en e1, 5 en e0, 2 en L0 y 10 en T.
// ponytail: sin cotas explícitas para G y S; el prior las mantiene lejos de 0,5. Añade cotas si un ajuste degenera.
export function ajustar(secuencias, vueltas = 100) {
  let L0 = 0.5, T = 0.1, e1 = [0.9, 0.05, 0.05], e0 = [0.3, 0.2, 0.5];
  for (let it = 0; it < vueltas; it++) {
    let nL0 = 1, dL0 = 2, nT = 1, dT = 10;
    const c1 = [9.5, 0.2, 0.3], c0 = [1.5, 1, 2.5];
    for (const s of secuencias) {
      const n = s.length, a = [], b = Array(n);
      let p = [1 - L0, L0];
      for (let t = 0; t < n; t++) {
        if (t) p = [a[t - 1][0] * (1 - T), a[t - 1][0] * T + a[t - 1][1]];
        const x = [p[0] * e0[s[t]], p[1] * e1[s[t]]], z = x[0] + x[1];
        a.push([x[0] / z, x[1] / z]);
      }
      b[n - 1] = [1, 1];
      for (let t = n - 2; t >= 0; t--) {
        const o = s[t + 1], u = [b[t + 1][0] * e0[o], b[t + 1][1] * e1[o]];
        const v = [(1 - T) * u[0] + T * u[1], u[1]], z = v[0] + v[1];
        b[t] = [v[0] / z, v[1] / z];
      }
      for (let t = 0; t < n; t++) {
        const y0 = a[t][0] * b[t][0], y1 = a[t][1] * b[t][1], z = y0 + y1;
        c0[s[t]] += y0 / z; c1[s[t]] += y1 / z;
        if (!t) { nL0 += y1 / z; dL0++; continue; }
        const o = s[t], w00 = a[t - 1][0] * (1 - T) * e0[o] * b[t][0], w01 = a[t - 1][0] * T * e1[o] * b[t][1];
        const w = w00 + w01 + a[t - 1][1] * e1[o] * b[t][1];
        nT += w01 / w; dT += (w00 + w01) / w;
      }
    }
    L0 = nL0 / dL0; T = nT / dT;
    const s1 = c1[0] + c1[1] + c1[2], s0 = c0[0] + c0[1] + c0[2];
    e1 = c1.map((x) => x / s1); e0 = c0.map((x) => x / s0);
  }
  return { L0, T, e1, e0 };
}

const probabilidad = (secuencia, { L0, T, e1, e0 }) => secuencia.reduce((p, o) => {
  const q = (p * e1[o]) / (p * e1[o] + (1 - p) * e0[o]);
  return q + (1 - q) * T;
}, L0);

// La app pide el modelo varias veces por pantalla, así que el último ajuste queda guardado.
let guardadoDelModelo = null;
function modelo(banco, historial) {
  const g = guardadoDelModelo;
  if (g && g.banco === banco && g.historial === historial && g.n === historial.length) return g.m;
  const { lista, ultima } = pruebas(banco, historial);
  const parametros = ajustar([...lista.values()].filter((s) => s.length));
  const pl = new Map([...lista].map(([c, s]) => [c, probabilidad(s, parametros)]));
  const m = { lista, ultima, parametros, pl };
  guardadoDelModelo = { banco, historial, n: historial.length, m };
  return m;
}

// Lo sabes con una probabilidad del 90 % o más.
const SABES = 0.9;
const TAMANO_SESION = 12;

export function barajar(lista, azar = Math.random) {
  const copia = [...lista];
  for (let i = copia.length - 1; i > 0; i--) {
    const j = Math.floor(azar() * (i + 1));
    [copia[i], copia[j]] = [copia[j], copia[i]];
  }
  return copia;
}

// La sesión toma los 12 conceptos con la P(L) más baja, porque una prueba gana T · (1 − P(L)).
// Cada concepto da una pregunta distinta de la última que contestaste.
export function sesion(banco, historial, azar = Math.random) {
  const { pl, ultima } = modelo(banco, historial);
  const preguntas = Map.groupBy(banco, (q) => q.concepto);
  const elegidos = barajar([...preguntas.keys()], azar).sort((a, b) => pl.get(a) - pl.get(b)).slice(0, TAMANO_SESION);
  return barajar(elegidos.map((c) => {
    const otras = preguntas.get(c).filter((q) => q.id !== ultima.get(c));
    return barajar(otras.length ? otras : preguntas.get(c), azar)[0];
  }), azar);
}

const cuentaConceptos = (lista, pl) => {
  const cuenta = { sabes: 0, noSabes: 0, sinVer: 0 };
  for (const [c, s] of lista) cuenta[!s.length ? "sinVer" : pl.get(c) >= SABES ? "sabes" : "noSabes"]++;
  return cuenta;
};

export const conceptos = (banco, historial) => {
  const { lista, pl } = modelo(banco, historial);
  return cuentaConceptos(lista, pl);
};

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

// El dominio de un concepto es su P(L).
export const dominio = (banco, historial) => modelo(banco, historial).pl;

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

const combinaciones = (n, k) => {
  let c = 1;
  for (let i = 0; i < k; i++) c = (c * (n - i)) / (i + 1);
  return c;
};

// acierto y fallo son las probabilidades de acertar y de fallar una pregunta, y el resto queda en blanco.
// objetivo es la nota sobre 10, y pide objetivo * n / 10 aciertos netos.
export function probabilidadDeLaNota(acierto, fallo, objetivo, n = TAMANO_SIMULACRO) {
  const netos = (objetivo * n) / 10, blanco = Math.max(0, 1 - acierto - fallo);
  let total = 0;
  for (let a = 0; a <= n; a++)
    for (let f = 0; a + f <= n; f++)
      if (a - PENALIZACION * f >= netos) total += combinaciones(n, a) * combinaciones(n - a, f) * acierto ** a * fallo ** f * blanco ** (n - a - f);
  return total;
}

const META = 0.95;
const VENTANA = 60;
const A_CIEGAS = 1 / 4;
const REMUESTRAS = 30;

// El bootstrap: cada remuestra ajusta los parámetros con los conceptos vistos, sacados con reemplazo,
// y da la P(L) de cada concepto con esos parámetros. La semilla fija da la misma tarjeta con el mismo historial.
function remuestras(m) {
  if (m.remuestras) return m.remuestras;
  const vistas = [...m.lista.values()].filter((s) => s.length), azar = semilla(1);
  m.remuestras = Array.from({ length: REMUESTRAS }, () => {
    const parametros = ajustar(vistas.map(() => vistas[Math.floor(azar() * vistas.length)]), 50);
    return { parametros, pl: [...m.lista.values()].map((s) => probabilidad(s, parametros)) };
  });
  return m.remuestras;
}

// La tarjeta de la nota: la probabilidad de hoy y de la forma con más probabilidad, los conceptos y las sesiones que faltan para un 95 %.
// Cada pregunta del examen cae en un concepto al azar del banco. Un concepto da sus salidas con su P(L), y una duda
// queda en blanco o se contesta con el porcentaje de aciertos con dudo.
export function tarjetaDeLaNota(banco, historial, objetivo) {
  const m = modelo(banco, historial), cuenta = cuentaConceptos(m.lista, m.pl);
  const n = m.lista.size, total = n || 1;
  const dudo = porcentajeDudo(historial);
  const tasaDudo = dudo ? dudo.porcentaje / 100 : A_CIEGAS;
  const orden = [...m.pl.values()].map((p, i) => [p, i]).sort((a, b) => b[0] - a[0]).map(([, i]) => i);
  const muestras = remuestras(m).map(({ parametros: { e1, e0 }, pl }) => {
    const resto = [0];
    for (let k = n - 1; k >= 0; k--) resto.unshift(resto[0] + pl[orden[k]]);
    return { e1, e0, resto };
  });
  // Con k sabidos, los k conceptos con la P(L) más alta pasan a P(L) = 1, y el resto sigue con la suya.
  const probabilidadCon = (k, contesta) => muestras.reduce((suma, { e1, e0, resto }) => {
    const sabe = k + resto[k], [a, f, d] = e1.map((x, o) => (sabe * x + (n - sabe) * e0[o]) / total);
    const q = contesta ? tasaDudo : 0;
    return suma + probabilidadDeLaNota(a + q * d, f + (contesta ? (1 - q) * d : 0), objetivo);
  }, 0) / muestras.length;
  const blanco = probabilidadCon(0, false), contesta = probabilidadCon(0, true), hoy = Math.max(blanco, contesta);
  const mejor = (k) => Math.max(probabilidadCon(k, false), probabilidadCon(k, true));
  let meta = cuenta.sabes;
  if (hoy < META && meta < n) {
    let alto = n;
    meta++;
    while (meta < alto) { const medio = (meta + alto) >> 1; if (mejor(medio) >= META) alto = medio; else meta = medio + 1; }
  }
  const enMeta = hoy >= META ? hoy : mejor(meta);
  const faltan = meta - cuenta.sabes;
  // El ritmo y las sesiones cuentan los conceptos sabidos esperados, la suma de las P(L). Un concepto llega al 90 %
  // tras varias pruebas, así que la cuenta de "sabes" queda quieta durante sesiones con progreso.
  const esperados = (pl) => [...pl.values()].reduce((x, y) => x + y, 0);
  const ordenado = [...historial].sort((a, b) => a.ts.localeCompare(b.ts));
  const antes = pruebas(banco, ordenado.slice(0, -VENTANA)).lista;
  const ganancia = esperados(m.pl) - esperados(new Map([...antes].map(([c, s]) => [c, probabilidad(s, m.parametros)])));
  const ritmo = ganancia / (Math.min(VENTANA, historial.length) / TAMANO_SESION || 1);
  const pl = [...m.pl.values()], faltanEsperados = orden.slice(0, meta).reduce((x, i) => x + 1 - pl[i], 0);
  return { ...cuenta, total, dudo, tasaDudo, blanco, contesta, contestar: contesta > blanco,
    puntos: Math.round(100 * Math.abs(contesta - blanco)), hoy, meta, enMeta, alcanza: enMeta >= META,
    faltan, ritmo, sesiones: faltan <= 0 ? 0 : ritmo > 0 ? Math.ceil(faltanEsperados / ritmo) : null };
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
