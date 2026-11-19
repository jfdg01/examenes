import { readFileSync } from "node:fs";
import { semilla, probabilidadDeLaNota } from "../../../logica.js";
const D = new URL("../../../", import.meta.url).pathname;
const banco = JSON.parse(readFileSync(D + "preguntas.json", "utf8"));
const hist = readFileSync(D + "respuestas.jsonl", "utf8").split("\n").filter(Boolean).map(JSON.parse);
const conceptoDe = new Map(banco.map(q => [q.id, q.concepto]));
const conceptos = [...new Set(banco.map(q => q.concepto))];
// Los parámetros del ajuste de bkt.mjs.
const L0 = 0.178, T = 0.293, e1 = [0.942, 0.012, 0.046], e0 = [0.281, 0.132, 0.586], Q = 0.5; // Q: acierto de una duda contestada
const salida = r => r.blanco || !r.seguro ? 2 : r.acierto ? 0 : 1;
const act = (p, o) => { const q = p * e1[o] / (p * e1[o] + (1 - p) * e0[o]); return q + (1 - q) * T; };
// El estado de hoy: P(L), la última salida y la caja aproximada de Leitner.
const ini = new Map(conceptos.map(c => [c, { p: L0, ult: null, caja: 1, visto: false }]));
const ultimaId = new Map();
for (const r of [...hist].sort((a, b) => a.ts.localeCompare(b.ts))) {
  const c = conceptoDe.get(r.id); if (!c) continue; const e = ini.get(c), o = salida(r);
  if (ultimaId.get(c) !== r.id) e.p = act(e.p, o);
  ultimaId.set(c, r.id); e.ult = o; e.visto = true; e.caja = o === 0 ? Math.min(3, e.caja + 1) : 1;
}
const nota = (sabe, obj) => {
  let a = 0, f = 0;
  for (const s of sabe) { const e = s ? e1 : e0; a += e[0] + e[2] * Q; f += e[1] + e[2] * (1 - Q); }
  return probabilidadDeLaNota(a / sabe.length, f / sabe.length, obj);
};
const sortea = (rng, cands, peso, n) => {
  const pool = cands.map(c => [c, peso(c)]).filter(x => x[1] > 0), out = [];
  while (out.length < n && pool.length) {
    const S = pool.reduce((x, y) => x + y[1], 0); let r = rng() * S, j = 0;
    while (j < pool.length - 1 && (r -= pool[j][1]) > 0) j++;
    out.push(pool[j][0]); pool.splice(j, 1);
  }
  return out;
};
const fallado = e => e.visto && e.ult !== 0;
const politica = (nombre, elige) => ({ nombre, elige });
const pesoP = (k, m) => e => Math.max((1 - e.p) ** k, m);
const conPrimero = (filtro, resto) => (rng, est) => {
  const prim = conceptos.filter(c => filtro(est.get(c))).sort(() => rng() - 0.5).slice(0, 12);
  return [...prim, ...sortea(rng, conceptos.filter(c => !prim.includes(c)), c => resto(est.get(c)), 12 - prim.length)];
};
const POL = [
  politica("hoy: caja 1, sin ver, sorteo por caja", (rng, est) => {
    const c1 = conceptos.filter(c => { const e = est.get(c); return e.visto && e.caja === 1; }).sort(() => rng() - 0.5).slice(0, 12);
    const sv = conceptos.filter(c => !est.get(c).visto).sort(() => rng() - 0.5).slice(0, 12 - c1.length);
    const ya = [...c1, ...sv];
    return [...ya, ...sortea(rng, conceptos.filter(c => !ya.includes(c) && est.get(c).visto), c => ({ 1: 1, 2: 1, 3: 0.5 })[est.get(c).caja], 12 - ya.length)];
  }),
  politica("P2: fallados primero, peso 1−P, mín 0,1", conPrimero(fallado, pesoP(1, 0.1))),
  politica("sorteo puro, peso 1−P, mín 0,1", conPrimero(() => false, pesoP(1, 0.1))),
  politica("fallados primero, peso (1−P)², mín 0,05", conPrimero(fallado, pesoP(2, 0.05))),
  politica("sorteo puro, peso (1−P)⁴, mín 0,02", conPrimero(() => false, pesoP(4, 0.02))),
  politica("los 12 con la P más baja", (rng, est) => [...conceptos].sort((a, b) => est.get(a).p - est.get(b).p || rng() - 0.5).slice(0, 12)),
  politica("al azar, sin pesos", conPrimero(() => false, () => 1)),
];
const R = 400;
for (const H of [4, 8, 12]) {
  console.log(`\n${H} sesiones (${12 * H} preguntas) antes del examen`);
  for (const pol of POL) {
    let s5 = 0, s7 = 0, sab = 0, s0 = 0;
    for (let k = 0; k < R; k++) {
      const rng = semilla(1000 + k), mundo = semilla(5000 + k);
      const est = new Map([...ini].map(([c, e]) => [c, { ...e }]));
      const sabe = new Map(conceptos.map(c => [c, mundo() < ini.get(c).p]));
      if (pol === POL[0]) s0 += nota([...sabe.values()], 5);
      for (let h = 0; h < H; h++) for (const c of pol.elige(rng, est)) {
        const e = est.get(c), em = sabe.get(c) ? e1 : e0, u = mundo();
        const o = u < em[0] ? 0 : u < em[0] + em[1] ? 1 : 2;
        if (!sabe.get(c) && mundo() < T) sabe.set(c, true);
        e.p = act(e.p, o); e.ult = o; e.visto = true; e.caja = o === 0 ? Math.min(3, e.caja + 1) : 1;
      }
      const v = [...sabe.values()];
      s5 += nota(v, 5); s7 += nota(v, 7); sab += v.filter(Boolean).length;
    }
    if (pol === POL[0] && H === 4) console.log(`  hoy, sin más sesiones: P(≥5) ${(100 * s0 / R).toFixed(1)} %`);
    console.log(`  ${pol.nombre.padEnd(42)} P(≥5) ${(100 * s5 / R).toFixed(1)} %  P(≥7) ${(100 * s7 / R).toFixed(1)} %  sabes ${(sab / R).toFixed(1)}`);
  }
}
