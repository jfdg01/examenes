import { test } from "node:test";
import { readFileSync } from "node:fs";
import assert from "node:assert/strict";
import { cajas, sesion, proximaFecha, opciones, respuesta, porcentajeDudo, dominio, unir, simulacro, nota, imagenDePagina, pdfEnPagina, resultado } from "./logica.js";

const r = (id, ts, acierto, seguro) => ({ id, ts, acierto, seguro });

test("un acierto con seguro sube una caja, hasta la 3", () => {
  const h = [
    r("a", "2026-11-01T10:00:00Z", true, true),
    r("a", "2026-11-03T10:00:00Z", true, true),
    r("a", "2026-11-07T10:00:00Z", true, true),
  ];
  assert.equal(cajas(h).get("a").caja, 3);
  assert.equal(cajas(h.slice(0, 1)).get("a").caja, 2);
});

test("un fallo o un acierto con dudo vuelve a la caja 1", () => {
  const sube = [r("a", "2026-11-01T10:00:00Z", true, true), r("a", "2026-11-03T10:00:00Z", true, true)];
  assert.equal(cajas([...sube, r("a", "2026-11-07T10:00:00Z", false, true)]).get("a").caja, 1);
  assert.equal(cajas([...sube, r("a", "2026-11-07T10:00:00Z", true, false)]).get("a").caja, 1);
  assert.equal(cajas([...sube, r("a", "2026-11-07T10:00:00Z", false, false)]).get("a").caja, 1);
});

test("el historial se ordena por fecha, venga de donde venga", () => {
  const h = [r("a", "2026-11-03T10:00:00Z", false, true), r("a", "2026-11-01T10:00:00Z", true, true)];
  assert.equal(cajas(h).get("a").caja, 1);
  assert.equal(cajas(h).get("a").ts, "2026-11-03T10:00:00Z");
});

