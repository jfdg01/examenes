//! Las herramientas del banco de un examen: la comprobación de `preguntas.json`, las imágenes de las páginas
//! de los PDFs y los arreglos de una revisión. Las 3 leen la carpeta del examen. La comprobación y las
//! imágenes llaman a `pdfinfo` y `pdftoppm` de poppler.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use nucleo::{Examen, Pregunta};
use serde::Deserialize;

fn leer<T: serde::de::DeserializeOwned>(archivo: &Path) -> Result<T, String> {
    let texto = fs::read_to_string(archivo).map_err(|e| format!("{}: {e}", archivo.display()))?;
    serde_json::from_str(&texto).map_err(|e| format!("{}: {e}", archivo.display()))
}

fn escribir<T: serde::Serialize>(archivo: &Path, datos: &T) -> Result<(), String> {
    let texto = serde_json::to_string_pretty(datos).map_err(|e| e.to_string())? + "\n";
    fs::write(archivo, texto).map_err(|e| format!("{}: {e}", archivo.display()))
}

/// Un concepto de una sigla, que tiene una sola pregunta.
fn sigla(concepto: &str) -> bool {
    concepto.starts_with("sigla-")
}

/// Las páginas de un PDF, según `pdfinfo`.
pub fn paginas(pdf: &Path) -> Result<u32, String> {
    let salida = Command::new("pdfinfo").arg(pdf).output().map_err(|e| format!("pdfinfo: {e}"))?;
    String::from_utf8_lossy(&salida.stdout)
        .lines()
        .find_map(|l| l.strip_prefix("Pages:"))
        .and_then(|n| n.trim().parse().ok())
        .ok_or_else(|| format!("pdfinfo no lee {}", pdf.display()))
}

/// Los errores del banco: cada pregunta que rompe el formato de `BANCO.md`.
pub fn comprobar(preguntas: &[Pregunta], examen: &Examen, carpeta: &Path) -> Vec<String> {
    let distractores = (examen.opciones as usize).saturating_sub(1);
    let mut errores = Vec::new();
    let mut vistos = HashSet::new();
    let mut paginas_de: HashMap<&str, Result<u32, String>> = HashMap::new();
    for p in preguntas {
        let id = &p.id;
        if !vistos.insert(id) {
            errores.push(format!("{id}: id repetido"));
        }
        if p.distractores.len() != distractores {
            errores.push(format!("{id}: {} distractores, y no {distractores}", p.distractores.len()));
        }
        if p.distractores.contains(&p.correcta) {
            errores.push(format!("{id}: un distractor es la respuesta correcta: {}", p.correcta));
        }
        let lineas: Vec<&str> = p.explicacion.lines().filter(|l| l.starts_with('✗')).collect();
        if lineas.len() > p.distractores.len() {
            errores.push(format!("{id}: {} líneas ✗, y {} distractores", lineas.len(), p.distractores.len()));
        }
        for (i, (linea, d)) in lineas.into_iter().zip(&p.distractores).enumerate() {
            if !clave(linea).is_some_and(|(c, _)| d.to_lowercase().contains(&c.to_lowercase())) {
                errores.push(format!("{id}: la línea ✗ {} no lleva una clave «…» de su distractor: {d}", i + 1));
            }
        }
        let nombre = p.fuente.pdf.as_str();
        let pdf = carpeta.join("pdfs").join(nombre);
        let Some(tema) = examen.pdfs.get(nombre).filter(|_| pdf.is_file()) else {
            errores.push(format!("{id}: el pdf no está en examen.json o en pdfs/: {nombre}"));
            continue;
        };
        if !tema.apartados.contains(&p.apartado) {
            errores.push(format!("{id}: el apartado {} no es un apartado de {nombre} en examen.json", p.apartado));
        }
        match paginas_de.entry(nombre).or_insert_with(|| paginas(&pdf)) {
            Ok(n) if p.fuente.pagina == 0 || p.fuente.pagina > *n => {
                errores.push(format!("{id}: la página {} no está entre las {n} de {nombre}", p.fuente.pagina))
            }
            Ok(_) => {}
            Err(e) => errores.push(format!("{id}: {e}")),
        }
    }
    for (pdf, tema) in &examen.pdfs {
        for a in tema.apartados.iter().filter(|a| !examen.apartados.contains_key(*a)) {
            errores.push(format!("{pdf}: el apartado {a} no tiene nombre en apartados de examen.json"));
        }
    }
    let mut cuenta: Vec<(&str, usize)> = Vec::new();
    for p in preguntas {
        match cuenta.iter_mut().find(|(c, _)| *c == p.concepto) {
            Some((_, n)) => *n += 1,
            None => cuenta.push((&p.concepto, 1)),
        }
    }
    for (concepto, n) in cuenta {
        let esperado = if sigla(concepto) { 1 } else { 3 };
        if n != esperado {
            let s = if n == 1 { "" } else { "s" };
            errores.push(format!("{concepto}: {n} pregunta{s}, y no {esperado}"));
        }
    }
    errores
}

/// Los `id` de `respuestas.jsonl` que no están en el banco, sin repetir y en orden.
pub fn huerfanas(preguntas: &[Pregunta], historial: &Path) -> Result<Vec<String>, String> {
    #[derive(Deserialize)]
    struct Linea {
        id: String,
    }
    let texto = match fs::read_to_string(historial) {
        Ok(texto) => texto,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("{}: {e}", historial.display())),
    };
    let ids: HashSet<&str> = preguntas.iter().map(|p| p.id.as_str()).collect();
    let mut sin_banco = BTreeSet::new();
    for linea in texto.lines().filter(|l| !l.trim().is_empty()) {
        let r: Linea = serde_json::from_str(linea).map_err(|e| format!("{}: {e}", historial.display()))?;
        if !ids.contains(r.id.as_str()) {
            sin_banco.insert(r.id);
        }
    }
    Ok(sin_banco.into_iter().collect())
}

