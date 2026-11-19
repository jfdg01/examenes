import { test } from "node:test";
import { readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { conceptos, probabilidadDeLaNota, textoParaCopiar, sesion, opciones, respuesta, porcentajeDudo, tarjetaDeLaNota, dominio, unir, simulacro, nota, imagenDePagina, pdfEnPagina, resultado, mover, COLUMNAS, guardado, masNueva, restaurar, semilla } from "./logica.js";

const r = (id, ts, acierto, seguro) => ({ id, ts, acierto, seguro });
const p = (id, tema = "rgpd") => ({ id, tema, concepto: id, correcta: "c", distractores: ["x", "y", "z"] });
const iso = (d) => d.toISOString().slice(0, 19) + "Z";
const dia = (d, h = 12) => new Date(2026, 10, d, h);
const ids = (s) => s.map((q) => q.id).sort();
const hermanas = (c, n) => Array.from({ length: n }, (_, k) => ({ ...p(c + k), concepto: c }));
// Un historial que da a cada respuesta [id, acierto, seguro] una hora creciente.
const seguidas = (...respuestas) => respuestas.map(([id, acierto, seguro], k) => r(id, new Date(k * 60000).toISOString(), acierto, seguro));

test("un acierto con seguro sube la P(L), y una pregunta repetida no es una prueba", () => {
  const banco = hermanas("a", 2);
  const una = dominio(banco, seguidas(["a0", true, true])).get("a");
  const repetida = dominio(banco, seguidas(["a0", true, true], ["a0", true, true])).get("a");
  const hermana = dominio(banco, seguidas(["a0", true, true], ["a1", true, true])).get("a");
  assert.ok(una > dominio(banco, []).get("a"));
  assert.equal(repetida, una);
  assert.ok(hermana > una);
});

test("un acierto con dudo es un no lo sé, y sube la P(L) menos que un acierto con seguro", () => {
  const banco = [...hermanas("a", 2), ...hermanas("b", 2)];
  const d = dominio(banco, seguidas(["a0", true, true], ["b0", true, false]));
  assert.ok(d.get("b") < d.get("a"));
  assert.equal(d.get("b"), dominio(banco, seguidas(["a0", true, true], ["b0", false, false])).get("b"));
});

test("el historial se ordena por fecha, venga de donde venga", () => {
  const banco = hermanas("a", 2);
  const h = seguidas(["a0", false, true], ["a1", true, true]);
  assert.equal(dominio(banco, [...h].reverse()).get("a"), dominio(banco, h).get("a"));
});

test("un concepto sin respuestas está sin ver, uno con la P(L) del 90 % o más lo sabes, y el resto no lo sabes", () => {
  const banco = [...hermanas("a", 2), ...hermanas("b", 2), ...hermanas("c", 2)];
  const aciertos = Array.from({ length: 8 }, (_, k) => [`a${k % 2}`, true, true]);
  const h = seguidas(...aciertos, ["b0", false, true]);
  assert.ok(dominio(banco, h).get("a") >= 0.9);
  assert.deepEqual(conceptos(banco, h), { sabes: 1, noSabes: 1, sinVer: 1 });
});

test("una sesión tiene 12 preguntas de 12 conceptos distintos, mezclados", () => {
  const banco = [...Array.from({ length: 10 }, (_, k) => hermanas("r" + k, 3)).flat(), ...Array.from({ length: 10 }, (_, k) => hermanas("e" + k, 3).map((q) => ({ ...q, tema: "ens" }))).flat()];
  for (let k = 0; k < 20; k++) {
    const s = sesion(banco, [], semilla(k));
    assert.equal(s.length, 12);
    assert.equal(new Set(s.map((q) => q.concepto)).size, 12);
  }
  assert.deepEqual([...new Set(sesion(banco, [], semilla(7)).map((q) => q.tema))].sort(), ["ens", "rgpd"]);
});

test("la sesión toma los 12 conceptos con la P(L) más baja", () => {
  const banco = Array.from({ length: 20 }, (_, k) => hermanas("c" + k, 2)).flat();
  const h = seguidas(...Array.from({ length: 8 }, (_, k) => [`c${k}0`, true, true]), ...Array.from({ length: 8 }, (_, k) => [`c${k}1`, true, true]));
  const d = dominio(banco, h);
  const s = sesion(banco, h, semilla(1)).map((q) => q.concepto);
  const fuera = [...d.keys()].filter((c) => !s.includes(c));
  assert.ok(Math.max(...s.map((c) => d.get(c))) <= Math.min(...fuera.map((c) => d.get(c))));
});

test("un concepto da una pregunta distinta de la última que contestaste", () => {
  const banco = hermanas("a", 2);
  for (let k = 0; k < 10; k++) assert.deepEqual(ids(sesion(banco, seguidas(["a1", true, true], ["a0", false, true]), semilla(k))), ["a1"]);
});

test("la probabilidad de una nota cuenta los aciertos menos 1/4 de los fallos, sobre 20 preguntas", () => {
  const cerca = (a, b) => assert.ok(Math.abs(a - b) < 1e-9, `${a} ≠ ${b}`);
  // Con una parte s sabida y las dudas contestadas con un acierto de q: acierto = s + (1 − s)·q y fallo = (1 − s)·(1 − q).
  const contesta = (s, q, objetivo) => probabilidadDeLaNota(s + (1 - s) * q, (1 - s) * (1 - q), objetivo);
  cerca(probabilidadDeLaNota(1, 0, 5), 1);
  cerca(probabilidadDeLaNota(0, 0, 5), 0);
  cerca(probabilidadDeLaNota(0.5, 0, 5), 0.5880985260009766);
  cerca(contesta(0.5, 0.25, 5), 0.6828711582842681);
  cerca(contesta(0, 0.25, 5), 0.000935391579332645);
  cerca(probabilidadDeLaNota(0.5, 0, 7.5), 0.020694732666015625);
  cerca(contesta(0.5, 0.25, 7.5), 0.07904140886862707);
  cerca(contesta(0.9, 0.25, 10), 0.21029776386659405);
});

test("la interfaz baraja las 4 opciones", () => {
  const q = p("a");
  const vistas = new Set();
  for (let k = 0; k < 20; k++) {
    const o = opciones(q, semilla(k));
    assert.deepEqual([...o].sort(), ["c", "x", "y", "z"]);
    vistas.add(o.indexOf("c"));
  }
  assert.equal(vistas.size, 4);
});

test("una respuesta lleva el id, la hora, el acierto y la confianza", () => {
  const ahora = new Date("2026-11-06T18:02:11.532Z");
  assert.deepEqual(respuesta(p("a"), "c", true, ahora), { id: "a", ts: "2026-11-06T18:02:11Z", acierto: true, seguro: true });
  assert.deepEqual(respuesta(p("a"), "x", false, ahora), { id: "a", ts: "2026-11-06T18:02:11Z", acierto: false, seguro: false });
});

test("una pregunta en blanco cuenta como un fallo con dudo", () => {
  assert.deepEqual(respuesta(p("a"), null, true, dia(6)), { id: "a", ts: iso(dia(6)), acierto: false, seguro: false });
});

test("el porcentaje de aciertos con dudo solo cuenta las respuestas con dudo", () => {
  const h = [r("a", "1", true, false), r("b", "2", false, false), r("c", "3", true, false), r("d", "4", true, true), r("e", "5", false, true)];
  assert.deepEqual(porcentajeDudo(h), { porcentaje: 200 / 3, total: 3, aciertos: 2 });
  assert.equal(porcentajeDudo([r("d", "4", true, true)]), null);
});

test("unir 2 historiales quita las respuestas repetidas", () => {
  const a = r("a", "2026-11-06T18:02:11Z", true, true);
  const b = r("b", "2026-11-06T18:03:00Z", false, false);
  assert.deepEqual(unir([a, b], [{ ...a }]), [a, b]);
});

test("el simulacro saca 20 preguntas distintas al azar de todo el banco", () => {
  const banco = Array.from({ length: 100 }, (_, k) => p("q" + k));
  const s = simulacro(banco, semilla(2));
  assert.equal(s.length, 20);
  assert.equal(new Set(s.map((q) => q.id)).size, 20);
  assert.notDeepEqual(ids(s), ids(simulacro(banco, semilla(9))));
  assert.equal(simulacro(banco.slice(0, 6), semilla(2)).length, 6);
});

test("la nota del simulacro es sobre 10: un acierto suma 0,5, un fallo resta 0,125 y un blanco no cuenta", () => {
  const res = [...Array(14).fill("acierto"), ...Array(3).fill("fallo"), ...Array(3).fill("blanco")];
  assert.equal(nota(res, 1 / 4), 6.625);
  assert.equal(nota(Array(20).fill("acierto"), 1 / 4), 10);
});

const ejemplo = JSON.parse(readFileSync(new URL("preguntas-ejemplo.json", import.meta.url)));

test("la hoja enlaza la imagen de la página y el PDF en esa página", () => {
  const fuente = { pdf: "2A _Teoría.pdf", pagina: 5 };
  assert.equal(imagenDePagina(fuente), "paginas/2A%20_Teor%C3%ADa-5.png");
  assert.equal(pdfEnPagina(fuente), "2A%20_Teor%C3%ADa.pdf#page=5");
});

test("el banco de ejemplo tiene 6 preguntas de 6 conceptos, y una sesión nueva las saca todas", () => {
  assert.equal(ejemplo.length, 6);
  assert.deepEqual(ids(sesion(ejemplo, [], semilla(1))), ids(ejemplo));
  assert.equal(simulacro(ejemplo, semilla(1)).length, 6);
});

test("la página no tiene ninguna pregunta en su código", () => {
  const codigo = ["index.html", "app.js", "logica.js"].map((f) => readFileSync(new URL(f, import.meta.url), "utf8")).join("\n");
  for (const q of ejemplo) {
    assert.ok(!codigo.includes(q.pregunta), q.id);
    assert.ok(!codigo.includes(q.correcta), q.id);
  }
});

test("una respuesta del simulacro es un acierto, un fallo o una pregunta en blanco", () => {
  assert.equal(resultado(p("a"), "c"), "acierto");
  assert.equal(resultado(p("a"), "x"), "fallo");
  assert.equal(resultado(p("a"), null), "blanco");
});

test("un tirador se mueve, y cada columna mide al menos el 15 % del ancho", () => {
  const cerca = (real, esperado) => assert.ok(Math.abs(real - esperado) < 1e-9, `${real} no es ${esperado}`);
  assert.deepEqual(mover(COLUMNAS, "L", 0.2), { L: 0.2, S: 0.5, R: 0.92 });
  cerca(mover(COLUMNAS, "L", 0.9).L, 0.35);
  cerca(mover(COLUMNAS, "L", -0.1).L, 0);
  cerca(mover(COLUMNAS, "S", 0.99).S, 0.77);
  cerca(mover(COLUMNAS, "S", 0).S, 0.23);
  cerca(mover(COLUMNAS, "R", 0.1).R, 0.65);
  cerca(mover(COLUMNAS, "R", 1.5).R, 1);
});

test("el texto para copiar lleva el enunciado y una letra por opción, sin la solución", () => {
  const q = { pregunta: "¿Cuál es?", correcta: "uno", distractores: ["dos", "tres"], explicacion: "porque sí" };
  const texto = textoParaCopiar(q, ["dos", "uno", "tres"]);
  assert.equal(texto, "¿Cuál es?\n\nA) dos\nB) uno\nC) tres");
});

// Un banco de n conceptos con 2 preguntas cada uno, y un historial de [concepto, acierto, seguro] que alterna sus 2 preguntas.
const bancoDe = (n) => Array.from({ length: n }, (_, i) => hermanas(`c${i}`, 2)).flat();
const historialDe = (...respuestas) => {
  const vistas = new Map();
  return seguidas(...respuestas.map(([i, acierto, seguro]) => {
    vistas.set(i, (vistas.get(i) ?? -1) + 1);
    return [`c${i}${vistas.get(i) % 2}`, acierto, seguro];
  }));
};
// Un concepto aprendido empieza con un "no lo sé" y sigue con aciertos con seguro. Sin los fallos, el ajuste
// no separa los 2 estados, y aprende que un concepto sin ver ya se sabe.
const aprendidos = (desde, n, aciertos = 6) => Array.from({ length: n }, (_, i) => [[desde + i, false, false], ...Array.from({ length: aciertos }, () => [desde + i, true, true])]).flat();
const sabidos = (n) => aprendidos(0, n);

test("la tarjeta sin historial acierta una duda a ciegas, 1 de 4, y no tiene ritmo", () => {
  const t = tarjetaDeLaNota(bancoDe(20), [], 5);
  assert.equal(t.dudo, null);
  assert.equal(t.tasaDudo, 1 / 4);
  assert.equal(t.sinVer, 20);
  assert.equal(t.sesiones, null);
});

test("si sabes todos los conceptos, un 10 es posible y un 9 es casi seguro", () => {
  const banco = bancoDe(20), h = historialDe(...sabidos(20));
  assert.equal(tarjetaDeLaNota(banco, h, 5).sabes, 20);
  assert.ok(tarjetaDeLaNota(banco, h, 10).hoy > 0.2);
  assert.ok(tarjetaDeLaNota(banco, h, 9).hoy > 0.9);
});

test("con un 95 % o más hoy no faltan conceptos ni sesiones", () => {
  const t = tarjetaDeLaNota(bancoDe(20), historialDe(...sabidos(20)), 5);
  assert.ok(t.hoy >= 0.95);
  assert.equal(t.faltan, 0);
  assert.equal(t.meta, 20);
  assert.equal(t.sesiones, 0);
});

test("con las dudas siempre falladas, el consejo deja las dudas en blanco, y da la diferencia en puntos", () => {
  const h = historialDe(...sabidos(10), ...Array.from({ length: 10 }, (_, i) => [10 + i, false, false]));
  const t = tarjetaDeLaNota(bancoDe(20), h, 5);
  assert.equal(t.tasaDudo, 0);
  assert.equal(t.contestar, false);
  assert.equal(t.hoy, t.blanco);
  assert.equal(t.puntos, Math.round(100 * (t.blanco - t.contesta)));
});

test("el camino busca los conceptos que faltan para un 95 %, y dice si ni con todos llega", () => {
  const banco = bancoDe(20), h = historialDe(...sabidos(4), ...Array.from({ length: 8 }, (_, i) => [4 + i, false, false]));
  const t = tarjetaDeLaNota(banco, h, 7);
  assert.equal(t.sabes, 4);
  assert.ok(t.hoy < 0.95);
  assert.ok(t.alcanza && t.enMeta >= 0.95);
  assert.ok(t.meta > 4 && t.meta <= 20);
  assert.equal(t.faltan, t.meta - 4);
  const diez = tarjetaDeLaNota(banco, h, 10);
  assert.equal(diez.meta, 20);
  assert.equal(diez.alcanza, false);
});

test("el ritmo cuenta los conceptos sabidos esperados que ganaron las 60 últimas respuestas, 5 sesiones de 12", () => {
  const banco = bancoDe(100);
  const viejo = aprendidos(0, 30, 2), nuevo = aprendidos(30, 20, 2);
  const t = tarjetaDeLaNota(banco, historialDe(...viejo, ...nuevo), 7);
  // Las 60 últimas respuestas son las de los 20 conceptos nuevos, y solo sus P(L) suben en la ventana.
  assert.equal(nuevo.length, 60);
  assert.ok(t.ritmo > 0);
  assert.ok(t.faltan > 0);
  assert.ok(t.sesiones > 0 && Number.isInteger(t.sesiones));
});

test("con un ritmo de 0, las sesiones no se pueden calcular", () => {
  const t = tarjetaDeLaNota(bancoDe(20), historialDe([0, false, true]), 10);
  assert.ok(t.faltan > 0);
  assert.equal(t.sesiones, null);
});

test("con un banco vacío no faltan conceptos", () => {
  const t = tarjetaDeLaNota([], [], 5);
  assert.equal(t.faltan, 0);
  assert.equal(t.sesiones, 0);
});

const rondaDe = (preguntas, resultados, simulacro = false, limite = null) => ({ preguntas, resultados, simulacro, limite, i: resultados.length });

test("un guardado lleva los id de la ronda, sus resultados, el simulacro y el límite, con la hora del guardado", () => {
  const banco = JSON.parse(readFileSync("./preguntas-ejemplo.json", "utf8"));
  const g = guardado(rondaDe(banco.slice(0, 2), ["acierto"], true, 500), 123);
  assert.deepEqual(g, { guardada: 123, ronda: { ids: [banco[0].id, banco[1].id], resultados: ["acierto"], simulacro: true, limite: 500 } });
  assert.deepEqual(guardado(null, 124), { guardada: 124, ronda: null });
});

test("de 2 copias gana la guardada más nueva", () => {
  const vieja = { guardada: 1, ronda: null }, nueva = { guardada: 2, ronda: { ids: [], resultados: [], simulacro: false, limite: null } };
  assert.equal(masNueva(vieja, nueva), nueva);
  assert.equal(masNueva(nueva, vieja), nueva);
});

test("restaurar abre la primera pregunta sin resultado, en el orden guardado", () => {
  const banco = JSON.parse(readFileSync("./preguntas-ejemplo.json", "utf8"));
  const orden = [banco[3], banco[1], banco[2]];
  const ronda = restaurar(guardado(rondaDe(orden, ["fallo", "blanco"], true, 500), 1), banco);
  assert.deepEqual(ronda.preguntas.map((q) => q.id), orden.map((q) => q.id));
  assert.equal(ronda.i, 2);
  assert.deepEqual(ronda.resultados, ["fallo", "blanco"]);
  assert.equal(ronda.simulacro, true);
  assert.equal(ronda.limite, 500);
});

test("sin ronda a medias, o con un id que no está en el banco, restaurar devuelve null", () => {
  const banco = JSON.parse(readFileSync("./preguntas-ejemplo.json", "utf8"));
  assert.equal(restaurar({ guardada: 1, ronda: null }, banco), null);
  assert.equal(restaurar({ guardada: 1, ronda: { ids: [banco[0].id, "no-existe"], resultados: [], simulacro: false, limite: null } }, banco), null);
});
