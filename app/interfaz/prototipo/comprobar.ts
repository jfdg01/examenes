// PROTOTIPO de navegar-preguntas y editar-pregunta: recorre una sesión y un simulacro contra el Vite del prototipo.
// Las capturas quedan en prototipo/capturas/.
import assert from "node:assert/strict";
import { mkdirSync } from "node:fs";
import { chromium, type Page } from "playwright";

const url = "http://127.0.0.1:8604/ps/";
mkdirSync("prototipo/capturas", { recursive: true });
const navegador = await chromium.launch();
const foto = (p: Page, n: string) => p.screenshot({ path: `prototipo/capturas/${n}.png` });
const contador = (p: Page) => p.locator("span.truncate").first().innerText();

for (const [v, ancho, alto] of [["a", 390, 800], ["b", 390, 800], ["a", 1400, 900]] as const) {
  const n = `${v}-${ancho}`;
  const p = await navegador.newPage({ viewport: { width: ancho, height: alto } });
  await p.goto(`${url}?v=${v}`);

  // La sesión: contesta la 1, salta a la 3 por la cuadrícula, y "Terminar" vuelve a la 2.
  await p.getByText("Empezar la sesión").click();
  await p.locator(".opcion button").first().click();
  assert.equal(await p.locator(".icono").count() > 0, true, "la sesión corrige al momento");
  if (v === "a") await p.getByTitle("Las preguntas").click();
  else await p.getByRole("button", { name: "1 / 12" }).click();
  await foto(p, `${n}-sesion-cuadricula`);
  await p.locator("div.grid button", { hasText: /^3$/ }).click();
  assert.match(await contador(p), /^3 \/ 12/);
  if (v === "a") await p.getByTitle("Las preguntas").click();
  else await p.getByRole("button", { name: "3 / 12" }).click();
  await p.getByText("Terminar la sesión").click();
  assert.match(await contador(p), /^2 \/ 12/);
  await p.getByText(/faltan 11 preguntas/).waitFor();

  // La edición: cambia el enunciado de la 2 y guarda.
  await p.getByTitle("Editar la pregunta").click();
  await foto(p, `${n}-editar`);
  await p.locator("textarea").first().fill("Un enunciado editado");
  await p.getByText("Guardar").click();
  await p.getByText("Un enunciado editado").waitFor();
  // Cancelar deja el texto como estaba.
  await p.getByTitle("Editar la pregunta").click();
  await p.locator("textarea").first().fill("Otro");
  await p.getByText("Cancelar").click();
  await p.getByText("Un enunciado editado").waitFor();
  await foto(p, `${n}-sesion`);
  await p.getByTitle("Abandonar").click({ trial: true });
  p.on("dialog", (d) => d.accept());
  await p.getByTitle("Abandonar").click();

  // El simulacro: contesta todas sin ver la corrección, y el final las corrige.
  await p.getByText(/^Simulacro/).click();
  for (let k = 0; k < 20; k++) {
    await p.locator(".opcion button").first().click();
    assert.equal(await p.locator(".icono").count(), 0, "el simulacro no corrige al momento");
    if (k === 0) await foto(p, `${n}-simulacro-contestada`);
    await p.getByRole("button", { name: k === 19 ? "Terminar" : "Siguiente", exact: true }).last().click();
  }
  await p.getByText("El simulacro ha terminado").waitFor();
  await foto(p, `${n}-simulacro-final`);
  await p.locator("main div.grid button", { hasText: /^5$/ }).click();
  assert.match(await contador(p), /^Revisión · 5 \/ 20/);
  assert.equal(await p.locator(".icono").count() > 0, true, "la revisión corrige");
  await foto(p, `${n}-simulacro-revision`);
  await p.getByTitle("Volver al resultado").click();
  await p.getByText("El simulacro ha terminado").waitFor();
  await p.close();
}
await navegador.close();
console.log("ok");