/// Las palabras absolutas de un texto: "siempre", "nunca" y "solo", sin contar "un solo" ni "una sola".
fn absolutos(texto: &str) -> BTreeSet<String> {
    let palabras: Vec<String> =
        texto.split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(str::to_lowercase).collect();
    let solo = |w: &str| matches!(w, "solo" | "sólo" | "sola" | "sóla");
    let absoluto = |w: &str| matches!(w, "siempre" | "nunca") || solo(w) || solo(w.strip_suffix('s').unwrap_or(""));
    palabras
        .iter()
        .enumerate()
        .filter(|(i, w)| absoluto(w) && !(solo(w) && *i > 0 && matches!(palabras[i - 1].as_str(), "un" | "una")))
        .map(|(_, w)| w.clone())
        .collect()
}

/// Los avisos de las pistas de la regla 6 de `BANCO.md`, que un revisor juzga.
pub fn pistas(preguntas: &[Pregunta]) -> Vec<String> {
    let mut avisos = Vec::new();
    for p in preguntas {
        let correcta = absolutos(&p.correcta);
        let sobran: BTreeSet<String> =
            p.distractores.iter().flat_map(|d| absolutos(d)).filter(|w| !correcta.contains(w)).collect();
        if !sobran.is_empty() {
            let lista = sobran.into_iter().collect::<Vec<_>>().join(", ");
            avisos.push(format!("{}: un distractor dice {lista}, y la correcta no", p.id));
        }
    }
    let largo = |s: &String| s.chars().count();
    let normales: Vec<&Pregunta> = preguntas.iter().filter(|p| !sigla(&p.concepto) && !p.distractores.is_empty()).collect();
    let larga = normales.iter().filter(|p| largo(&p.correcta) > p.distractores.iter().map(largo).max().unwrap_or(0)).count();
    if !normales.is_empty() && larga * 3 > normales.len() {
        avisos.push(format!("la correcta es la opción más larga en {larga} de {} preguntas, más de 1 de cada 3", normales.len()));
    }
    avisos
}

/// La clave `«…»` de una línea `✗` de la explicación, y el matiz que la sigue.
fn clave(linea: &str) -> Option<(&str, &str)> {
    let (_, resto) = linea.strip_prefix('✗')?.split_once('«')?;
    let (clave, matiz) = resto.split_once('»').filter(|(c, _)| !c.trim().is_empty())?;
    Some((clave, matiz.trim_start_matches(':').trim()))
}

/// Los avisos de las explicaciones: una línea `✓` de menos de 8 palabras, y una línea `✗` con el matiz de
/// otra línea `✗` del mismo apartado.
pub fn explicaciones(preguntas: &[Pregunta]) -> Vec<String> {
    let mut avisos = Vec::new();
    let mut matices: HashMap<(&str, &str), &str> = HashMap::new();
    for p in preguntas {
        if let Some(l) = p.explicacion.lines().find(|l| l.starts_with('✓')) {
            let n = l.trim_start_matches('✓').split_whitespace().count();
            if n < 8 {
                avisos.push(format!("{}: la línea ✓ tiene {n} palabras, menos de 8", p.id));
            }
        }
        for (_, matiz) in p.explicacion.lines().filter_map(clave).filter(|(_, m)| !m.is_empty()) {
            if let Some(otra) = matices.insert((&p.apartado, matiz), &p.id) {
                avisos.push(format!("{}: la línea ✗ repite el matiz de {otra}: {matiz}", p.id));
            }
        }
    }
    avisos
}

/// Los avisos de las páginas de cada PDF con alguna pregunta: una página sin pregunta y sin razón en
/// `sin_pregunta`, y una página con pregunta y con razón.
pub fn paginas_sin_pregunta(preguntas: &[Pregunta], examen: &Examen, carpeta: &Path) -> Vec<String> {
    let mut citadas: BTreeMap<&str, BTreeSet<u32>> = BTreeMap::new();
    for p in preguntas {
        citadas.entry(&p.fuente.pdf).or_default().insert(p.fuente.pagina);
    }
    let mut avisos = Vec::new();
    for (pdf, con) in citadas {
        // Un PDF que pdfinfo no lee ya es un error de `comprobar`.
        let Ok(n) = paginas(&carpeta.join("pdfs").join(pdf)) else { continue };
        let razon = |i: &u32| examen.sin_pregunta.get(pdf).is_some_and(|m| m.contains_key(&i.to_string()));
        let lista = |v: Vec<u32>| v.iter().map(u32::to_string).collect::<Vec<_>>().join(", ");
        let sin: Vec<u32> = (1..=n).filter(|i| !con.contains(i) && !razon(i)).collect();
        if !sin.is_empty() {
            avisos.push(format!("{pdf}: páginas sin pregunta y sin razón en sin_pregunta: {}", lista(sin)));
        }
        let las_dos: Vec<u32> = con.iter().copied().filter(razon).collect();
        if !las_dos.is_empty() {
            avisos.push(format!("{pdf}: páginas con pregunta y con razón en sin_pregunta: {}", lista(las_dos)));
        }
    }
    avisos
}

/// Comprueba el banco de la carpeta y escribe el informe. Devuelve `true` si el banco no tiene errores.
pub fn informe(carpeta: &Path) -> Result<bool, String> {
    let examen: Examen = leer(&carpeta.join("examen.json"))?;
    let preguntas: Vec<Pregunta> = leer(&carpeta.join("preguntas.json"))?;
    let errores = comprobar(&preguntas, &examen, carpeta);
    if errores.is_empty() {
        println!("El banco no tiene errores.");
    }
    for e in &errores {
        println!("{e}");
    }
    let avisos = [pistas(&preguntas), explicaciones(&preguntas), paginas_sin_pregunta(&preguntas, &examen, carpeta)];
    for aviso in avisos.concat() {
        println!("Aviso: {aviso}");
    }
    let sin_banco = huerfanas(&preguntas, &carpeta.join("respuestas.jsonl"))?;
    if !sin_banco.is_empty() {
        println!("Aviso: {} id del historial no están en el banco: {}", sin_banco.len(), sin_banco.join(", "));
    }
    Ok(errores.is_empty())
}

