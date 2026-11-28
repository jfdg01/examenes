// La comprobación de cada pantalla en el ancho del móvil y en el del PC, contra el servidor y una copia del examen de `prueba/`.
// `npm run build` va antes. Las capturas quedan en `capturas/`.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { chromium, type Page } from "playwright";

const raiz = mkdtempSync(join(tmpdir(), "examenes-"));
// Cada pantalla tiene su copia, porque el servidor guarda la ronda a medias de cada examen.
for (const examen of ["movil", "pc", "mixto-movil", "mixto-pc"]) {
  mkdirSync(join(raiz, examen));
  for (const f of ["examen.json", "preguntas.json", "conceptos.json", "desarrollo.json"]) cpSync(join("prueba", f), join(raiz, examen, f));
}
// El simulacro mixto lleva las 2 preguntas de desarrollo del banco de prueba.
for (const examen of ["mixto-movil", "mixto-pc"]) {
  const e = JSON.parse(readFileSync(join("prueba", "examen.json"), "utf8"));
  writeFileSync(join(raiz, examen, "examen.json"), JSON.stringify({ ...e, desarrollo: 2, pesos: { test: 5, desarrollo: 5 } }));
}
spawnSync("cargo", ["build", "-q", "-p", "servidor"], { stdio: "inherit" });
const servidor = spawn("../target/debug/servidor", [raiz, "127.0.0.1:8691"], { stdio: "ignore" });
await new Promise((r) => setTimeout(r, 500));

const navegador = await chromium.launch();
// La copia no tiene `paginas/`: cada página es una hoja en blanco de 600 × 800.
const hoja = await (await navegador.newPage({ viewport: { width: 600, height: 800 } })).screenshot();
mkdirSync("capturas", { recursive: true });

