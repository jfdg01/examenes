import { readFileSync } from "node:fs";
import { sesion, cajas, semilla } from "../../../logica.js";
const D = new URL("../../../", import.meta.url).pathname;
const banco = JSON.parse(readFileSync(D + "preguntas.json", "utf8"));
const hist = readFileSync(D + "respuestas.jsonl", "utf8").split("\n").filter(Boolean).map(JSON.parse);
const conceptoDe = new Map(banco.map(q => [q.id, q.concepto]));
const conceptos = [...new Set(banco.map(q => q.concepto))];
// 0 = lo sé y acierto, 1 = creo que lo sé y fallo, 2 = no lo sé (dudo o blanco)
const salida = r => r.blanco || !r.seguro ? 2 : r.acierto ? 0 : 1;
const porConcepto = new Map(conceptos.map(c => [c, []]));
const ultimaId = new Map(), ultimaSalida = new Map();
for (const r of [...hist].sort((a, b) => a.ts.localeCompare(b.ts))) {
  const c = conceptoDe.get(r.id); if (!c) continue;
  ultimaSalida.set(c, salida(r));
  if (ultimaId.get(c) !== r.id) porConcepto.get(c).push(salida(r));
  ultimaId.set(c, r.id);
}
const seqs = [...porConcepto.values()].filter(s => s.length);
// EM (Baum-Welch con 2 estados), con pseudo-counts como prior.
let L0 = 0.5, T = 0.1, e1 = [0.9, 0.05, 0.05], e0 = [0.3, 0.2, 0.5];
const P1 = [9.5, 0.2, 0.3], P0 = [1.5, 1, 2.5]; // prior: 10 y 5 pruebas
for (let it = 0; it < 200; it++) {
  let nL0 = 1, dL0 = 2, nT = 1, dT = 10, c1 = [...P1], c0 = [...P0];
  for (const s of seqs) {
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
    const g = a.map((x, t) => { const y = [x[0] * b[t][0], x[1] * b[t][1]], z = y[0] + y[1]; return [y[0] / z, y[1] / z]; });
    nL0 += g[0][1]; dL0 += 1;
    for (let t = 0; t < n; t++) { c1[s[t]] += g[t][1]; c0[s[t]] += g[t][0]; }
    for (let t = 1; t < n; t++) {
      const o = s[t], w00 = a[t - 1][0] * (1 - T) * e0[o] * b[t][0], w01 = a[t - 1][0] * T * e1[o] * b[t][1];
      const w11 = a[t - 1][1] * e1[o] * b[t][1], z = w00 + w01 + w11;
      nT += w01 / z; dT += (w00 + w01) / z;
    }
  }
  L0 = nL0 / dL0; T = nT / dT;
  const s1 = c1.reduce((x, y) => x + y), s0 = c0.reduce((x, y) => x + y);
  e1 = c1.map(x => x / s1); e0 = c0.map(x => x / s0);
}
const f = x => x.toFixed(3);
console.log("L0", f(L0), "T", f(T), "e1 (sé, fallo seguro, no sé)", e1.map(f).join(" "), "e0", e0.map(f).join(" "));
// P(L) de hoy de cada concepto
const PL = new Map(conceptos.map(c => {
  let p = L0;
  for (const o of porConcepto.get(c)) { const q = p * e1[o] / (p * e1[o] + (1 - p) * e0[o]); p = q + (1 - q) * T; }
  return [c, p];
}));
const grupo = c => !ultimaSalida.has(c) ? "sin ver" : ultimaSalida.get(c) !== 0 ? "acabas de fallar" : PL.get(c) >= 0.9 ? "lo sabes (≥ 90 %)" : "acertado, < 90 %";
const nombres = ["acabas de fallar", "sin ver", "acertado, < 90 %", "lo sabes (≥ 90 %)"];
const cuenta = g => conceptos.filter(c => grupo(c) === g);
for (const g of nombres) { const cs = cuenta(g); const m = cs.reduce((x, c) => x + PL.get(c), 0) / (cs.length || 1); console.log(g.padEnd(20), String(cs.length).padStart(3), "conceptos, P(L) media", f(m)); }
// El reparto de una sesión de 12: sorteo sin reemplazo con peso max(1 − P(L), minimo)
for (const minimo of [0.1, 0.05]) {
  const tot = Object.fromEntries(nombres.map(g => [g, 0])), N = 20000;
  let rng = semilla(7);
  for (let k = 0; k < N; k++) {
    const pool = conceptos.map(c => [c, Math.max(1 - PL.get(c), minimo)]);
    for (let i = 0; i < 12; i++) {
      const S = pool.reduce((x, y) => x + y[1], 0); let r = rng() * S, j = 0;
      while ((r -= pool[j][1]) > 0) j++;
      tot[grupo(pool[j][0])]++; pool.splice(j, 1);
    }
  }
  console.log(`mínimo ${minimo}:`, nombres.map(g => `${g} ${(tot[g] / N).toFixed(1)} (${(100 * tot[g] / N / 12).toFixed(0)} %)`).join(" · "));
}
// La sesión de hoy, para comparar
const tot = Object.fromEntries(nombres.map(g => [g, 0]));
for (let k = 0; k < 2000; k++) for (const q of sesion(banco, hist, new Date(), semilla(k))) tot[grupo(q.concepto)]++;
console.log("la sesión de hoy:", nombres.map(g => `${g} ${(tot[g] / 2000).toFixed(1)}`).join(" · "));