/// Saca `paginas/<pdf sin .pdf>-<página>.webp`, a 200 ppp y calidad 60, de cada página de los PDFs de `pdfs/`.
/// Una imagen que ya existe se queda.
pub fn sacar(carpeta: &Path) -> Result<(), String> {
    let destino = carpeta.join("paginas");
    fs::create_dir_all(&destino).map_err(|e| format!("{}: {e}", destino.display()))?;
    let leidos = fs::read_dir(carpeta.join("pdfs")).map_err(|e| format!("{}/pdfs: {e}", carpeta.display()))?;
    let mut pdfs: Vec<_> =
        leidos.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "pdf")).collect();
    pdfs.sort();
    for pdf in pdfs {
        let base = pdf.file_stem().unwrap().to_string_lossy();
        for pagina in 1..=paginas(&pdf)? {
            let nombre = destino.join(format!("{base}-{pagina}"));
            if nombre.with_added_extension("webp").exists() {
                continue;
            }
            let n = pagina.to_string();
            let estado = Command::new("pdftoppm")
                .args(["-png", "-r", "200", "-f", &n, "-l", &n, "-singlefile"])
                .arg(&pdf)
                .arg(&nombre)
                .status()
                .map_err(|e| format!("pdftoppm: {e}"))?;
            if !estado.success() {
                return Err(format!("pdftoppm falla en la página {pagina} de {}", pdf.display()));
            }
            let png = nombre.with_added_extension("png");
            let estado = Command::new("ffmpeg")
                .args(["-v", "error", "-y", "-i"])
                .arg(&png)
                .args(["-c:v", "libwebp", "-quality", "60"])
                .arg(nombre.with_added_extension("webp"))
                .status()
                .map_err(|e| format!("ffmpeg: {e}"))?;
            fs::remove_file(&png).map_err(|e| format!("{}: {e}", png.display()))?;
            if !estado.success() {
                return Err(format!("ffmpeg falla en la página {pagina} de {}", pdf.display()));
            }
        }
    }
    Ok(())
}

/// Un fallo del informe de un revisor. La revisión de `BANCO.md` define cada campo.
#[derive(Deserialize)]
pub struct Fallo {
    /// El `id` de la pregunta, o `null` para un fallo sin pregunta, como una página sin concepto.
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub gravedad: String,
    #[serde(default)]
    pub fallo: String,
    /// Un `Arreglo` que el revisor propone, o `null`.
    #[serde(default)]
    pub arreglo: serde_json::Value,
}

/// Los campos de una pregunta que un arreglo cambia. Otro campo deja el fallo para arreglarlo a mano.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Arreglo {
    pregunta: Option<String>,
    correcta: Option<String>,
    distractores: Option<Vec<String>>,
    explicacion: Option<String>,
    fuente: Option<nucleo::Fuente>,
}

/// Aplica cada arreglo al banco, y devuelve los fallos que un agente arregla a mano: un fallo sin arreglo,
/// con un campo que un arreglo no cambia, o con un `id` fuera del banco.
pub fn aplicar(banco: &mut [Pregunta], fallos: Vec<Fallo>) -> Vec<Fallo> {
    let mut pendientes = Vec::new();
    for f in fallos {
        let arreglo = serde_json::from_value::<Arreglo>(f.arreglo.clone()).ok().filter(|a| {
            a.pregunta.is_some()
                || a.correcta.is_some()
                || a.distractores.is_some()
                || a.explicacion.is_some()
                || a.fuente.is_some()
        });
        let (Some(a), Some(p)) = (arreglo, banco.iter_mut().find(|p| Some(&p.id) == f.id.as_ref())) else {
            pendientes.push(f);
            continue;
        };
        if let Some(v) = a.pregunta {
            p.pregunta = v;
        }
        if let Some(v) = a.correcta {
            p.correcta = v;
        }
        if let Some(v) = a.distractores {
            p.distractores = v;
        }
        if let Some(v) = a.explicacion {
            p.explicacion = v;
        }
        if let Some(v) = a.fuente {
            p.fuente = v;
        }
    }
    pendientes
}

/// Aplica los informes de la revisión a `preguntas.json` de la carpeta, y lista los fallos sin aplicar.
pub fn revision(carpeta: &Path, informes: &[String]) -> Result<(), String> {
    let archivo = carpeta.join("preguntas.json");
    let mut banco: Vec<Pregunta> = leer(&archivo)?;
    let mut fallos = Vec::new();
    for informe in informes {
        fallos.extend(leer::<Vec<Fallo>>(Path::new(informe))?);
    }
    let total = fallos.len();
    let pendientes = aplicar(&mut banco, fallos);
    escribir(&archivo, &banco)?;
    println!("{} arreglos aplicados.", total - pendientes.len());
    for f in pendientes {
        println!("A mano: {} ({}): {}", f.id.as_deref().unwrap_or("sin id"), f.gravedad, f.fallo);
    }
    Ok(())
}