async function abrir(examen: string, width: number, height: number) {
  const page = await navegador.newPage({ viewport: { width, height } });
  await page.route("**/paginas/**", (r) => r.fulfill({ contentType: "image/png", body: hoja }));
  await page.goto(`http://127.0.0.1:8691/${examen}/`);
  await page.getByText(/^Tipo test \(/).click();
  return page;
}
const visible = (page: Page, sel: string) => page.locator(sel).first().isVisible();
// En el móvil, editar va en el menú «⋯».
async function editar(page: Page) {
  if (await page.getByTitle("Más").isVisible()) await page.getByTitle("Más").click();
  await page.getByTitle("Editar la pregunta").click();
}
const contestar = (page: Page) => page.locator(".opcion button").first().click();
const rejilla = (page: Page) => page.getByTitle("Las preguntas").filter({ visible: true });
async function irA(page: Page, n: number) {
  await rejilla(page).click();
  await page.locator(".cuadricula button", { hasText: new RegExp(`^${n}$`) }).click();
}

// La navegación por la cuadrícula, la edición y el simulacro, sobre una página con la pregunta 1 contestada.
async function navegar(page: Page, nombre: string) {
  // La ronda a medias vuelve al recargar, en la primera pregunta sin contestar.
  await page.reload();
  await rejilla(page).getByText("2 / 12").waitFor();
  await irA(page, 4);
  await rejilla(page).getByText("4 / 12").waitFor();
  assert.equal(await page.locator(".icono").count(), 0, "una pregunta saltada espera su respuesta");
  await rejilla(page).click();
  assert(await page.getByText("Terminar la sesión").isDisabled(), "terminar queda gris con preguntas sin contestar");
  await page.getByText("Te faltan 11 preguntas. Contéstalas para terminar.").waitFor();
  await page.screenshot({ path: `capturas/${nombre}-cuadricula.png` });
  await page.locator(".cuadricula button", { hasText: /^1$/ }).click();
  assert(await page.locator(".opcion button").first().isDisabled(), "una pregunta contestada no cambia de respuesta");

  // En el móvil, el menú «⋯» abre editar, la nota y copiar, y un toque fuera lo cierra.
  if (await page.getByTitle("Más").isVisible()) {
    await page.getByTitle("Más").click();
    await page.screenshot({ path: `capturas/${nombre}-menu.png` });
    await page.mouse.click(200, 400);
    assert(!(await page.getByTitle("Copiar la pregunta y las opciones").isVisible()), "un toque fuera cierra el menú");
  }
  // En el móvil, el texto de arriba a la izquierda se lee entero, sin «…».
  if (nombre === "movil") {
    await page.locator(".donde").evaluate((e) => (e.textContent = "Propiedad intelectual · sin ver"));
    await page.screenshot({ path: "capturas/movil-arriba.png", clip: { x: 0, y: 0, width: 390, height: 80 } });
    assert(await page.locator(".donde").evaluate((e) => e.scrollWidth <= e.clientWidth), "el texto de arriba se lee entero");
  }
  // La edición entra en `preguntas.json` con el mismo id, y Cancelar deja la pregunta como estaba.
  await editar(page);
  assert.equal(await page.locator(".hoja").count(), 0, "la edición oculta la hoja");
  assert.equal(await page.locator(".nota").count(), 0, "la edición oculta la nota");
  await page.screenshot({ path: `capturas/${nombre}-editar.png` });
  await page.locator("textarea").first().fill("¿Un enunciado editado?");
  await page.getByText("Guardar").click();
  await page.getByText("¿Un enunciado editado?").waitFor();
  const banco = await page.evaluate(() => fetch("api/inicio").then((r) => r.json()));
  assert.equal(banco.banco.filter((q: { pregunta: string }) => q.pregunta === "¿Un enunciado editado?").length, 1, "la edición llega al banco");
  await editar(page);
  await page.locator("textarea").first().fill("Otro");
  await page.getByText("Cancelar").click();
  await page.getByText("¿Un enunciado editado?").waitFor();

  // El simulacro solo marca la opción elegida, y corrige al final.
  page.once("dialog", (d) => d.accept());
  await page.getByTitle("Abandonar").click();
  await page.getByText(/^Simulacro/).click();
  assert(!(await visible(page, ".pomodoro")), "en un simulacro solo se ve su reloj");
  // Las respuestas del simulacro entran en el historial al terminar.
  const historial = join(raiz, nombre, "respuestas.jsonl");
  const lineas = () => (existsSync(historial) ? readFileSync(historial, "utf8").trim().split("\n").length : 0);
  const antes = lineas();
  for (let k = 0; k < 20; k++) {
    if (k === 0) {
      // Una respuesta del simulacro cambia hasta el final: la última elegida queda marcada.
      await page.locator(".opcion button").first().click();
      await page.locator(".opcion").last().locator("button").first().click();
      assert((await page.locator(".opcion").last().getAttribute("class"))!.includes("border-acento"), "la respuesta del simulacro cambia");
      assert(!(await page.locator(".opcion").first().getAttribute("class"))!.includes("border-acento"), "solo la última elegida queda marcada");
    } else await page.locator(".opcion button").first().click();
    assert.equal(await page.locator(".icono").count(), 0, "el simulacro no corrige al momento");
    // En el PC, el simulacro avanza con "Siguiente ›", porque la hoja no lleva la corrección.
    if (k < 19) await page.getByRole("button", { name: /^Siguiente/ }).filter({ visible: true }).last().click();
  }
  await page.waitForTimeout(500);
  assert.equal(lineas(), antes, "el simulacro no manda respuestas al historial antes de terminar");
  // Con todas contestadas, el móvil termina con el botón grande desde cualquier pregunta, y el PC desde la cuadrícula.
  await irA(page, 3);
  if (nombre === "movil") await page.getByRole("button", { name: "Terminar", exact: true }).filter({ visible: true }).click();
  else {
    await rejilla(page).click();
    await page.getByText("Terminar el simulacro").click();
  }
  await page.getByText("El simulacro ha terminado").waitFor();
  await page.waitForTimeout(500);
  assert.equal(lineas() - antes, 20, "cada pregunta del simulacro entra una vez en el historial, al terminar");
  await page.screenshot({ path: `capturas/${nombre}-final.png` });
  await page.locator(".cuadricula button", { hasText: /^5$/ }).click();
  await page.getByText(/^Revisión · /).waitFor();
  assert((await page.locator(".icono").count()) > 0, "la revisión corrige");
  await page.getByRole("button", { name: "Resultado", exact: true }).filter({ visible: true }).click();
  await page.getByText("El simulacro ha terminado").waitFor();
}

// La sesión de desarrollo: el botón del inicio, la pregunta, el texto tras recargar, la solución, las 2 notas y volver a una hecha.
async function desarrollar(examen: string, width: number, height: number) {
  const page = await navegador.newPage({ viewport: { width, height } });
  await page.route("**/paginas/**", (r) => r.fulfill({ contentType: "image/png", body: hoja }));
  await page.goto(`http://127.0.0.1:8691/${examen}/`);
  const boton = page.locator("button.desarrollo");
  await boton.getByText("Desarrollo (2)").waitFor();
  const sesion = (await page.getByText(/^Tipo test \(/).boundingBox())!;
  assert((await boton.boundingBox())!.x > sesion.x, "el botón de desarrollo va a la derecha de la sesión");
  await page.screenshot({ path: `capturas/${examen}-inicio-desarrollo.png`, fullPage: true });
  await boton.click();
  await page.getByText(/^1 \/ 2 · RGPD · sin ver$/).waitFor();
  await page.screenshot({ path: `capturas/${examen}-desarrollo.png` });
  // El texto viaja con la ronda guardada, y vuelve al recargar.
  await page.locator(".respuesta").fill("Mi respuesta");
  await page.waitForTimeout(800);
  await page.reload();
  assert.equal(await page.locator(".respuesta").inputValue(), "Mi respuesta", "el texto vuelve al recargar");
  await page.getByText("Ver respuesta modelo").click();
  assert.equal(await page.locator(".puntos li").count(), 4, "la solución lleva los 4 puntos");
  const texto = (await page.locator(".respuesta").boundingBox())!;
  const modelo = (await page.locator(".modelo").boundingBox())!;
  if (width >= 900) assert(modelo.x > texto.x + texto.width - 1, "en el PC, el modelo va al lado del texto");
  else assert(modelo.y > texto.y + texto.height - 1, "en el móvil, el modelo va debajo del texto");
  await page.screenshot({ path: `capturas/${examen}-desarrollo-solucion.png`, fullPage: true });
  await page.locator(".puntos .ver-pagina").first().click();
  // El punto abre su página a pantalla completa.
  await page.locator(".pagina img").waitFor();
  await page.goBack();
  assert.equal(await page.locator(".pagina").count(), 0, "el botón de atrás cierra la página");
  // Una respuesta calificada guarda 2 líneas, y «Corregir luego» guarda solo la respuesta.
  await page.getByText("La sabía", { exact: true }).click();
  await page.getByText(/^2 \/ 2 · /).waitFor();
  assert.equal(await page.locator(".respuesta").inputValue(), "", "la pregunta siguiente empieza en blanco");
  assert.equal(await page.locator(".corregir-luego").count(), 0, "«Corregir luego» sale con la respuesta modelo");
  // Anterior enseña la pregunta hecha con su nota, sin cambiarla.
  await page.locator(".desarrollo").getByTitle("Anterior").click();
  await page.locator(".hecha").getByText("La sabía").waitFor();
  assert.match(await page.locator(".metadatos").innerText(), / · vuelve en \d+ (horas?|días?)$/, "la pregunta hecha dice cuándo vuelve");
  assert(await page.locator(".respuesta").evaluate((e: HTMLTextAreaElement) => e.readOnly), "una pregunta hecha no se edita");
  await page.locator(".desarrollo").getByTitle("Siguiente").click();
  await page.getByText(/^2 \/ 2 · /).waitFor();
  await page.getByText("Ver respuesta modelo").click();
  assert(await page.locator(".corregir-luego").isDisabled(), "un cuadro vacío no se deja sin corregir");
  await page.locator(".respuesta").fill("Otra respuesta");
  await page.screenshot({ path: `capturas/${examen}-desarrollo-notas.png`, fullPage: true });
  await page.locator(".corregir-luego").click();
  await page.getByText(/^Tipo test \(/).waitFor();
  await page.getByText("1 respuesta sin corregir").waitFor();
  await page.screenshot({ path: `capturas/${examen}-sin-corregir.png`, fullPage: true });
  await page.waitForTimeout(500);
  const lineas = readFileSync(join(raiz, examen, "desarrollo.jsonl"), "utf8").trim().split("\n");
  assert.equal(lineas.length, 3, "las 2 respuestas y la corrección de la primera llegan a desarrollo.jsonl");
  assert(!existsSync(join(raiz, examen, "respuestas.jsonl")), "una nota de desarrollo no toca el historial de test");
  await page.close();
}

// El simulacro mixto: los 2 desarrollos antes del test, el texto tras recargar, y su corrección en el final.
async function mixto(examen: string, width: number, height: number) {
  const page = await navegador.newPage({ viewport: { width, height } });
  await page.goto(`http://127.0.0.1:8691/${examen}/`);
  await page.getByText("Simulacro (2 de desarrollo y 20 de test)").click();
  await page.locator(".desarrollo-del-simulacro").getByText(/ · desarrollo$/).waitFor();
  assert.equal(await page.locator(".modelo, .puntos").count(), 0, "el desarrollo del simulacro solo enseña el enunciado y el cuadro");
  await page.locator(".respuesta").fill("Mi desarrollo");
  await page.screenshot({ path: `capturas/${examen}-desarrollo.png` });
  await page.waitForTimeout(800);
  await page.reload();
  assert.equal(await page.locator(".respuesta").inputValue(), "Mi desarrollo", "el texto del simulacro vuelve al recargar");
  // El segundo desarrollo queda en blanco, y Siguiente lleva a la primera pregunta de test.
  await page.getByRole("button", { name: "Siguiente ›" }).click();
  await page.getByTitle("Las preguntas").getByText("2 / 22").waitFor();
  await page.getByRole("button", { name: "Siguiente ›" }).click();
  await rejilla(page).getByText("3 / 22").waitFor();
  for (let k = 0; k < 20; k++) {
    await page.locator(".opcion button").first().click();
    if (k < 19) await page.getByRole("button", { name: /^Siguiente/ }).filter({ visible: true }).last().click();
  }
  await rejilla(page).click();
  await page.getByText("Terminar el simulacro").click();
  await page.getByText("El simulacro ha terminado").waitFor();
  assert.equal(await page.locator(".correccion").count(), 2, "cada desarrollo sale en el final");
  await page.locator(".nota-del-simulacro").getByText("desarrollo sin corregir").waitFor();
  assert.equal(await page.locator(".cuadricula button").count(), 20, "la cuadrícula del final es del test");
  await page.screenshot({ path: `capturas/${examen}-final.png`, fullPage: true });
  // El desarrollo vacío ya es «No la sabía», y el otro se corrige en el final.
  await page.locator(".correccion").nth(1).getByText("No la sabía").waitFor();
  await page.locator(".correccion").first().getByRole("button", { name: "La sabía", exact: true }).click();
  await page.locator(".nota-del-simulacro").getByText(/Nota: .+ sobre 10/).waitFor();
  await page.waitForTimeout(500);
  const lineas = readFileSync(join(raiz, examen, "desarrollo.jsonl"), "utf8").trim().split("\n");
  assert.equal(lineas.length, 3, "la respuesta con texto, el Again del vacío y la corrección llegan a desarrollo.jsonl");
  await page.close();
}

try {
  // La lista: cada examen de la raíz abre su ruta, y su inicio lleva de vuelta a la lista.
  for (const [nombre, width, height] of [["movil", 390, 844], ["pc", 1280, 800]] as const) {
    const lista = await navegador.newPage({ viewport: { width, height } });
    await lista.goto("http://127.0.0.1:8691/");
    await lista.locator(".examen").first().waitFor();
    assert.equal(await lista.locator(".examen").count(), 4, "la lista lleva los 4 exámenes de la raíz");
    await lista.screenshot({ path: `capturas/${nombre}-lista.png` });
    await lista.locator(`.examen[href="/${nombre}/"]`).click();
    await lista.getByText(/^Tipo test \(/).waitFor();

    // La selección: un tema desmarcado sale de la tarjeta y queda tras recargar. Sin nada marcado, los botones no van.
    const temas = lista.locator(".tema input");
    assert.equal(await temas.count(), 5, "una casilla por tema");
    await temas.nth(1).uncheck();
    await lista.getByText(/^Solo se muestran 4 de 5 temas\. Cambia la selección abajo\.$/).waitFor();
    await lista.reload();
    await lista.getByText(/^Solo se muestran 4 de 5 temas\. Cambia la selección abajo\.$/).waitFor();
    assert(!(await temas.nth(1).isChecked()), "la selección vuelve al recargar");
    await lista.getByText(/^0 sabes \(de \d+\)$/).waitFor();
    assert.equal(await lista.locator(".tema").getByText(/ · 0\/\d+ desarrollo$/).count() > 0, true, "un tema cuenta sus desarrollos");
    // Un tema empieza con sus apartados plegados, y su flecha los abre.
    assert.equal(await lista.locator(".apartado").count(), 0, "los apartados empiezan plegados");
    await lista.locator(".abrir-tema").first().click();
    await lista.locator(".apartado input").first().uncheck();
    assert(await lista.evaluate(() => (document.querySelector(".tema input") as HTMLInputElement).indeterminate), "un tema a medias");
    await lista.screenshot({ path: `capturas/${nombre}-seleccion.png`, fullPage: true });
    // Un clic en un tema a medias lo marca entero.
    await temas.nth(0).click();
    assert(await lista.locator(".apartado input").first().isChecked(), "un clic en un tema a medias marca sus apartados");
    for (let k = 0; k < 5; k++) await temas.nth(k).setChecked(false);
    assert(await lista.getByText(/^Tipo test \(/).isDisabled(), "sin nada marcado, la sesión no empieza");
    assert(await lista.getByText(/^Simulacro/).isDisabled(), "sin nada marcado, el simulacro no empieza");
    for (let k = 0; k < 5; k++) await temas.nth(k).setChecked(true);
    assert.equal(await lista.getByText(/ temas$/).count(), 0, "con todo marcado, la tarjeta no nombra los temas");
    // La tarjeta de los temas se pliega entera, y sigue plegada tras recargar.
    await lista.locator(".plegar-temas").click();
    assert.equal(await temas.count(), 0, "la tarjeta plegada esconde los temas");
    await lista.getByText("5 de 5 marcados").waitFor();
    await lista.reload();
    await lista.getByText("5 de 5 marcados").waitFor();
    assert.equal(await temas.count(), 0, "la tarjeta sigue plegada tras recargar");
    await lista.screenshot({ path: `capturas/${nombre}-temas-plegados.png` });
    await lista.locator(".plegar-temas").click();
    await lista.locator(".volver").click();
    await lista.locator(".examen").first().waitFor();
    assert.equal(new URL(lista.url()).pathname, "/", "el enlace del inicio vuelve a la lista");
    await lista.route("**/api/examenes", (r) => r.fulfill({ json: [] }));
    await lista.reload();
    await lista.getByText("No hay exámenes").waitFor();
    await lista.close();
  }

  // El cronómetro de pausas: los controles salen sobre la flecha, y la fase sigue tras recargar.
  const reloj = await navegador.newPage({ viewport: { width: 1280, height: 800 } });
  await reloj.goto("http://127.0.0.1:8691/");
  const pomodoro = reloj.locator(".pomodoro");
  await pomodoro.getByText("25:00").waitFor();
  assert(!(await reloj.getByTitle("+5 min").isVisible()), "los controles no se ven sin el ratón sobre la flecha");
  await reloj.getByTitle("Los controles").hover();
  await reloj.getByTitle("+5 min").click();
  await pomodoro.getByText("30:00").waitFor();
  await reloj.getByTitle("Saltar el estudio").click();
  await pomodoro.getByText("5:00").waitFor();
  await reloj.getByTitle("Saltar la pausa").click();
  await reloj.getByTitle("Saltar el estudio").click();
  await reloj.getByTitle("Saltar la pausa").click();
  await reloj.getByTitle("Saltar el estudio").click();
  await pomodoro.getByText("20:00").waitFor();
  await pomodoro.getByTitle("Empezar").first().click();
  await reloj.reload();
  await pomodoro.getByText(/^\s*19:5\d\s*$/).waitFor();
  // Una fase que acaba con la página cerrada pasa a la siguiente al volver, y espera el clic.
  await reloj.evaluate(() => localStorage.setItem("pomodoro", JSON.stringify({ ...JSON.parse(localStorage.getItem("pomodoro")!), fin: Date.now() })));
  await reloj.reload();
  await pomodoro.getByText("25:00").waitFor();
  // Reiniciar vuelve al estudio 1 desde cualquier fase.
  await reloj.getByTitle("Los controles").hover();
  await reloj.getByTitle("Saltar el estudio").click();
  await pomodoro.getByText("5:00").waitFor();
  await reloj.getByTitle("Volver al estudio 1").click();
  await pomodoro.getByText("25:00").waitFor();
  await pomodoro.getByText("Estudio · 1 de 3").waitFor({ state: "attached" });
  await reloj.close();

  await desarrollar("movil", 390, 844);
  await desarrollar("pc", 1280, 800);
  await mixto("mixto-movil", 390, 844);
  await mixto("mixto-pc", 1280, 800);

  // El móvil: la página de la fuente se abre a pantalla completa, y las otras opciones se pliegan.
  const movil = await abrir("movil", 390, 844);
  await movil.screenshot({ path: "capturas/movil-pregunta.png" });
  assert(!(await visible(movil, ".tirador")), "el móvil no tiene tiradores");
  assert(!(await visible(movil, ".nota")), "el móvil oculta la nota sin la clave");
  await contestar(movil);
  assert(await visible(movil, ".hoja p"), "el móvil muestra la explicación");
  assert(!(await visible(movil, ".hoja img")), "el móvil no muestra la página en la hoja");
  assert(await visible(movil, ".chip"), "el móvil pliega las otras opciones");
  const marcadas = await movil.locator(".opcion:has(.icono)").count();
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
  const caja = (await pc.locator(".tirador").nth(1).boundingBox())!;
  await pc.mouse.move(caja.x + 12, 400);
  assert.equal(await asa(), "block", "el asa sale al pasar el ratón");
  const antes = (await pc.locator(".hoja").boundingBox())!.x;
  await pc.mouse.down();
  await pc.mouse.move(0.35 * 1280, 400, { steps: 5 });
  await pc.mouse.up();
  assert((await pc.locator(".hoja").boundingBox())!.x < antes - 100, "el separador mueve la hoja");
  const guardadas = JSON.parse(await pc.evaluate(() => localStorage.getItem("pc-columnas")!));
  assert(Math.abs(guardadas.s - 0.35) < 0.01, "las columnas quedan en localStorage");
  await pc.screenshot({ path: "capturas/pc-solucion.png" });

  // La nota llega a `notas.json`. El botón de la nota la oculta en ese examen.
  await pc.locator(".nota").fill("Mi nota");
  await pc.waitForTimeout(800);
  const notas = await pc.evaluate(() => fetch("api/notas").then((r) => r.json()));
  assert.deepEqual(Object.values(notas), ["Mi nota"], "la nota llega al servidor");
  await pc.locator(".ver-nota").click();
  assert(!(await visible(pc, ".nota")), "el botón oculta la nota");
  assert.equal(await pc.evaluate(() => localStorage.getItem("pc-notas-visibles")), "no");
  await pc.locator(".ver-nota").click();

  // El botón de copiar deja la pregunta y sus opciones en el portapapeles, sin la solución.
  await pc.context().grantPermissions(["clipboard-read", "clipboard-write"]);
  await pc.locator(".copiar").click();
  const copiado = await pc.evaluate(() => navigator.clipboard.readText());
  assert.match(copiado, /\n\nA\) .+\nB\) /, "el portapapeles lleva las opciones con su letra");

  // Una ventana estrecha del PC usa el diseño del móvil, sin recargar.
  await pc.setViewportSize({ width: 800, height: 800 });
  assert(await visible(pc, ".ver-pagina"), "una ventana estrecha usa el diseño del móvil");
  assert(!(await visible(pc, ".tirador")), "una ventana estrecha no tiene tiradores");
  await pc.screenshot({ path: "capturas/pc-estrecho.png" });
  await pc.setViewportSize({ width: 1280, height: 800 });

  await navegar(movil, "movil");
  await navegar(pc, "pc");
  console.log("Todas las pantallas pasan.");
} finally {
  await navegador.close();
  servidor.kill();
}
