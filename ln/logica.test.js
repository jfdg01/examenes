import { test } from "node:test";
import { readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { cajas, conceptos, probabilidadDeLaNota, pruebas, semilla, textoParaCopiar, sesion, opciones, respuesta, tarjetaDeLaNota, unir, simulacro, nota, imagenDePagina, pdfEnPagina, resultado, mover, COLUMNAS, guardado, masNueva, restaurar } from "./logica.js";

const r = (id, ts, acierto, seguro) => ({ id, ts, acierto, seguro });
const p = (id, tema = "rgpd") => ({ id, tema, concepto: id, correcta: "c", distractores: ["x", "y", "z"] });

test("un acierto con seguro sube una caja, hasta la 3", () => {
  const h = [
    r("a", "2026-11-01T10:00:00Z", true, true),
    r("a", "2026-11-03T10:00:00Z", true, true),
    r("a", "2026-11-07T10:00:00Z", true, true),
  ];
  assert.equal(cajas([p("a")], h).get("a").caja, 3);
  assert.equal(cajas([p("a")], h.slice(0, 1)).get("a").caja, 2);
});

test("un fallo o un acierto con dudo vuelve a la caja 1", () => {
  const sube = [r("a", "2026-11-01T10:00:00Z", true, true), r("a", "2026-11-03T10:00:00Z", true, true)];
  for (const final of [r("a", "2026-11-07T10:00:00Z", false, true), r("a", "2026-11-07T10:00:00Z", true, false), r("a", "2026-11-07T10:00:00Z", false, false)])
    assert.equal(cajas([p("a")], [...sube, final]).get("a").caja, 1);
});

test("el historial se ordena por fecha, venga de donde venga", () => {
  const h = [r("a", "2026-11-03T10:00:00Z", false, true), r("a", "2026-11-01T10:00:00Z", true, true)];
  assert.equal(cajas([p("a")], h).get("a").caja, 1);
  assert.equal(cajas([p("a")], h).get("a").ts, "2026-11-03T10:00:00Z");
});

const iso = (d) => d.toISOString().slice(0, 19) + "Z";
const dia = (d, h = 12) => new Date(2026, 10, d, h);
const ids = (s) => s.map((q) => q.id).sort();

test("una pregunta nueva sale en la sesión", () => {
  assert.deepEqual(ids(sesion([p("a")], [], dia(6), semilla(1))), ["a"]);
});

test("una pregunta de la caja 2 o de la caja 3 no sale en su bloque, y sale después de una pausa de 3 horas", () => {
  const caja2 = [r("a", iso(dia(6, 10)), true, true)];
  assert.deepEqual(ids(sesion([p("a")], caja2, dia(6, 12), semilla(1))), []);
  assert.deepEqual(ids(sesion([p("a")], caja2, dia(6, 13), semilla(1))), ["a"]);
  const caja3 = [r("a", iso(dia(5)), true, true), r("a", iso(dia(6, 10)), true, true)];
  assert.deepEqual(ids(sesion([p("a")], caja3, dia(6, 12), semilla(1))), []);
  assert.deepEqual(ids(sesion([p("a")], caja3, dia(6, 13), semilla(1))), ["a"]);
});

test("un bloque sigue mientras las pausas entre respuestas no llegan a 3 horas", () => {
  const hist = [r("a", iso(dia(6, 10)), true, true), r("b", iso(dia(6, 12)), false, true), r("b", iso(dia(6, 14)), false, true)];
  assert.deepEqual(ids(sesion([p("a"), p("b")], hist, dia(6, 15), semilla(1))), ["b"]);
  assert.deepEqual(ids(sesion([p("a"), p("b")], hist, dia(6, 17), semilla(1))), ["a", "b"]);
});

test("una sesión tiene 12 preguntas, y la caja 1 entra entera antes que cualquier otra pregunta", () => {
  const banco = Array.from({ length: 40 }, (_, k) => p("q" + k));
  const hist = [];
  for (let k = 0; k < 30; k++) hist.push(r("q" + k, iso(dia(1)), true, true));
  for (let k = 30; k < 35; k++) hist.push(r("q" + k, iso(dia(1)), k % 2 === 0, false));
  const s = sesion(banco, hist, dia(6), semilla(3));
  assert.equal(s.length, 12);
  for (let k = 30; k < 35; k++) assert.ok(s.some((q) => q.id === "q" + k));
  for (let k = 20; k < 40; k++) hist.push(r("q" + k, iso(dia(2)), false, true));
  assert.ok(sesion(banco, hist, dia(6), semilla(3)).every((q) => Number(q.id.slice(1)) >= 20));
});

const cuenta = (banco, hist, prefijo) => {
  let n = 0;
  for (let k = 0; k < 300; k++) n += sesion(banco, hist, dia(6), semilla(k)).filter((q) => q.id.startsWith(prefijo)).length;
  return n;
};

test("una pregunta de la caja 3 sale la mitad de veces que una de la caja 2", () => {
  const banco = [...Array.from({ length: 100 }, (_, k) => p("d" + k)), ...Array.from({ length: 100 }, (_, k) => p("t" + k))];
  const hist = [];
  for (let k = 0; k < 100; k++) hist.push(r("d" + k, iso(dia(1)), true, true));
  for (let k = 0; k < 100; k++) hist.push(r("t" + k, iso(dia(1)), true, true), r("t" + k, iso(dia(2)), true, true));
  const razon = cuenta(banco, hist, "t") / cuenta(banco, hist, "d");
  assert.ok(razon > 0.45 && razon < 0.6, String(razon));
});

const hermanas = (c, n) => Array.from({ length: n }, (_, k) => ({ ...p(c + k), concepto: c }));

test("la sesión da una sola pregunta de cada concepto sin ver", () => {
  const banco = Array.from({ length: 20 }, (_, k) => hermanas("c" + k, 3)).flat();
  for (let k = 0; k < 20; k++) {
    const s = sesion(banco, [], dia(6), semilla(k));
    assert.equal(s.length, 12);
    assert.equal(new Set(s.map((q) => q.concepto)).size, 12);
  }
});

test("un concepto sin ver entra después de la caja 1 y antes del sorteo", () => {
  const vistas = Array.from({ length: 30 }, (_, k) => p("v" + k));
  const banco = [...vistas, ...hermanas("a", 2), ...hermanas("b", 2), p("f")];
  const hist = [...vistas.map((q) => r(q.id, iso(dia(1)), true, true)), r("f", iso(dia(1)), false, true)];
  for (let k = 0; k < 20; k++) {
    const s = sesion(banco, hist, dia(6), semilla(k));
    assert.deepEqual(s.filter((q) => !q.id.startsWith("v")).map((q) => q.concepto).sort(), ["a", "b", "f"]);
  }
});

test("el estado de un concepto sale de la última respuesta a una de sus preguntas", () => {
  const banco = [...hermanas("a", 2), ...hermanas("b", 2), ...hermanas("c", 2), ...hermanas("d", 2)];
  const hist = [
    r("a0", iso(dia(1)), true, true),
    r("b0", iso(dia(1)), true, true), r("b1", iso(dia(2)), false, true),
    r("c1", iso(dia(2)), true, false),
  ];
  assert.deepEqual(conceptos(banco, hist), { sabes: 1, noSabes: 2, sinVer: 1 });
});

test("la probabilidad de una nota cuenta los aciertos menos 1/4 de los fallos, sobre 20 preguntas", () => {
  const cerca = (a, b) => assert.ok(Math.abs(a - b) < 1e-9, `${a} ≠ ${b}`);
  cerca(probabilidadDeLaNota(1, 0, 5), 1);
  cerca(probabilidadDeLaNota(0, 0, 5), 0);
  cerca(probabilidadDeLaNota(0.5, 0, 5), 0.5880985260009766);
  cerca(probabilidadDeLaNota(0.625, 0.375, 5), 0.6828711582842681);
  cerca(probabilidadDeLaNota(0.25, 0.75, 5), 0.000935391579332645);
  cerca(probabilidadDeLaNota(0.5, 0, 7.5), 0.020694732666015625);
  cerca(probabilidadDeLaNota(0.625, 0.375, 7.5), 0.07904140886862707);
  cerca(probabilidadDeLaNota(0.925, 0.075, 10), 0.21029776386659405);
});

test("la sesión mezcla los temas", () => {
  const banco = [...Array.from({ length: 50 }, (_, k) => p("r" + k, "rgpd")), ...Array.from({ length: 50 }, (_, k) => p("e" + k, "ens"))];
  const temas = new Set(sesion(banco, [], dia(6), semilla(7)).map((q) => q.tema));
  assert.deepEqual([...temas].sort(), ["ens", "rgpd"]);
});

test("una sesión sin preguntas pendientes está vacía", () => {
  const hist = [r("a", iso(dia(6, 10)), true, true), r("b", iso(dia(6, 11)), true, true)];
  assert.deepEqual(sesion([p("a"), p("b")], hist, dia(6, 12), semilla(1)), []);
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

test("una pregunta en blanco lleva blanco, con acierto y seguro en false", () => {
  assert.deepEqual(respuesta(p("a"), null, true, dia(6)), { id: "a", ts: iso(dia(6)), acierto: false, seguro: false, blanco: true });
});

test("una prueba es una respuesta a una pregunta distinta de la anterior del concepto, en el estado de antes", () => {
  const banco = [...hermanas("a", 2), ...hermanas("b", 2)];
  const hist = [
    r("a0", iso(dia(1, 10)), true, true), // sin ver: acierto con seguro
    r("a0", iso(dia(1, 11)), false, true), // la misma pregunta: no es una prueba, y el concepto pasa a "no lo sabes"
    r("a1", iso(dia(1, 12)), true, false), // no lo sabes: acierto con dudo
    { ...r("b0", iso(dia(1, 13)), false, false), blanco: true }, // sin ver: blanco
    r("a0", iso(dia(1, 14)), true, true), // no lo sabes: acierto con seguro
    r("a1", iso(dia(1, 15)), false, true), // lo sabes: fallo con seguro
    r("huerfana", iso(dia(1, 16)), true, true),
  ];
  assert.deepEqual(pruebas(banco, hist), { sabes: [0, 1, 0, 0, 0], noSabes: [1, 0, 1, 0, 0], sinVer: [1, 0, 0, 0, 1] });
});

test("la caja es del concepto: un fallo en una pregunta hermana devuelve el concepto a la caja 1", () => {
  const banco = hermanas("a", 2);
  const sube = [r("a0", iso(dia(1)), true, true), r("a1", iso(dia(2)), true, true)];
  assert.equal(cajas(banco, sube).get("a").caja, 3);
  assert.equal(cajas(banco, [...sube, r("a0", iso(dia(3)), false, true)]).get("a").caja, 1);
  assert.equal(cajas(banco, [r("otra", iso(dia(1)), true, true)]).size, 0);
});

test("un concepto sale con la pregunta que lleva más tiempo sin salir", () => {
  const banco = hermanas("a", 3);
  const hist = [r("a1", iso(dia(1, 10)), false, true), r("a0", iso(dia(1, 11)), false, true)];
  for (let k = 0; k < 10; k++) assert.deepEqual(ids(sesion(banco, hist, dia(6), semilla(k))), ["a2"]);
  const todas = [...hist, r("a2", iso(dia(1, 12)), false, true)];
  assert.deepEqual(ids(sesion(banco, todas, dia(6), semilla(1))), ["a1"]);
});

test("unir 2 historiales quita las respuestas repetidas", () => {
  const a = r("a", "2026-11-06T18:02:11Z", true, true);
  const b = r("b", "2026-11-06T18:03:00Z", false, false);
  assert.deepEqual(unir([a, b], [{ ...a }]), [a, b]);
});

test("el simulacro saca 20 preguntas distintas al azar de todo el banco, sin mirar las cajas", () => {
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

test("el banco de ejemplo tiene 6 preguntas, y una sesión nueva las saca todas", () => {
  assert.equal(ejemplo.length, 6);
  assert.deepEqual(ids(sesion(ejemplo, [], dia(6), semilla(1))), ids(ejemplo));
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

// Un banco de n conceptos con una pregunta cada uno, y un historial que da a cada respuesta una hora creciente.
const bancoDe = (n) => Array.from({ length: n }, (_, i) => ({ id: `q${i}`, concepto: `c${i}` }));
const historialDe = (...respuestas) => respuestas.map(([i, acierto, seguro], k) => r(`q${i}`, new Date(k * 60000).toISOString(), acierto, seguro));
const cerca = (a, b) => assert.ok(Math.abs(a - b) < 1e-9, `${a} ≠ ${b}`);

test("la tarjeta sin historial usa el prior: 1 de 4 a ciegas, y no tiene ritmo", () => {
  const t = tarjetaDeLaNota(bancoDe(20), [], 5);
  assert.equal(t.sinVer, 20);
  cerca(t.filas.sinVer.aciertos, 1 / 4);
  cerca(t.filas.sabes.aciertos, 1);
  assert.equal(t.filas.sinVer.pruebas, 0);
  assert.equal(t.blanco, 0);
  // El prior pesa 2 pruebas: la incertidumbre sube la cifra por encima de probabilidadDeLaNota(0.25, 0.75, 5), un 0,1 %.
  assert.ok(t.hoy > 0.01 && t.hoy < 0.2, String(t.hoy));
  assert.equal(t.contestar, true);
  assert.equal(t.sesiones, null);
});

test("las tasas de la tarjeta salen de las pruebas, y la cifra no cambia entre 2 cálculos", () => {
  // 40 conceptos sin ver: 30 aciertos con seguro y 10 fallos con dudo.
  const h = historialDe(...Array.from({ length: 40 }, (_, i) => [i, i < 30, i < 30]));
  const t = tarjetaDeLaNota(bancoDe(60), h, 5);
  cerca(t.filas.sinVer.aciertos, 30.5 / 42);
  assert.deepEqual([t.filas.sinVer.vistos, t.filas.sinVer.pruebas], [30, 40]);
  assert.equal(tarjetaDeLaNota(bancoDe(60), h, 5).hoy, t.hoy);
});

test("la tarjeta sigue la forma con la probabilidad más alta, y el consejo da la diferencia en puntos", () => {
  // 10 conceptos que sabes, y 10 conceptos con 2 fallos con dudo, uno en cada pregunta: una duda casi nunca acierta.
  const banco = [...bancoDe(10), ...Array.from({ length: 10 }, (_, i) => hermanas("d" + i, 2)).flat()];
  const h = [...historialDe(...Array.from({ length: 10 }, (_, i) => [i, true, true])),
    ...Array.from({ length: 10 }, (_, i) => [r(`d${i}0`, iso(dia(2, 10)), false, false), r(`d${i}1`, iso(dia(2, 11)), false, false)]).flat()];
  const t = tarjetaDeLaNota(banco, h, 5);
  cerca(t.hoy, 0.5880985260009766);
  assert.equal(t.contestar, false);
  assert.equal(t.puntos, Math.round(100 * (t.blanco - t.contesta)));
  assert.ok(t.puntos > 0);
});

test("el camino cuenta los conceptos que faltan para un 95 %, y las sesiones salen del ritmo, redondeadas hacia arriba", () => {
  // Para un 10 con 20 preguntas, solo 20 aciertos valen, así que el 95 % pide todos los conceptos.
  const h = historialDe(...[0, 1, 2, 3].map((i) => [i, true, true]));
  const t = tarjetaDeLaNota(bancoDe(20), h, 10);
  assert.equal(t.sabes, 4);
  assert.equal(t.meta, 20);
  assert.equal(t.faltan, 16);
  cerca(t.ritmo, 12);
  assert.equal(t.sesiones, 2);
});

test("el ritmo cuenta solo las 60 últimas respuestas, 5 sesiones de 12", () => {
  const h = historialDe(
    ...Array.from({ length: 40 }, (_, i) => [i, true, true]),
    ...Array.from({ length: 10 }, (_, i) => [40 + i, true, true]),
    ...Array.from({ length: 50 }, () => [0, true, true]));
  const t = tarjetaDeLaNota(bancoDe(100), h, 10);
  assert.equal(t.sabes, 50);
  cerca(t.ritmo, 2);
  assert.equal(t.sesiones, Math.ceil(t.faltan / 2));
});

test("sin un 95 % ni con todos los conceptos, el camino no alcanza y no tiene sesiones", () => {
  // Un fallo con seguro en una pregunta hermana de cada concepto que sabes: "lo sabes" acierta poco.
  const banco = Array.from({ length: 20 }, (_, i) => hermanas("c" + i, 2)).flat();
  const h = Array.from({ length: 20 }, (_, i) => [r(`c${i}0`, iso(dia(1, 10)), true, true), r(`c${i}1`, iso(dia(1, 11)), false, true), r(`c${i}0`, iso(dia(1, 12)), true, true)]).flat();
  const t = tarjetaDeLaNota(banco, h.slice(0, -3), 5);
  assert.equal(t.alcanza, false);
  assert.equal(t.meta, 20);
  assert.equal(t.sesiones, null);
});

test("con un 95 % o más hoy no faltan conceptos ni sesiones", () => {
  const t = tarjetaDeLaNota(bancoDe(20), historialDe(...Array.from({ length: 20 }, (_, i) => [i, true, true])), 5);
  assert.equal(t.faltan, 0);
  assert.equal(t.meta, 20);
  assert.equal(t.sesiones, 0);
});

test("con un ritmo de 0 o menos, las sesiones no se pueden calcular", () => {
  const t = tarjetaDeLaNota(bancoDe(20), historialDe([0, false, true]), 10);
  assert.equal(t.faltan, 20);
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