/// Prepara un lote de apartados: quita del banco sus preguntas y de `sin_pregunta` sus PDFs, y crea la carpeta
/// de trabajo de cada apartado. Falla sin cambiar nada con un historial o una ronda que usan los `id` del lote,
/// porque los `id` nuevos los reusan.
pub fn preparar(carpeta: &Path, apartados: &[String]) -> Result<(), String> {
    let mut examen: Examen = leer(&carpeta.join("examen.json"))?;
    let archivo = carpeta.join("preguntas.json");
    let banco: Vec<Pregunta> = leer(&archivo)?;
    if let Some(a) = apartados.iter().find(|a| !examen.pdfs.values().any(|t| t.apartados.contains(a))) {
        return Err(format!("el apartado {a} no está en examen.json"));
    }
    let historial = carpeta.join("respuestas.jsonl");
    let resto: Vec<Pregunta> = banco.iter().filter(|p| !apartados.contains(&p.apartado)).cloned().collect();
    let del_lote = huerfanas(&resto, &historial)?.len() - huerfanas(&banco, &historial)?.len();
    if del_lote > 0 {
        return Err(format!("respuestas.jsonl tiene {del_lote} id de los apartados del lote: el dueño decide antes qué pasa con ellas"));
    }
    if carpeta.join("ronda.json").exists() {
        return Err("ronda.json existe: una ronda a medias usa los id del banco".into());
    }
    let trabajo = carpeta.join("trabajo");
    if let Some(a) = apartados.iter().find(|a| trabajo.join(a).exists()) {
        return Err(format!("{} ya existe", trabajo.join(a).display()));
    }
    examen.sin_pregunta.retain(|pdf, _| !examen.pdfs[pdf].apartados.iter().any(|a| apartados.contains(a)));
    escribir(&archivo, &resto)?;
    escribir(&carpeta.join("examen.json"), &examen)?;
    for a in apartados {
        let dir = trabajo.join(a);
        fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        escribir(&dir.join("examen.json"), &examen)?;
        escribir(&dir.join("preguntas.json"), &Vec::<Pregunta>::new())?;
        std::os::unix::fs::symlink("../../pdfs", dir.join("pdfs")).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    Ok(())
}

/// Une al banco las preguntas y el `sin_pregunta` de la carpeta de trabajo de cada apartado del lote. Falla sin
/// cambiar nada con una pregunta de otro apartado o con un `id` que ya está en el banco. Devuelve las preguntas
/// añadidas.
pub fn montar(carpeta: &Path, apartados: &[String]) -> Result<usize, String> {
    let mut examen: Examen = leer(&carpeta.join("examen.json"))?;
    let archivo = carpeta.join("preguntas.json");
    let mut banco: Vec<Pregunta> = leer(&archivo)?;
    let antes = banco.len();
    for a in apartados {
        let dir = carpeta.join("trabajo").join(a);
        let nuevas: Vec<Pregunta> = leer(&dir.join("preguntas.json"))?;
        if let Some(p) = nuevas.iter().find(|p| &p.apartado != a || banco.iter().any(|b| b.id == p.id)) {
            return Err(format!("{}: {} no es del apartado {a}, o ya está en el banco", dir.display(), p.id));
        }
        let copia: Examen = leer(&dir.join("examen.json"))?;
        for (pdf, t) in &examen.pdfs {
            if let (true, Some(razones)) = (t.apartados.contains(a), copia.sin_pregunta.get(pdf)) {
                examen.sin_pregunta.insert(pdf.clone(), razones.clone());
            }
        }
        banco.extend(nuevas);
    }
    escribir(&archivo, &banco)?;
    escribir(&carpeta.join("examen.json"), &examen)?;
    Ok(banco.len() - antes)
}

/// Una pregunta de un bloque del borrador, sin `id` ni `apartado`. `BANCO.md` describe el formato del bloque.
fn bloque(encabezado: &str, lineas: &[&str], pdf: &str) -> Result<Pregunta, String> {
    let error = |e: &str| format!("{encabezado}: {e}");
    let (concepto, pagina) = encabezado
        .split_once(" · p")
        .and_then(|(c, n)| Some((c, n.trim().parse().ok()?)))
        .ok_or_else(|| error("el encabezado no es «## concepto · pN»"))?;
    let (mut pregunta, mut correcta, mut verdad) = (Vec::new(), Vec::new(), Vec::new());
    let (mut distractores, mut explicacion) = (Vec::new(), Vec::new());
    for l in lineas {
        if let Some(c) = l.strip_prefix("+ ") {
            correcta.push(c.trim());
        } else if l.starts_with('✓') {
            verdad.push(l.trim().to_string());
        } else if let Some(d) = l.strip_prefix("- ") {
            let [texto, matiz] = d.split('|').collect::<Vec<_>>()[..] else {
                return Err(error(&format!("el distractor no tiene un solo «|»: {d}")));
            };
            let marcas = texto.split_once('«').and_then(|(a, r)| Some((a, r.split_once('»')?)));
            let Some((antes, (clave, despues))) = marcas.filter(|(_, (c, _))| !c.trim().is_empty()) else {
                return Err(error(&format!("el distractor no marca su clave con «»: {d}")));
            };
            distractores.push(format!("{antes}{clave}{despues}").trim().to_string());
            explicacion.push(format!("✗ «{}»: {}", clave.trim(), matiz.trim()));
        } else {
            pregunta.push(l.trim());
        }
    }
    let ([pregunta], [correcta], [verdad]) = (&pregunta[..], &correcta[..], &verdad[..]) else {
        return Err(error("el bloque no tiene una línea ✓, una línea + y una línea de enunciado"));
    };
    explicacion.insert(0, verdad.clone());
    Ok(Pregunta {
        id: String::new(),
        concepto: concepto.trim().into(),
        apartado: String::new(),
        fuente: nucleo::Fuente { pdf: pdf.into(), pagina },
        pregunta: pregunta.to_string(),
        correcta: correcta.to_string(),
        distractores,
        explicacion: explicacion.join("\n"),
    })
}

/// Añade al final de `preguntas.json` las preguntas del borrador, cada una con el siguiente `id` libre del apartado.
/// Un concepto que ya está en el banco da un error, y el banco queda igual. Devuelve los `id` nuevos.
pub fn importar(carpeta: &Path, apartado: &str, borrador: &Path) -> Result<Vec<String>, String> {
    let examen: Examen = leer(&carpeta.join("examen.json"))?;
    let archivo = carpeta.join("preguntas.json");
    let mut banco: Vec<Pregunta> = leer(&archivo)?;
    let texto = fs::read_to_string(borrador).map_err(|e| format!("{}: {e}", borrador.display()))?;
    let pdfs: Vec<&String> =
        examen.pdfs.iter().filter(|(_, t)| t.apartados.iter().any(|a| a == apartado)).map(|(p, _)| p).collect();
    let [pdf] = pdfs[..] else {
        return Err(format!("el apartado {apartado} tiene {} PDF en examen.json, y no 1", pdfs.len()));
    };
    let mut bloques: Vec<(&str, Vec<&str>)> = Vec::new();
    for l in texto.lines().filter(|l| !l.trim().is_empty()) {
        match (l.strip_prefix("## "), bloques.last_mut()) {
            (Some(encabezado), _) => bloques.push((encabezado.trim(), Vec::new())),
            (None, Some((_, lineas))) => lineas.push(l),
            (None, None) => return Err(format!("una línea antes del primer «## »: {l}")),
        }
    }
    if bloques.is_empty() {
        return Err(format!("el borrador no tiene bloques: {}", borrador.display()));
    }
    let siguiente =
        |prefijo: String| banco.iter().filter_map(|p| p.id.strip_prefix(&prefijo)?.parse::<u32>().ok()).max().unwrap_or(0);
    let (mut normal, mut de_sigla) = (siguiente(format!("{apartado}-")), siguiente(format!("{apartado}-s")));
    let mut nuevas = Vec::new();
    for (encabezado, lineas) in &bloques {
        let mut p = bloque(encabezado, lineas, pdf)?;
        if banco.iter().any(|b| b.concepto == p.concepto) {
            return Err(format!("el concepto {} ya está en el banco", p.concepto));
        }
        p.id = if sigla(&p.concepto) {
            de_sigla += 1;
            format!("{apartado}-s{de_sigla:02}")
        } else {
            normal += 1;
            format!("{apartado}-{normal:03}")
        };
        p.apartado = apartado.into();
        nuevas.push(p);
    }
    let ids = nuevas.iter().map(|p| p.id.clone()).collect();
    banco.extend(nuevas);
    escribir(&archivo, &banco)?;
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nucleo::{Fuente, Tema};

    /// Un PDF mínimo con `n` páginas, que poppler lee.
    fn pdf(archivo: &Path, n: u32) {
        let kids: Vec<String> = (0..n).map(|i| format!("{} 0 R", i + 3)).collect();
        let mut t = format!(
            "%PDF-1.4\n1 0 obj <</Type/Catalog/Pages 2 0 R>> endobj\n2 0 obj <</Type/Pages/Kids[{}]/Count {n}>> endobj\n",
            kids.join(" ")
        );
        for i in 0..n {
            t += &format!("{} 0 obj <</Type/Page/Parent 2 0 R/MediaBox[0 0 100 100]>> endobj\n", i + 3);
        }
        t += "trailer <</Root 1 0 R>>\n%%EOF\n";
        fs::write(archivo, t).unwrap();
    }

    /// Una carpeta de examen con `1_Legislacion.pdf` de 5 páginas, para los apartados `rgpd` y `lssi`.
    fn carpeta() -> (tempfile::TempDir, Examen) {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join("pdfs")).unwrap();
        pdf(&dir.path().join("pdfs/1_Legislacion.pdf"), 5);
        let examen = Examen {
            titulo: "LN".into(),
            preguntas: 20,
            opciones: 4,
            minutos: 25,
            penalizacion: 0.25,
            pdfs: BTreeMap::from([
                ("1_Legislacion.pdf".into(), tema("Legislación", &["rgpd", "lssi"])),
                ("2_Teoria.pdf".into(), tema("Teoría", &["ens"])),
            ]),
            apartados: ["rgpd", "lssi", "ens"].map(|a| (a.into(), a.to_uppercase())).into(),
            sin_pregunta: BTreeMap::new(),
        };
        (dir, examen)
    }

    fn tema(nombre: &str, apartados: &[&str]) -> Tema {
        Tema { nombre: nombre.into(), apartados: apartados.iter().map(|a| a.to_string()).collect() }
    }

    fn pregunta(id: &str) -> Pregunta {
        Pregunta {
            id: id.into(),
            concepto: "rgpd-brecha-plazo".into(),
            apartado: "rgpd".into(),
            pregunta: "¿En qué plazo notifica el responsable una brecha?".into(),
            correcta: "72 horas".into(),
            distractores: vec!["24 horas".into(), "Un mes".into(), "48 horas".into()],
            explicacion: "Art. 33 del RGPD.".into(),
            fuente: Fuente { pdf: "1_Legislacion.pdf".into(), pagina: 5 },
        }
    }

    /// La pregunta cambiada, con otras 2 del mismo concepto para que el concepto tenga 3.
    fn con_pareja(p: Pregunta) -> Vec<Pregunta> {
        vec![p, pregunta("rgpd-002"), pregunta("rgpd-003")]
    }

    fn un_error(preguntas: &[Pregunta], texto: &str) {
        let (dir, examen) = carpeta();
        let errores = comprobar(preguntas, &examen, dir.path());
        assert_eq!(errores.len(), 1, "{errores:?}");
        assert!(errores[0].contains(texto), "{errores:?}");
    }

    #[test]
    fn una_pregunta_valida_no_da_errores() {
        let (dir, examen) = carpeta();
        assert_eq!(comprobar(&con_pareja(pregunta("rgpd-001")), &examen, dir.path()), Vec::<String>::new());
    }

    #[test]
    fn un_id_repetido_falla() {
        un_error(&[pregunta("rgpd-001"), pregunta("rgpd-001"), pregunta("rgpd-002")], "rgpd-001: id repetido");
    }

    #[test]
    fn un_apartado_que_no_es_de_su_pdf_falla() {
        un_error(&con_pareja(Pregunta { apartado: "ens".into(), ..pregunta("rgpd-001") }), "el apartado ens");
    }

    #[test]
    fn un_apartado_sin_nombre_falla() {
        let (dir, mut examen) = carpeta();
        examen.apartados.remove("ens");
        let errores = comprobar(&con_pareja(pregunta("rgpd-001")), &examen, dir.path());
        assert_eq!(errores, ["2_Teoria.pdf: el apartado ens no tiene nombre en apartados de examen.json"]);
    }

    #[test]
    fn un_numero_de_distractores_distinto_de_opciones_menos_1_falla() {
        for n in [2, 4] {
            let distractores = (0..n).map(|i| format!("{i} días")).collect();
            un_error(&con_pareja(Pregunta { distractores, ..pregunta("rgpd-001") }), &format!("{n} distractores"));
        }
    }

    #[test]
    fn un_distractor_igual_a_la_correcta_falla() {
        let distractores = vec!["24 horas".into(), "72 horas".into(), "Un mes".into()];
        un_error(&con_pareja(Pregunta { distractores, ..pregunta("rgpd-001") }), "correcta: 72 horas");
    }

    #[test]
    fn un_pdf_fuera_de_examen_json_o_de_pdfs_falla() {
        for nombre in ["4_Teoria.pdf", "2_Teoria.pdf"] {
            let fuente = Fuente { pdf: nombre.into(), pagina: 1 };
            un_error(&con_pareja(Pregunta { fuente, ..pregunta("rgpd-001") }), nombre);
        }
    }

    #[test]
    fn una_pagina_fuera_del_pdf_falla() {
        for pagina in [0, 6] {
            let fuente = Fuente { pdf: "1_Legislacion.pdf".into(), pagina };
            un_error(&con_pareja(Pregunta { fuente, ..pregunta("rgpd-001") }), &format!("la página {pagina}"));
        }
    }

    #[test]
    fn un_concepto_con_2_o_con_4_preguntas_falla() {
        let (dir, examen) = carpeta();
        let tres = vec![pregunta("rgpd-001"), pregunta("rgpd-002"), pregunta("rgpd-003")];
        assert!(comprobar(&tres, &examen, dir.path()).is_empty());
        un_error(&tres[..2], "rgpd-brecha-plazo: 2 preguntas, y no 3");
        un_error(&[tres, vec![pregunta("rgpd-004")]].concat(), "rgpd-brecha-plazo: 4 preguntas");
    }

    #[test]
    fn un_concepto_de_sigla_tiene_1_pregunta() {
        let (dir, examen) = carpeta();
        let sigla = |id: &str| Pregunta { concepto: "sigla-rgpd".into(), ..pregunta(id) };
        assert!(comprobar(&[sigla("rgpd-001")], &examen, dir.path()).is_empty());
        un_error(&[sigla("rgpd-001"), sigla("rgpd-002")], "sigla-rgpd: 2 preguntas, y no 1");
    }

    #[test]
    fn una_clave_que_no_esta_en_su_distractor_falla() {
        let (dir, examen) = carpeta();
        let con = |explicacion: &str| con_pareja(Pregunta { explicacion: explicacion.into(), ..pregunta("rgpd-001") });
        assert!(comprobar(&con("✓ x\n✗ «24 HORAS»: a\n✗ «un mes»: b"), &examen, dir.path()).is_empty());
        un_error(&con("✓ x\n✗ «24 horas»: a\n✗ «48 horas»: b"), "rgpd-001: la línea ✗ 2");
        un_error(&con("✓ x\n✗ 24 horas: a"), "rgpd-001: la línea ✗ 1");
        un_error(&con("✓ x\n✗ «»: a"), "rgpd-001: la línea ✗ 1");
        un_error(&con("✓ x\n✗ «24»: a\n✗ «Un»: b\n✗ «48»: c\n✗ «9»: d"), "rgpd-001: 4 líneas ✗, y 3 distractores");
    }

    #[test]
    fn una_linea_corta_o_un_matiz_repetido_en_el_apartado_es_un_aviso() {
        let con = |id: &str, apartado: &str, e: &str| Pregunta { apartado: apartado.into(), explicacion: e.into(), ..pregunta(id) };
        let buena = "✓ El responsable notifica la brecha en 72 horas desde que la conoce.\n✗ «24 horas»: el plazo es de 72";
        assert!(explicaciones(&[con("a", "rgpd", buena), con("b", "lssi", buena)]).is_empty());
        assert_eq!(explicaciones(&[con("a", "rgpd", "✓ Son 72 horas.")]), ["a: la línea ✓ tiene 3 palabras, menos de 8"]);
        assert_eq!(
            explicaciones(&[con("a", "rgpd", buena), con("b", "rgpd", buena)]),
            ["b: la línea ✗ repite el matiz de a: el plazo es de 72"]
        );
    }

    #[test]
    fn una_pagina_sin_pregunta_ni_razon_o_con_las_dos_es_un_aviso() {
        let (dir, mut examen) = carpeta();
        let razones = BTreeMap::from([("1".into(), "portada".into()), ("2".into(), "índice".into()), ("5".into(), "repetida".into())]);
        examen.sin_pregunta = BTreeMap::from([("1_Legislacion.pdf".into(), razones)]);
        let en = |id: &str, pagina| Pregunta { fuente: Fuente { pdf: "1_Legislacion.pdf".into(), pagina }, ..pregunta(id) };
        assert_eq!(
            paginas_sin_pregunta(&[en("a", 3), en("b", 5)], &examen, dir.path()),
            [
                "1_Legislacion.pdf: páginas sin pregunta y sin razón en sin_pregunta: 4",
                "1_Legislacion.pdf: páginas con pregunta y con razón en sin_pregunta: 5"
            ]
        );
    }

    const BORRADOR: &str = "\
## rgpd-brecha-plazo · p5
¿En qué plazo notifica el responsable una brecha?
+ 72 horas
✓ El RGPD da 72 horas desde que el responsable conoce la brecha.
- «24 horas» desde la brecha | es el plazo de otra norma
- «Un mes» | es el plazo de respuesta al interesado
- «48 horas» | no es un plazo del RGPD

## sigla-dpd · p2
¿Qué significa DPD?
+ Delegado de Protección de Datos
✓ El DPD es la persona que supervisa el cumplimiento del RGPD.
- «Director» de Protección de Datos | x
- «Defensor» del Público Digital | y
- «Delegado de Política» Digital | z
";

    /// La carpeta con `examen.json`, un banco con `rgpd-007` y `rgpd-s02`, y el borrador dado.
    fn para_importar(borrador: &str) -> tempfile::TempDir {
        let (dir, mut examen) = carpeta();
        examen.pdfs.insert("3_Otro.pdf".into(), tema("Otro", &["lssi"]));
        fs::write(dir.path().join("examen.json"), serde_json::to_string(&examen).unwrap()).unwrap();
        let banco = [pregunta("rgpd-007"), Pregunta { concepto: "sigla-aepd".into(), ..pregunta("rgpd-s02") }];
        fs::write(dir.path().join("preguntas.json"), serde_json::to_string(&banco).unwrap()).unwrap();
        fs::write(dir.path().join("borrador.md"), borrador).unwrap();
        dir
    }

    #[test]
    fn importar_anade_cada_bloque_con_el_siguiente_id_libre() {
        let dir = para_importar(&BORRADOR.replace("rgpd-brecha-plazo", "rgpd-brecha-nuevo"));
        let ids = importar(dir.path(), "rgpd", &dir.path().join("borrador.md")).unwrap();
        assert_eq!(ids, ["rgpd-008", "rgpd-s03"]);
        let banco: Vec<serde_json::Value> = leer(&dir.path().join("preguntas.json")).unwrap();
        assert_eq!(banco.len(), 4);
        assert_eq!(
            banco[2],
            serde_json::json!({
                "id": "rgpd-008",
                "concepto": "rgpd-brecha-nuevo",
                "apartado": "rgpd",
                "fuente": { "pdf": "1_Legislacion.pdf", "pagina": 5 },
                "pregunta": "¿En qué plazo notifica el responsable una brecha?",
                "correcta": "72 horas",
                "distractores": ["24 horas desde la brecha", "Un mes", "48 horas"],
                "explicacion": "✓ El RGPD da 72 horas desde que el responsable conoce la brecha.\n✗ «24 horas»: es el plazo de otra norma\n✗ «Un mes»: es el plazo de respuesta al interesado\n✗ «48 horas»: no es un plazo del RGPD"
            })
        );
    }

    #[test]
    fn importar_falla_sin_tocar_el_banco() {
        let nuevo = BORRADOR.replace("rgpd-brecha-plazo", "rgpd-brecha-nuevo");
        let casos = [
            (BORRADOR.to_string(), "rgpd", "el concepto rgpd-brecha-plazo ya está en el banco"),
            (nuevo.replace("- «Un mes» |", "- Un mes |"), "rgpd", "rgpd-brecha-nuevo · p5: el distractor no marca"),
            (nuevo.replace("- «Un mes» |", "- «Un mes»"), "rgpd", "rgpd-brecha-nuevo · p5: el distractor no tiene un"),
            (nuevo.replace("| x", "| x | y"), "rgpd", "sigla-dpd · p2: el distractor no tiene un"),
            (nuevo.replace("«Director»", "«»Director"), "rgpd", "sigla-dpd · p2: el distractor no marca"),
            ("\n".into(), "rgpd", "el borrador no tiene bloques"),
            (nuevo.replace("✓ El DPD", "El DPD"), "rgpd", "sigla-dpd · p2: el bloque no tiene una línea ✓"),
            (nuevo.replace(" · p5", ""), "rgpd", "rgpd-brecha-nuevo: el encabezado no es"),
            (nuevo.clone(), "lssi", "el apartado lssi tiene 2 PDF en examen.json, y no 1"),
            (nuevo.clone(), "nada", "el apartado nada tiene 0 PDF"),
        ];
        for (borrador, apartado, error) in casos {
            let dir = para_importar(&borrador);
            let antes = fs::read_to_string(dir.path().join("preguntas.json")).unwrap();
            let e = importar(dir.path(), apartado, &dir.path().join("borrador.md")).unwrap_err();
            assert!(e.contains(error), "{e}");
            assert_eq!(fs::read_to_string(dir.path().join("preguntas.json")).unwrap(), antes);
        }
    }

    #[test]
    fn un_id_del_historial_fuera_del_banco_es_huerfano() {
        let (dir, _) = carpeta();
        let historial = dir.path().join("respuestas.jsonl");
        assert_eq!(huerfanas(&[pregunta("rgpd-001")], &historial).unwrap(), Vec::<String>::new());
        let lineas: String = ["rgpd-001", "viejo-1", "viejo-1"]
            .map(|id| format!("{{\"id\":\"{id}\",\"ts\":\"2026-11-18T10:00:00Z\",\"resultado\":\"Acierto\",\"seguro\":true}}\n"))
            .concat();
        fs::write(&historial, lineas).unwrap();
        assert_eq!(huerfanas(&[pregunta("rgpd-001")], &historial).unwrap(), vec!["viejo-1"]);
    }

    #[test]
    fn un_absoluto_solo_en_un_distractor_es_una_pista() {
        let con = |correcta: &str, d: &str| Pregunta {
            correcta: correcta.into(),
            distractores: vec![d.into(), "Un mes".into(), "48 horas".into()],
            ..pregunta("rgpd-001")
        };
        let avisos = pistas(&[con("72 horas", "Siempre 24 h")]);
        assert_eq!(avisos.len(), 1);
        assert!(avisos[0].contains("siempre"), "{avisos:?}");
        assert!(pistas(&[con("72 horas", "Sólo 24 h")])[0].contains("sólo"));
        assert!(pistas(&[con("72 horas", "Un solo día")]).is_empty());
        assert!(pistas(&[con("Nunca 72 h", "Nunca 24 horas")]).is_empty());
    }

    #[test]
    fn la_correcta_mas_larga_en_mas_de_1_de_cada_3_es_una_pista() {
        let larga = Pregunta { correcta: "72 horas desde que lo sabe".into(), ..pregunta("rgpd-001") };
        let sigla = Pregunta { concepto: "sigla-rgpd".into(), ..larga.clone() };
        assert!(pistas(&[larga.clone(), pregunta("a"), pregunta("b"), sigla]).is_empty());
        assert_eq!(pistas(&[larga.clone(), larga, pregunta("a")]).len(), 1);
    }

    #[test]
    fn sacar_da_una_imagen_por_pagina_y_deja_las_que_ya_existen() {
        let (dir, _) = carpeta();
        pdf(&dir.path().join("pdfs/2 _Teoría.pdf"), 2);
        let paginas = dir.path().join("paginas");
        fs::create_dir(&paginas).unwrap();
        fs::write(paginas.join("1_Legislacion-1.webp"), "ya estaba").unwrap();
        sacar(dir.path()).unwrap();
        let mut nombres: Vec<String> =
            fs::read_dir(&paginas).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
        nombres.sort();
        assert_eq!(
            nombres,
            [
                "1_Legislacion-1.webp",
                "1_Legislacion-2.webp",
                "1_Legislacion-3.webp",
                "1_Legislacion-4.webp",
                "1_Legislacion-5.webp",
                "2 _Teoría-1.webp",
                "2 _Teoría-2.webp"
            ]
        );
        assert_eq!(fs::read_to_string(paginas.join("1_Legislacion-1.webp")).unwrap(), "ya estaba");
        // 100 puntos a 200 ppp son 278 píxeles: el ancho va en los bytes 26 y 27 del WebP con pérdida.
        let webp = fs::read(paginas.join("1_Legislacion-2.webp")).unwrap();
        assert_eq!(u16::from_le_bytes(webp[26..28].try_into().unwrap()) & 0x3fff, 278);
    }

    #[test]
    fn aplicar_cambia_los_campos_del_arreglo_y_da_el_resto() {
        let mut banco = vec![pregunta("rgpd-001")];
        let fallos: Vec<Fallo> = serde_json::from_str(
            r#"[
                {"id": "rgpd-001", "regla": 3, "gravedad": "grave", "fallo": "x", "arreglo": {"correcta": "3 días"}},
                {"id": "rgpd-001", "regla": 5, "gravedad": "leve", "fallo": "y", "arreglo": null},
                {"id": "rgpd-001", "regla": 0, "gravedad": "leve", "fallo": "z", "arreglo": {"id": "rgpd-009"}},
                {"id": "viejo-1", "regla": 3, "gravedad": "grave", "fallo": "w", "arreglo": {"correcta": "1"}},
                {"id": "rgpd-001", "regla": 4, "gravedad": "leve", "fallo": "v", "arreglo": {}},
                {"id": null, "regla": 1, "gravedad": "leve", "fallo": "u", "arreglo": null}
            ]"#,
        )
        .unwrap();
        let pendientes: Vec<String> = aplicar(&mut banco, fallos).into_iter().map(|f| f.fallo).collect();
        assert_eq!(pendientes, ["y", "z", "w", "v", "u"]);
        assert_eq!(banco[0].correcta, "3 días");
        assert_eq!(banco[0].id, "rgpd-001");
    }

    #[test]
    fn preparar_vacia_el_lote_y_montar_lo_devuelve() {
        let (dir, mut examen) = carpeta();
        let c = dir.path();
        examen.sin_pregunta.insert("1_Legislacion.pdf".into(), BTreeMap::from([("1".into(), "portada".into())]));
        escribir(&c.join("examen.json"), &examen).unwrap();
        let mut otra = pregunta("e1");
        otra.apartado = "ens".into();
        escribir(&c.join("preguntas.json"), &vec![pregunta("r1"), otra]).unwrap();
        let apartados = vec!["rgpd".to_string()];
        fs::write(c.join("respuestas.jsonl"), "{\"id\":\"r1\",\"ts\":1,\"resultado\":\"Acierto\",\"seguro\":true}\n").unwrap();
        assert!(preparar(c, &apartados).unwrap_err().contains("respuestas.jsonl"));
        fs::remove_file(c.join("respuestas.jsonl")).unwrap();

        preparar(c, &apartados).unwrap();
        let banco: Vec<Pregunta> = leer(&c.join("preguntas.json")).unwrap();
        assert_eq!(banco.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), ["e1"]);
        assert!(leer::<Examen>(&c.join("examen.json")).unwrap().sin_pregunta.is_empty());
        let t = c.join("trabajo/rgpd");
        assert!(t.join("pdfs/1_Legislacion.pdf").exists());
        assert!(preparar(c, &apartados).unwrap_err().contains("ya existe"));

        escribir(&t.join("preguntas.json"), &vec![pregunta("r2")]).unwrap();
        escribir(&t.join("examen.json"), &examen).unwrap();
        assert_eq!(montar(c, &apartados).unwrap(), 1);
        assert_eq!(leer::<Vec<Pregunta>>(&c.join("preguntas.json")).unwrap().len(), 2);
        assert_eq!(leer::<Examen>(&c.join("examen.json")).unwrap().sin_pregunta, examen.sin_pregunta);
        assert!(montar(c, &apartados).unwrap_err().contains("r2"));
    }
}
