export function cajas(historial) {
  const estado = new Map();
  for (const r of [...historial].sort((a, b) => a.ts.localeCompare(b.ts))) {
    const caja = estado.get(r.id)?.caja ?? 1;
    estado.set(r.id, { caja: r.acierto && r.seguro ? Math.min(caja + 1, 3) : 1, ts: r.ts });
  }
  return estado;
}

const DIAS = { 2: 2, 3: 4 };
const TAMANO_SESION = 25;

export function barajar(lista, azar = Math.random) {
  const copia = [...lista];
  for (let i = copia.length - 1; i > 0; i--) {
    const j = Math.floor(azar() * (i + 1));
    [copia[i], copia[j]] = [copia[j], copia[i]];
  }
  return copia;
}

function vuelve({ caja, ts }) {
  const fecha = new Date(ts);
  fecha.setHours(0, 0, 0, 0);
  fecha.setDate(fecha.getDate() + DIAS[caja]);
  return fecha;
}

export function sesion(banco, historial, ahora, azar = Math.random) {
  const estado = cajas(historial);
  const falladas = [], nuevas = [], repaso = [];
  for (const q of banco) {
    const e = estado.get(q.id);
    if (!e) nuevas.push(q);
    else if (e.caja === 1) falladas.push(q);
    else if (vuelve(e) <= ahora) repaso.push(q);
  }
  return [falladas, nuevas, repaso].flatMap((grupo) => barajar(grupo, azar)).slice(0, TAMANO_SESION);
}

export function proximaFecha(banco, historial) {
  const estado = cajas(historial);
  const fechas = banco.map((q) => estado.get(q.id)).filter((e) => e && e.caja > 1).map(vuelve);
  return fechas.length ? new Date(Math.min(...fechas)) : null;
}

export const opciones = (pregunta, azar = Math.random) => barajar([pregunta.correcta, ...pregunta.distractores], azar);

export function respuesta(pregunta, eleccion, seguro, ahora) {
  const ts = ahora.toISOString().slice(0, 19) + "Z";
  if (eleccion === null) return { id: pregunta.id, ts, acierto: false, seguro: false };
  return { id: pregunta.id, ts, acierto: eleccion === pregunta.correcta, seguro };
}

export function porcentajeDudo(historial) {
  const dudas = historial.filter((r) => !r.seguro);
  if (!dudas.length) return null;
  const porcentaje = (100 * dudas.filter((r) => r.acierto).length) / dudas.length;
  return { porcentaje, total: dudas.length, contestar: porcentaje > 33 };
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

const TAMANO_SIMULACRO = 40;

export const simulacro = (banco, azar = Math.random) => barajar(banco, azar).slice(0, TAMANO_SIMULACRO);

export const resultado = (pregunta, eleccion) => (eleccion === null ? "blanco" : eleccion === pregunta.correcta ? "acierto" : "fallo");

export function nota(resultados, penalizacion) {
  const cuenta = (tipo) => resultados.filter((x) => x === tipo).length;
  return ((cuenta("acierto") - penalizacion * cuenta("fallo")) / resultados.length) * 10;
}

export const imagenDePagina = ({ pdf, pagina }) => encodeURI(`paginas/${pdf.replace(/\.pdf$/, "")}-${pagina}.png`);

export const pdfEnPagina = ({ pdf, pagina }) => `${encodeURI(pdf)}#page=${pagina}`;

export const COLUMNAS = { L: 0.08, S: 0.5, R: 0.92 };
const COLUMNA_MINIMA = 0.15;

export function mover(pos, tirador, x) {
  const [min, max] = { L: [0, pos.S - COLUMNA_MINIMA], S: [pos.L + COLUMNA_MINIMA, pos.R - COLUMNA_MINIMA], R: [pos.S + COLUMNA_MINIMA, 1] }[tirador];
  return { ...pos, [tirador]: Math.min(max, Math.max(min, x)) };
}
