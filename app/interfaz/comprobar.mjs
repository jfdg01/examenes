// La comprobación de cada pantalla en el ancho del móvil y en el del PC, contra el servidor y una copia de LN.
// `npm run build` va antes. Las capturas quedan en `capturas/`.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { cpSync, mkdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { chromium } from "playwright";

const raiz = mkdtempSync(join(tmpdir(), "examenes-"));
// Cada pantalla tiene su copia, porque el servidor guarda la ronda a medias de cada examen.
for (const examen of ["movil", "pc"]) {
  mkdirSync(join(raiz, examen));
  for (const f of ["examen.json", "preguntas.json"]) cpSync(join("../../ln", f), join(raiz, examen, f));
}
spawnSync("cargo", ["build", "-q", "-p", "servidor"], { stdio: "inherit" });
const servidor = spawn("../target/debug/servidor", [raiz, "127.0.0.1:8691"], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 500));

const navegador = await chromium.launch();
// La copia de LN no tiene `paginas/`: cada página es una hoja en blanco de 600 × 800.
const hoja = await (await navegador.newPage({ viewport: { width: 600, height: 800 } })).screenshot();
mkdirSync("capturas", { recursive: true });

async function abrir(examen, width, height) {
  const page = await navegador.newPage({ viewport: { width, height } });
  await page.route("**/paginas/**", (r) => r.fulfill({ contentType: "image/png", body: hoja }));
  await page.goto(`http://127.0.0.1:8691/${examen}/`);
  await page.getByText("Empezar la sesión").click();
  return page;
}
const visible = (page, sel) => page.locator(sel).first().isVisible();
const contestar = (page) => page.locator(".opcion button").first().click();

try {
  // El móvil: la página de la fuente se abre a pantalla completa, y las otras opciones se pliegan.
  const movil = await abrir("movil", 390, 844);
  await movil.screenshot({ path: "capturas/movil-pregunta.png" });
  assert(!(await visible(movil, ".tirador")), "el móvil no tiene tiradores");
  await contestar(movil);
  assert(await visible(movil, ".hoja p"), "el móvil muestra la explicación");
  assert(!(await visible(movil, ".hoja img")), "el móvil no muestra la página en la hoja");
  assert(await visible(movil, ".chip"), "el móvil pliega las otras opciones");
  const marcadas = await movil.locator(".opcion", { hasText: /[✓✗]/ }).count();
  assert.equal(await movil.locator(".opcion:visible").count(), marcadas, "solo quedan la correcta y la elegida");
  await movil.screenshot({ path: "capturas/movil-solucion.png" });
  await movil.locator(".ver-pagina").click();
  assert(await visible(movil, ".pagina img"), "la página se abre a pantalla completa");
  await movil.screenshot({ path: "capturas/movil-pagina.png" });
  await movil.goBack();
  assert.equal(await movil.locator(".pagina").count(), 0, "el botón de atrás cierra la página");

  // El PC: 2 columnas, la página al lado de la pregunta y un tirador por borde.
  const pc = await abrir("pc", 1280, 800);
  assert(await visible(pc, ".vacia"), "el PC muestra la hoja vacía antes de contestar");
  assert.equal(await pc.locator(".tirador:visible").count(), 3);
  assert.equal(await pc.locator("[title*=tamaño]").count(), 0, "el PC no tiene el botón del engranaje");
  await pc.screenshot({ path: "capturas/pc-pregunta.png" });
  await contestar(pc);
  // El PC muestra la página al lado, cuando la imagen carga.
  await pc.locator(".hoja img").waitFor();
  assert(!(await visible(pc, ".ver-pagina")), "el PC no tiene el botón de la página");
  assert(!(await visible(pc, ".chip")), "el PC no pliega las opciones");

  const asa = () => pc.locator(".tirador").nth(1).evaluate((t) => getComputedStyle(t, "::before").display);
  assert.equal(await asa(), "none", "el asa no se ve sin el ratón encima");
  const caja = await pc.locator(".tirador").nth(1).boundingBox();
  await pc.mouse.move(caja.x + 12, 400);
  assert.equal(await asa(), "block", "el asa sale al pasar el ratón");
  const antes = (await pc.locator(".hoja").boundingBox()).x;
  await pc.mouse.down();
  await pc.mouse.move(0.35 * 1280, 400, { steps: 5 });
  await pc.mouse.up();
  assert((await pc.locator(".hoja").boundingBox()).x < antes - 100, "el separador mueve la hoja");
  const guardadas = JSON.parse(await pc.evaluate(() => localStorage.getItem("pc-columnas")));
  assert(Math.abs(guardadas.s - 0.35) < 0.01, "las columnas quedan en localStorage");
  await pc.screenshot({ path: "capturas/pc-solucion.png" });

  // Una ventana estrecha del PC usa el diseño del móvil, sin recargar.
  await pc.setViewportSize({ width: 800, height: 800 });
  assert(await visible(pc, ".ver-pagina"), "una ventana estrecha usa el diseño del móvil");
  assert(!(await visible(pc, ".tirador")), "una ventana estrecha no tiene tiradores");
  await pc.screenshot({ path: "capturas/pc-estrecho.png" });
  console.log("Todas las pantallas pasan.");
} finally {
  await navegador.close();
  servidor.kill();
}