function semilla(n) {
  return () => {
    n = (n + 0x6d2b79f5) | 0;
    let t = Math.imul(n ^ (n >>> 15), 1 | n);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
const p = (id, tema = "rgpd") => ({ id, tema, concepto: id, correcta: "c", distractores: ["x", "y", "z"] });
const iso = (d) => d.toISOString().slice(0, 19) + "Z";
const dia = (d, h = 12) => new Date(2026, 10, d, h);
const ids = (s) => s.map((q) => q.id).sort();

test("una pregunta nueva está en la caja 1 y sale en la sesión", () => {
  assert.deepEqual(ids(sesion([p("a")], [], dia(6), semilla(1))), ["a"]);
});

test("la caja 2 sale cada 2 días y la caja 3 cada 4 días, por días de calendario", () => {
  const caja2 = [r("a", iso(dia(1, 23)), true, true)];
  assert.deepEqual(ids(sesion([p("a")], caja2, dia(2, 20), semilla(1))), []);
  assert.deepEqual(ids(sesion([p("a")], caja2, dia(3, 8), semilla(1))), ["a"]);
  const caja3 = [r("a", iso(dia(1)), true, true), r("a", iso(dia(3)), true, true)];
  assert.deepEqual(ids(sesion([p("a")], caja3, dia(6, 23), semilla(1))), []);
  assert.deepEqual(ids(sesion([p("a")], caja3, dia(7, 1), semilla(1))), ["a"]);
});

test("una sesión tiene 25 preguntas, y las de la caja 1 van primero", () => {
  const banco = Array.from({ length: 40 }, (_, k) => p("q" + k));
  const hist = [];
  for (let k = 0; k < 20; k++) hist.push(r("q" + k, iso(dia(1)), true, true));
  for (let k = 20; k < 25; k++) hist.push(r("q" + k, iso(dia(1)), false, true));
  const s = sesion(banco, hist, dia(6), semilla(3));
  assert.equal(s.length, 25);
  const enCaja1 = (q) => !hist.some((h) => h.id === q.id && h.acierto);
  assert.ok(s.slice(0, 20).every(enCaja1));
  assert.ok(s.slice(20).every((q) => !enCaja1(q)));
});

test("en la caja 1, una pregunta fallada va antes que una pregunta nueva", () => {
  const banco = Array.from({ length: 40 }, (_, k) => p("q" + k));
  const hist = [r("q39", iso(dia(5)), false, true), r("q38", iso(dia(5)), true, false)];
  const s = sesion(banco, hist, dia(6), semilla(5));
  assert.deepEqual(ids(s.slice(0, 2)), ["q38", "q39"]);
});

test("la sesión mezcla los temas", () => {
  const banco = [...Array.from({ length: 50 }, (_, k) => p("r" + k, "rgpd")), ...Array.from({ length: 50 }, (_, k) => p("e" + k, "ens"))];
  const temas = new Set(sesion(banco, [], dia(6), semilla(7)).map((q) => q.tema));
  assert.deepEqual([...temas].sort(), ["ens", "rgpd"]);
});

test("sin preguntas pendientes, la próxima fecha es la de la pregunta que antes vuelve", () => {
  const hist = [r("a", iso(dia(5)), true, true), r("b", iso(dia(1)), true, true), r("b", iso(dia(3)), true, true)];
  assert.deepEqual(sesion([p("a"), p("b")], hist, dia(6), semilla(1)), []);
  assert.deepEqual(proximaFecha([p("a"), p("b")], hist), new Date(2026, 10, 7));
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
  assert.deepEqual(porcentajeDudo(h), { porcentaje: 200 / 3, total: 3, contestar: true });
  assert.equal(porcentajeDudo([r("d", "4", true, true)]), null);
});

test("el dominio de un concepto es la caja más baja de sus preguntas", () => {
  const banco = [{ ...p("a1"), concepto: "A" }, { ...p("a2"), concepto: "A" }, { ...p("b1"), concepto: "B" }];
  const h = [r("a1", iso(dia(1)), true, true), r("a1", iso(dia(3)), true, true), r("a2", iso(dia(1)), true, true), r("b1", iso(dia(1)), true, true)];
  const d = dominio(banco, h);
  assert.equal(d.get("A"), 2);
  assert.equal(d.get("B"), 2);
  assert.equal(dominio(banco, []).get("A"), 1);
});

test("unir 2 historiales quita las respuestas repetidas", () => {
  const a = r("a", "2026-11-06T18:02:11Z", true, true);
  const b = r("b", "2026-11-06T18:03:00Z", false, false);
  assert.deepEqual(unir([a, b], [{ ...a }]), [a, b]);
});

test("el simulacro saca 40 preguntas distintas al azar de todo el banco, sin mirar las cajas", () => {
  const banco = Array.from({ length: 100 }, (_, k) => p("q" + k));
  const s = simulacro(banco, semilla(2));
  assert.equal(s.length, 40);
  assert.equal(new Set(s.map((q) => q.id)).size, 40);
  assert.notDeepEqual(ids(s), ids(simulacro(banco, semilla(9))));
  assert.equal(simulacro(banco.slice(0, 6), semilla(2)).length, 6);
});

test("la nota del simulacro es sobre 10, con la penalización de cada fallo", () => {
  const res = [...Array(28).fill("acierto"), ...Array(6).fill("fallo"), ...Array(6).fill("blanco")];
  assert.equal(nota(res, 1 / 3), 6.5);
  assert.equal(nota(res, 1 / 2), 6.25);
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

test("con un porcentaje de dudo sobre el 33 %, una pregunta dudosa se contesta", () => {
  const h = (aciertos, fallos) => [...Array(aciertos).fill(r("a", "1", true, false)), ...Array(fallos).fill(r("b", "2", false, false))];
  assert.equal(porcentajeDudo(h(34, 66)).contestar, true);
  assert.equal(porcentajeDudo(h(33, 67)).contestar, false);
});

test("una respuesta del simulacro es un acierto, un fallo o una pregunta en blanco", () => {
  assert.equal(resultado(p("a"), "c"), "acierto");
  assert.equal(resultado(p("a"), "x"), "fallo");
  assert.equal(resultado(p("a"), null), "blanco");
});
