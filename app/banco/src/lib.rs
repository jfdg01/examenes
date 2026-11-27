//! Las herramientas del banco de un examen: la comprobación de sus archivos, el texto y las imágenes de las
//! páginas de los PDFs, y los pasos de un lote: el borrador, la carpeta de trabajo de cada apartado y los arreglos
//! de una revisión. Todas leen la carpeta del examen. La comprobación y las imágenes llaman a poppler.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use nucleo::{Concepto, Examen, Forma, Fuente, Pregunta, PreguntaDeDesarrollo};
use serde::Deserialize;

fn leer<T: serde::de::DeserializeOwned>(archivo: &Path) -> Result<T, String> {
    let texto = fs::read_to_string(archivo).map_err(|e| format!("{}: {e}", archivo.display()))?;
    serde_json::from_str(&texto).map_err(|e| format!("{}: {e}", archivo.display()))
}

fn leer_si_existe<T: serde::de::DeserializeOwned>(archivo: &Path) -> Result<Option<T>, String> {
    if archivo.exists() { leer(archivo).map(Some) } else { Ok(None) }
}

fn escribir<T: serde::Serialize>(archivo: &Path, datos: &T) -> Result<(), String> {
    let texto = serde_json::to_string_pretty(datos).map_err(|e| e.to_string())? + "\n";
    fs::write(archivo, texto).map_err(|e| format!("{}: {e}", archivo.display()))
}

/// El tipo de un lote: lo que escriben sus creadores. Cada tipo vive en su archivo del examen.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tipo {
    Conceptos,
    Test,
    Desarrollo,
}

impl std::str::FromStr for Tipo {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "conceptos" => Ok(Tipo::Conceptos),
            "test" => Ok(Tipo::Test),
            "desarrollo" => Ok(Tipo::Desarrollo),
            _ => Err(format!("el tipo {s} no es conceptos, test ni desarrollo")),
        }
    }
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

/// Los errores de la `fuente` de un concepto: un PDF que no existe, un apartado que no es de su PDF
/// y una página fuera del PDF.
fn comprobar_fuente(
    id: &str,
    apartado: &str,
    fuente: &Fuente,
    examen: &Examen,
    carpeta: &Path,
    paginas_de: &mut HashMap<String, Result<u32, String>>,
    errores: &mut Vec<String>,
) {
    let nombre = fuente.pdf.as_str();
    let pdf = carpeta.join("pdfs").join(nombre);
    let Some(tema) = examen.pdfs.get(nombre).filter(|_| pdf.is_file()) else {
        errores.push(format!("{id}: el pdf no está en examen.json o en pdfs/: {nombre}"));
        return;
    };
    if !tema.apartados.iter().any(|a| a == apartado) {
        errores.push(format!("{id}: el apartado {apartado} no es un apartado de {nombre} en examen.json"));
    }
    match paginas_de.entry(nombre.to_string()).or_insert_with(|| paginas(&pdf)) {
        Ok(n) if fuente.pagina == 0 || fuente.pagina > *n => {
            errores.push(format!("{id}: la página {} no está entre las {n} de {nombre}", fuente.pagina))
        }
        Ok(_) => {}
        Err(e) => errores.push(format!("{id}: {e}")),
    }
}

/// Los errores del banco de test y de sus conceptos. Sin `conceptos.json`, `conceptos` es `None`: solo un examen
/// congelado vive sin él, y sus preguntas no tienen página.
pub fn comprobar(preguntas: &[Pregunta], conceptos: Option<&[Concepto]>, examen: &Examen, carpeta: &Path) -> Vec<String> {
    let distractores = (examen.opciones as usize).saturating_sub(1);
    let mut errores = Vec::new();
    if conceptos.is_none() && !examen.congelado {
        errores.push("falta conceptos.json, y el examen no está congelado".into());
    }
    let mut paginas_de = HashMap::new();
    let mut vistos = HashSet::new();
    for c in conceptos.unwrap_or_default() {
        if !vistos.insert(&c.id) {
            errores.push(format!("{}: id repetido", c.id));
        }
        comprobar_fuente(&c.id, &c.apartado, &c.fuente, examen, carpeta, &mut paginas_de, &mut errores);
    }
    let concepto_de: HashMap<&str, &Concepto> = conceptos.unwrap_or_default().iter().map(|c| (c.id.as_str(), c)).collect();
    let mut vistos = HashSet::new();
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
        let mut claves = HashSet::new();
        for (i, (linea, d)) in lineas.into_iter().zip(&p.distractores).enumerate() {
            let c = clave(linea).map(|(c, _)| c.to_lowercase());
            if !c.as_ref().is_some_and(|c| d.to_lowercase().contains(c)) {
                errores.push(format!("{id}: la línea ✗ {} no lleva una clave «…» de su distractor: {d}", i + 1));
            } else if !claves.insert(c) {
                errores.push(format!("{id}: la línea ✗ {} repite la clave de otra línea ✗", i + 1));
            }
        }
        match (conceptos, concepto_de.get(p.concepto.as_str())) {
            (Some(_), None) => errores.push(format!("{id}: el concepto {} no está en conceptos.json", p.concepto)),
            (Some(_), Some(c)) if c.apartado != p.apartado => {
                errores.push(format!("{id}: el apartado {} no es el de su concepto {}, {}", p.apartado, c.id, c.apartado))
            }
            (None, _) if !examen.pdfs.values().any(|t| t.apartados.contains(&p.apartado)) => {
                errores.push(format!("{id}: el apartado {} no está en ningún tema de examen.json", p.apartado))
            }
            _ => {}
        }
    }
    for (pdf, tema) in &examen.pdfs {
        for a in tema.apartados.iter().filter(|a| !examen.apartados.contains_key(*a)) {
            errores.push(format!("{pdf}: el apartado {a} no tiene nombre en apartados de examen.json"));
        }
    }
    if !examen.congelado {
        errores.extend(cuentas(preguntas, conceptos));
    }
    errores
}

/// Cada concepto con un número de preguntas distinto de 3, o de 1 si es de sigla. Con `conceptos.json`, cuenta
/// también los conceptos sin preguntas de un apartado con preguntas de test: un apartado sin ellas espera su lote de test.
pub fn cuentas(preguntas: &[Pregunta], conceptos: Option<&[Concepto]>) -> Vec<String> {
    let con_test: HashSet<&str> = preguntas.iter().map(|p| p.apartado.as_str()).collect();
    let mut cuenta: Vec<(&str, usize)> = Vec::new();
    for c in conceptos.unwrap_or_default().iter().filter(|c| con_test.contains(c.apartado.as_str())) {
        if !cuenta.iter().any(|(id, _)| *id == c.id) {
            cuenta.push((&c.id, 0));
        }
    }
    for p in preguntas {
        match cuenta.iter_mut().find(|(c, _)| *c == p.concepto) {
            Some((_, n)) => *n += 1,
            None => cuenta.push((&p.concepto, 1)),
        }
    }
    let mut fallos = Vec::new();
    for (concepto, n) in cuenta {
        let esperado = if sigla(concepto) { 1 } else { 3 };
        if n != esperado {
            let s = if n == 1 { "" } else { "s" };
            fallos.push(format!("{concepto}: {n} pregunta{s}, y no {esperado}"));
        }
    }
    fallos
}

/// Los errores de `desarrollo.json`: un `id` repetido o del test, un punto que no es un concepto de su apartado,
/// un número de puntos fuera de 4 a 8, y un concepto en 2 preguntas que no son de la forma comparación.
pub fn comprobar_desarrollo(desarrollo: &[PreguntaDeDesarrollo], preguntas: &[Pregunta], conceptos: &[Concepto]) -> Vec<String> {
    let del_test: HashSet<&str> = preguntas.iter().map(|p| p.id.as_str()).collect();
    let apartado_de: HashMap<&str, &str> = conceptos.iter().map(|c| (c.id.as_str(), c.apartado.as_str())).collect();
    let mut errores = Vec::new();
    let mut vistos = HashSet::new();
    let mut usos: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for d in desarrollo {
        let id = &d.id;
        if !vistos.insert(id) {
            errores.push(format!("{id}: id repetido"));
        }
        if del_test.contains(id.as_str()) {
            errores.push(format!("{id}: id repetido en preguntas.json"));
        }
        if !(4..=8).contains(&d.puntos.len()) {
            errores.push(format!("{id}: {} puntos, y no de 4 a 8", d.puntos.len()));
        }
        for punto in &d.puntos {
            if apartado_de.get(punto.as_str()) != Some(&d.apartado.as_str()) {
                errores.push(format!("{id}: el punto {punto} no es un concepto de {}", d.apartado));
            }
            if d.forma != Forma::Comparacion {
                usos.entry(punto).or_default().push(id);
            }
        }
    }
    for (concepto, ids) in usos.into_iter().filter(|(_, ids)| ids.len() > 1) {
        errores.push(format!("{concepto}: punto de {}, que no son de la forma comparación", ids.join(" y de ")));
    }
    errores
}

/// Las frases de un texto: cada `.`, `?` o `!` seguido de un espacio o del final, y un final sin ellos.
fn cuantas_frases(texto: &str) -> usize {
    let c: Vec<char> = texto.trim().chars().collect();
    let fin = |i: usize| matches!(c[i], '.' | '?' | '!') && c.get(i + 1).is_none_or(|s| s.is_whitespace());
    let cortes = (0..c.len()).filter(|&i| fin(i)).count();
    cortes + usize::from(!c.is_empty() && !fin(c.len() - 1))
}

/// Los avisos de una respuesta modelo de menos de 4 frases o de más de 8.
pub fn frases(desarrollo: &[PreguntaDeDesarrollo]) -> Vec<String> {
    desarrollo
        .iter()
        .map(|d| (d, cuantas_frases(&d.modelo)))
        .filter(|(_, n)| !(4..=8).contains(n))
        .map(|(d, n)| format!("{}: la respuesta modelo tiene {n} frases, y no de 4 a 8", d.id))
        .collect()
}

/// Los `id` de `respuestas.jsonl` que no están en el banco, sin repetir y en orden.
pub fn huerfanas(preguntas: &[Pregunta], historial: &Path) -> Result<Vec<String>, String> {
    let ids: HashSet<&str> = preguntas.iter().map(|p| p.id.as_str()).collect();
    Ok(ids_del_historial(historial)?.into_iter().filter(|id| !ids.contains(id.as_str())).collect())
}

/// Los `id` de las líneas de un historial en JSONL, sin repetir y en orden. Un historial que no existe no tiene.
fn ids_del_historial(historial: &Path) -> Result<BTreeSet<String>, String> {
    #[derive(Deserialize)]
    struct Linea {
        id: String,
    }
    let texto = match fs::read_to_string(historial) {
        Ok(texto) => texto,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeSet::new()),
        Err(e) => return Err(format!("{}: {e}", historial.display())),
    };
    let mut ids = BTreeSet::new();
    for linea in texto.lines().filter(|l| !l.trim().is_empty()) {
        let r: Linea = serde_json::from_str(linea).map_err(|e| format!("{}: {e}", historial.display()))?;
        ids.insert(r.id);
    }
    Ok(ids)
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

/// Los avisos de las pistas de la regla 5 de `docs/TEST.md`, que un revisor juzga.
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
/// Una página con pregunta es la `fuente` de un concepto.
pub fn paginas_sin_pregunta(conceptos: &[Concepto], examen: &Examen, carpeta: &Path) -> Vec<String> {
    let mut citadas: BTreeMap<&str, BTreeSet<u32>> = BTreeMap::new();
    for f in conceptos.iter().map(|c| &c.fuente) {
        citadas.entry(&f.pdf).or_default().insert(f.pagina);
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
    let conceptos: Option<Vec<Concepto>> = leer_si_existe(&carpeta.join("conceptos.json"))?;
    let desarrollo: Vec<PreguntaDeDesarrollo> = leer_si_existe(&carpeta.join("desarrollo.json"))?.unwrap_or_default();
    let conceptos = conceptos.as_deref();
    let errores = [
        comprobar(&preguntas, conceptos, &examen, carpeta),
        comprobar_desarrollo(&desarrollo, &preguntas, conceptos.unwrap_or_default()),
    ]
    .concat();
    if errores.is_empty() {
        println!("El banco no tiene errores.");
    }
    for e in &errores {
        println!("{e}");
    }
    let congelado = if examen.congelado { cuentas(&preguntas, conceptos) } else { Vec::new() };
    let avisos = [
        pistas(&preguntas),
        explicaciones(&preguntas),
        paginas_sin_pregunta(conceptos.unwrap_or_default(), &examen, carpeta),
        frases(&desarrollo),
        congelado,
    ];
    for aviso in avisos.concat() {
        println!("Aviso: {aviso}");
    }
    let sin_banco = huerfanas(&preguntas, &carpeta.join("respuestas.jsonl"))?;
    if !sin_banco.is_empty() {
        println!("Aviso: {} id del historial no están en el banco: {}", sin_banco.len(), sin_banco.join(", "));
    }
    Ok(errores.is_empty())
}

/// Saca de cada PDF de `pdfs/` su texto, `texto/<pdf sin .pdf>.txt`, y la imagen de cada página,
/// `paginas/<pdf sin .pdf>-<página>.webp`, a 200 ppp y calidad 60. Un texto o una imagen que ya existe se queda.
pub fn sacar(carpeta: &Path) -> Result<(), String> {
    let destino = carpeta.join("paginas");
    let textos = carpeta.join("texto");
    for d in [&destino, &textos] {
        fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let leidos = fs::read_dir(carpeta.join("pdfs")).map_err(|e| format!("{}/pdfs: {e}", carpeta.display()))?;
    let mut pdfs: Vec<_> =
        leidos.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "pdf")).collect();
    pdfs.sort();
    for pdf in pdfs {
        let base = pdf.file_stem().unwrap().to_string_lossy();
        let texto = textos.join(format!("{base}.txt"));
        if !texto.exists() {
            let estado =
                Command::new("pdftotext").arg("-layout").arg(&pdf).arg(&texto).status().map_err(|e| format!("pdftotext: {e}"))?;
            if !estado.success() {
                return Err(format!("pdftotext falla en {}", pdf.display()));
            }
        }
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

/// Un fallo del informe de un revisor. «La revisión» del doc de cada tipo, como `docs/TEST.md`, define cada campo.
#[derive(Deserialize)]
pub struct Fallo {
    /// El `id` de la pregunta o del concepto, o `null` para un fallo sin `id`, como una página sin concepto.
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub gravedad: String,
    #[serde(default)]
    pub fallo: String,
    /// Los campos nuevos que el revisor propone, o `null`.
    #[serde(default)]
    pub arreglo: serde_json::Value,
}

/// Lo que un lote escribe en un apartado: un concepto o una pregunta.
pub trait DeUnApartado: serde::Serialize + serde::de::DeserializeOwned {
    /// Los campos que un arreglo de la revisión cambia. Otro campo deja el fallo para arreglarlo a mano.
    const CAMPOS: &[&str];
    fn id(&self) -> &str;
    fn apartado(&self) -> &str;
}

impl DeUnApartado for Pregunta {
    const CAMPOS: &[&str] = &["pregunta", "correcta", "distractores", "explicacion"];
    fn id(&self) -> &str {
        &self.id
    }
    fn apartado(&self) -> &str {
        &self.apartado
    }
}

impl DeUnApartado for PreguntaDeDesarrollo {
    const CAMPOS: &[&str] = &["forma", "enunciado", "modelo", "puntos"];
    fn id(&self) -> &str {
        &self.id
    }
    fn apartado(&self) -> &str {
        &self.apartado
    }
}

impl DeUnApartado for Concepto {
    const CAMPOS: &[&str] = &["idea", "fuente"];
    fn id(&self) -> &str {
        &self.id
    }
    fn apartado(&self) -> &str {
        &self.apartado
    }
}

/// Aplica cada arreglo al banco, y devuelve los fallos que un agente arregla a mano: un fallo sin arreglo,
/// con un campo que un arreglo no cambia o con un valor que no encaja, o con un `id` fuera del banco.
pub fn aplicar<T: DeUnApartado>(banco: &mut [T], fallos: Vec<Fallo>) -> Vec<Fallo> {
    let mut pendientes = Vec::new();
    for f in fallos {
        let campos = f.arreglo.as_object().filter(|a| !a.is_empty() && a.keys().all(|k| T::CAMPOS.contains(&k.as_str())));
        let item = banco.iter_mut().find(|x| Some(x.id()) == f.id.as_deref());
        let arreglado = campos.zip(item).and_then(|(campos, item)| {
            let mut valor = serde_json::to_value(&*item).ok()?;
            valor.as_object_mut()?.extend(campos.clone());
            *item = serde_json::from_value(valor).ok()?;
            Some(())
        });
        if arreglado.is_none() {
            pendientes.push(f);
        }
    }
    pendientes
}

fn revisar<T: DeUnApartado>(archivo: &Path, informes: &[String]) -> Result<(), String> {
    let mut banco: Vec<T> = leer(archivo)?;
    let mut fallos = Vec::new();
    for informe in informes {
        fallos.extend(leer::<Vec<Fallo>>(Path::new(informe))?);
    }
    let total = fallos.len();
    let pendientes = aplicar(&mut banco, fallos);
    escribir(archivo, &banco)?;
    println!("{} arreglos aplicados.", total - pendientes.len());
    for f in pendientes {
        println!("A mano: {} ({}): {}", f.id.as_deref().unwrap_or("sin id"), f.gravedad, f.fallo);
    }
    Ok(())
}

/// Aplica los informes de la revisión al archivo del tipo, y lista los fallos sin aplicar.
pub fn revision(carpeta: &Path, tipo: Tipo, informes: &[String]) -> Result<(), String> {
    match tipo {
        Tipo::Conceptos => revisar::<Concepto>(&carpeta.join("conceptos.json"), informes),
        Tipo::Test => revisar::<Pregunta>(&carpeta.join("preguntas.json"), informes),
        Tipo::Desarrollo => revisar::<PreguntaDeDesarrollo>(&carpeta.join("desarrollo.json"), informes),
    }
}

/// Prepara un lote de apartados: quita del banco lo que el lote escribe, y crea la carpeta de trabajo de cada
/// apartado, con una copia de `examen.json`, un `preguntas.json` y un `desarrollo.json` vacíos, un `conceptos.json`
/// y un enlace `pdfs`.
///
/// - `conceptos` quita los conceptos de los apartados y el `sin_pregunta` de sus PDFs, porque la cobertura de las
///   páginas mira los conceptos. Falla con una pregunta de test o de desarrollo de los apartados: quedaría sin
///   concepto. El `conceptos.json` del trabajo está vacío.
/// - `test` quita las preguntas de los apartados. Falla con un apartado sin conceptos, y con un historial o una
///   ronda que usan los `id` del lote, porque los `id` nuevos los reusan. El `conceptos.json` del trabajo es una
///   copia del banco.
/// - `desarrollo` quita las preguntas de desarrollo de los apartados, y falla como `test`, con `desarrollo.jsonl`
///   como historial.
///
/// Cada fallo deja todo sin cambiar.
pub fn preparar(carpeta: &Path, tipo: Tipo, apartados: &[String]) -> Result<(), String> {
    let mut examen: Examen = leer(&carpeta.join("examen.json"))?;
    let banco: Vec<Pregunta> = leer(&carpeta.join("preguntas.json"))?;
    let conceptos: Vec<Concepto> = leer_si_existe(&carpeta.join("conceptos.json"))?.unwrap_or_default();
    if let Some(a) = apartados.iter().find(|a| !examen.pdfs.values().any(|t| t.apartados.contains(a))) {
        return Err(format!("el apartado {a} no está en examen.json"));
    }
    let trabajo = carpeta.join("trabajo");
    if let Some(a) = apartados.iter().find(|a| trabajo.join(a).exists()) {
        return Err(format!("{} ya existe", trabajo.join(a).display()));
    }
    let del_lote = |a: &String| apartados.contains(a);
    let conceptos_del_trabajo = match tipo {
        Tipo::Conceptos => {
            let desarrollo: Vec<PreguntaDeDesarrollo> = leer_si_existe(&carpeta.join("desarrollo.json"))?.unwrap_or_default();
            let con_preguntas = banco.iter().map(|p| &p.apartado).chain(desarrollo.iter().map(|d| &d.apartado));
            if let Some(a) = con_preguntas.into_iter().find(|a| del_lote(a)) {
                return Err(format!(
                    "el apartado {a} tiene preguntas de test o de desarrollo: un lote de conceptos las deja sin concepto"
                ));
            }
            let resto: Vec<&Concepto> = conceptos.iter().filter(|c| !del_lote(&c.apartado)).collect();
            examen.sin_pregunta.retain(|pdf, _| !examen.pdfs[pdf].apartados.iter().any(del_lote));
            escribir(&carpeta.join("conceptos.json"), &resto)?;
            escribir(&carpeta.join("examen.json"), &examen)?;
            Vec::new()
        }
        Tipo::Test => {
            let resto = sin_el_lote(carpeta, &banco, &conceptos, apartados, "respuestas.jsonl")?;
            escribir(&carpeta.join("preguntas.json"), &resto)?;
            conceptos
        }
        Tipo::Desarrollo => {
            let desarrollo: Vec<PreguntaDeDesarrollo> = leer_si_existe(&carpeta.join("desarrollo.json"))?.unwrap_or_default();
            let resto = sin_el_lote(carpeta, &desarrollo, &conceptos, apartados, "desarrollo.jsonl")?;
            escribir(&carpeta.join("desarrollo.json"), &resto)?;
            conceptos
        }
    };
    for a in apartados {
        let dir = trabajo.join(a);
        fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        escribir(&dir.join("examen.json"), &examen)?;
        escribir(&dir.join("preguntas.json"), &Vec::<Pregunta>::new())?;
        escribir(&dir.join("desarrollo.json"), &Vec::<PreguntaDeDesarrollo>::new())?;
        escribir(&dir.join("conceptos.json"), &conceptos_del_trabajo)?;
        std::os::unix::fs::symlink("../../pdfs", dir.join("pdfs")).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    Ok(())
}

/// Las preguntas del banco fuera de los apartados del lote. Falla con un apartado sin conceptos, y con un historial
/// o una ronda que usan los `id` del lote, porque los `id` nuevos los reusan.
fn sin_el_lote<T: DeUnApartado + Clone>(
    carpeta: &Path,
    banco: &[T],
    conceptos: &[Concepto],
    apartados: &[String],
    historial: &str,
) -> Result<Vec<T>, String> {
    if let Some(a) = apartados.iter().find(|a| !conceptos.iter().any(|c| &c.apartado == *a)) {
        return Err(format!("el apartado {a} no tiene conceptos: corre antes un lote de conceptos"));
    }
    let (resto, lote): (Vec<T>, Vec<T>) = banco.iter().cloned().partition(|p| !apartados.iter().any(|a| a == p.apartado()));
    let del_lote: HashSet<&str> = lote.iter().map(|p| p.id()).collect();
    let de_otro: HashSet<&str> = resto.iter().map(|p| p.id()).collect();
    // Un `id` sin pregunta con el prefijo de un apartado del lote es de una pregunta borrada, y `importar` lo reusa.
    let huerfano = |id: &str| !de_otro.contains(id) && apartados.iter().any(|a| id.starts_with(&format!("{a}-")));
    let usados = ids_del_historial(&carpeta.join(historial))?;
    let de_lote = usados.iter().filter(|id| del_lote.contains(id.as_str()) || huerfano(id)).count();
    if de_lote > 0 {
        return Err(format!(
            "{historial} tiene {de_lote} id de los apartados del lote: el dueño decide antes qué pasa con ellas"
        ));
    }
    if carpeta.join("ronda.json").exists() {
        return Err("ronda.json existe: una ronda a medias usa los id del banco".into());
    }
    Ok(resto)
}

/// El banco del archivo con lo que trae la carpeta de trabajo de cada apartado del lote. Falla con un `id` de otro
/// apartado o que ya está en el banco, como una sigla de dos apartados.
fn unir<T: DeUnApartado>(carpeta: &Path, archivo: &str, apartados: &[String]) -> Result<Vec<T>, String> {
    let mut banco: Vec<T> = leer_si_existe(&carpeta.join(archivo))?.unwrap_or_default();
    for a in apartados {
        let dir = carpeta.join("trabajo").join(a);
        let nuevos: Vec<T> = leer(&dir.join(archivo))?;
        if let Some(x) = nuevos.iter().find(|x| x.apartado() != a || banco.iter().any(|b| b.id() == x.id())) {
            return Err(format!("{}: {} no es del apartado {a}, o ya está en el banco", dir.display(), x.id()));
        }
        banco.extend(nuevos);
    }
    Ok(banco)
}

/// Une al banco lo que escribió cada apartado del lote, y en un lote de conceptos también el `sin_pregunta` de sus
/// PDFs. Falla sin cambiar nada con un `id` de otro apartado o que ya está en el banco. Devuelve cuántos añade.
pub fn montar(carpeta: &Path, tipo: Tipo, apartados: &[String]) -> Result<usize, String> {
    match tipo {
        Tipo::Conceptos => {
            let archivo = carpeta.join("conceptos.json");
            let antes = leer_si_existe::<Vec<Concepto>>(&archivo)?.unwrap_or_default().len();
            let banco: Vec<Concepto> = unir(carpeta, "conceptos.json", apartados)?;
            let mut examen: Examen = leer(&carpeta.join("examen.json"))?;
            for a in apartados {
                let copia: Examen = leer(&carpeta.join("trabajo").join(a).join("examen.json"))?;
                for (pdf, t) in &examen.pdfs {
                    if let (true, Some(razones)) = (t.apartados.contains(a), copia.sin_pregunta.get(pdf)) {
                        examen.sin_pregunta.entry(pdf.clone()).or_default().extend(razones.clone());
                    }
                }
            }
            escribir(&archivo, &banco)?;
            escribir(&carpeta.join("examen.json"), &examen)?;
            Ok(banco.len() - antes)
        }
        Tipo::Test => montar_archivo::<Pregunta>(carpeta, "preguntas.json", apartados),
        Tipo::Desarrollo => montar_archivo::<PreguntaDeDesarrollo>(carpeta, "desarrollo.json", apartados),
    }
}

fn montar_archivo<T: DeUnApartado>(carpeta: &Path, archivo: &str, apartados: &[String]) -> Result<usize, String> {
    let antes = leer_si_existe::<Vec<T>>(&carpeta.join(archivo))?.unwrap_or_default().len();
    let banco: Vec<T> = unir(carpeta, archivo, apartados)?;
    escribir(&carpeta.join(archivo), &banco)?;
    Ok(banco.len() - antes)
}

/// Una pregunta de un bloque del borrador, sin `id` ni `apartado`. El encabezado es su concepto. «El borrador» de `docs/TEST.md` describe el formato del bloque.
fn bloque(concepto: &str, lineas: &[String]) -> Result<Pregunta, String> {
    let error = |e: &str| format!("{concepto}: {e}");
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
        concepto: concepto.into(),
        apartado: String::new(),
        pregunta: pregunta.to_string(),
        correcta: correcta.to_string(),
        distractores,
        explicacion: explicacion.join("\n"),
    })
}

/// Los bloques del borrador: el encabezado `## ` de cada uno, y sus líneas no vacías.
fn bloques(borrador: &Path) -> Result<Vec<(String, Vec<String>)>, String> {
    let texto = fs::read_to_string(borrador).map_err(|e| format!("{}: {e}", borrador.display()))?;
    let mut bloques: Vec<(String, Vec<String>)> = Vec::new();
    for l in texto.lines().filter(|l| !l.trim().is_empty()) {
        match (l.strip_prefix("## "), bloques.last_mut()) {
            (Some(encabezado), _) => bloques.push((encabezado.trim().into(), Vec::new())),
            (None, Some((_, lineas))) => lineas.push(l.into()),
            (None, None) => return Err(format!("una línea antes del primer «## »: {l}")),
        }
    }
    if bloques.is_empty() {
        return Err(format!("el borrador no tiene bloques: {}", borrador.display()));
    }
    Ok(bloques)
}

/// Un concepto de un bloque del borrador de conceptos: el encabezado es su `id`, la línea sin marca su idea, y la
/// línea `@ <página>` la página de su `fuente` en el PDF del apartado.
fn bloque_de_concepto(id: &str, apartado: &str, pdf: &str, lineas: &[String]) -> Result<Concepto, String> {
    let error = |e: &str| format!("{id}: {e}");
    if !id.starts_with(&format!("{apartado}-")) && !sigla(id) {
        return Err(error(&format!("el id no empieza por {apartado}- ni por sigla-")));
    }
    let (paginas, ideas): (Vec<&String>, Vec<&String>) = lineas.iter().partition(|l| l.starts_with('@'));
    let pagina = paginas.iter().map(|l| l[1..].trim().parse::<u32>());
    let ([idea], [Ok(pagina)]) = (&ideas[..], &pagina.collect::<Vec<_>>()[..]) else {
        return Err(error("el bloque no tiene una línea de idea y una línea @ <página>"));
    };
    let fuente = Fuente { pdf: pdf.into(), pagina: *pagina };
    Ok(Concepto { id: id.into(), apartado: apartado.into(), idea: idea.trim().into(), fuente })
}

/// Una pregunta de desarrollo de un bloque del borrador, sin `id` ni `apartado`. «El borrador» de
/// `docs/DESARROLLO.md` describe el formato del bloque.
fn bloque_de_desarrollo(forma: &str, lineas: &[String]) -> Result<PreguntaDeDesarrollo, String> {
    let forma: Forma = serde_json::from_value(forma.into()).map_err(|_| format!("la forma {forma} no es una de las 9"))?;
    let (mut enunciado, mut modelo, mut puntos) = (Vec::new(), Vec::new(), Vec::new());
    for l in lineas {
        if let Some(m) = l.strip_prefix("> ") {
            modelo.push(m.trim());
        } else if let Some(p) = l.strip_prefix("- ") {
            puntos.push(p.trim().to_string());
        } else {
            enunciado.push(l.trim());
        }
    }
    let ([enunciado], false, false) = (&enunciado[..], modelo.is_empty(), puntos.is_empty()) else {
        return Err("el bloque no tiene una línea de enunciado, una o más líneas > y una o más líneas -".into());
    };
    let (id, apartado, enunciado, modelo) = (String::new(), String::new(), enunciado.to_string(), modelo.join(" "));
    Ok(PreguntaDeDesarrollo { id, apartado, forma, enunciado, modelo, puntos })
}

/// Añade al final del archivo del tipo lo que trae el borrador. Una pregunta de test o de desarrollo recibe el
/// siguiente `id` libre del apartado, y un concepto lleva el `id` de su encabezado. Un concepto que ya está en el
/// banco da un error, y el banco queda igual. Devuelve los `id` nuevos.
pub fn importar(carpeta: &Path, tipo: Tipo, apartado: &str, borrador: &Path) -> Result<Vec<String>, String> {
    let examen: Examen = leer(&carpeta.join("examen.json"))?;
    let pdfs: Vec<&String> =
        examen.pdfs.iter().filter(|(_, t)| t.apartados.iter().any(|a| a == apartado)).map(|(p, _)| p).collect();
    if pdfs.is_empty() {
        return Err(format!("el apartado {apartado} no está en examen.json"));
    }
    let bloques = bloques(borrador)?;
    match tipo {
        Tipo::Desarrollo => {
            let archivo = carpeta.join("desarrollo.json");
            let mut banco: Vec<PreguntaDeDesarrollo> = leer_si_existe(&archivo)?.unwrap_or_default();
            let prefijo = format!("{apartado}-d");
            let mut n = banco.iter().filter_map(|d| d.id.strip_prefix(&prefijo)?.parse::<u32>().ok()).max().unwrap_or(0);
            let mut ids = Vec::new();
            for (i, (forma, lineas)) in bloques.iter().enumerate() {
                let mut d = bloque_de_desarrollo(forma, lineas).map_err(|e| format!("el bloque {} ({forma}): {e}", i + 1))?;
                n += 1;
                d.id = format!("{prefijo}{n:03}");
                d.apartado = apartado.into();
                ids.push(d.id.clone());
                banco.push(d);
            }
            escribir(&archivo, &banco)?;
            Ok(ids)
        }
        Tipo::Conceptos => {
            let [pdf] = pdfs[..] else {
                return Err(format!("el apartado {apartado} está en {} PDFs de examen.json", pdfs.len()));
            };
            let archivo = carpeta.join("conceptos.json");
            let mut conceptos: Vec<Concepto> = leer_si_existe(&archivo)?.unwrap_or_default();
            let mut ids = Vec::new();
            for (id, lineas) in &bloques {
                let c = bloque_de_concepto(id, apartado, pdf, lineas)?;
                if conceptos.iter().any(|b| b.id == c.id) {
                    return Err(format!("el concepto {} ya está en el banco", c.id));
                }
                ids.push(c.id.clone());
                conceptos.push(c);
            }
            escribir(&archivo, &conceptos)?;
            Ok(ids)
        }
        Tipo::Test => {
            let archivo = carpeta.join("preguntas.json");
            let mut banco: Vec<Pregunta> = leer(&archivo)?;
            let siguiente = |prefijo: String| {
                banco.iter().filter_map(|p| p.id.strip_prefix(&prefijo)?.parse::<u32>().ok()).max().unwrap_or(0)
            };
            let (mut normal, mut de_sigla) = (siguiente(format!("{apartado}-")), siguiente(format!("{apartado}-s")));
            let mut nuevas = Vec::new();
            for (encabezado, lineas) in &bloques {
                let mut p = bloque(encabezado, lineas)?;
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nucleo::{Concepto, Forma, Fuente, PreguntaDeDesarrollo, Tema};

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
            retencion: 0.9,
            pdfs: BTreeMap::from([
                ("1_Legislacion.pdf".into(), tema("Legislación", &["rgpd", "lssi"])),
                ("2_Teoria.pdf".into(), tema("Teoría", &["ens"])),
            ]),
            apartados: ["rgpd", "lssi", "ens"].map(|a| (a.into(), a.to_uppercase())).into(),
            sin_pregunta: BTreeMap::new(),
            congelado: false,
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
        }
    }

    /// La pregunta cambiada, con otras 2 del mismo concepto para que el concepto tenga 3.
    fn con_pareja(p: Pregunta) -> Vec<Pregunta> {
        vec![p, pregunta("rgpd-002"), pregunta("rgpd-003")]
    }

    /// El concepto de `pregunta`.
    fn base() -> Vec<Concepto> {
        vec![concepto("rgpd-brecha-plazo", "rgpd", 5)]
    }

    fn un_error(preguntas: &[Pregunta], texto: &str) {
        let (dir, examen) = carpeta();
        let errores = comprobar(preguntas, Some(&base()), &examen, dir.path());
        assert_eq!(errores.len(), 1, "{errores:?}");
        assert!(errores[0].contains(texto), "{errores:?}");
    }

    #[test]
    fn una_pregunta_valida_no_da_errores() {
        let (dir, examen) = carpeta();
        assert_eq!(comprobar(&con_pareja(pregunta("rgpd-001")), Some(&base()), &examen, dir.path()), Vec::<String>::new());
    }

    #[test]
    fn un_id_repetido_falla() {
        un_error(&[pregunta("rgpd-001"), pregunta("rgpd-001"), pregunta("rgpd-002")], "rgpd-001: id repetido");
    }

    #[test]
    fn un_apartado_sin_nombre_falla() {
        let (dir, mut examen) = carpeta();
        examen.apartados.remove("ens");
        let errores = comprobar(&con_pareja(pregunta("rgpd-001")), Some(&base()), &examen, dir.path());
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
    fn un_concepto_con_2_o_con_4_preguntas_falla() {
        let (dir, examen) = carpeta();
        let tres = vec![pregunta("rgpd-001"), pregunta("rgpd-002"), pregunta("rgpd-003")];
        assert!(comprobar(&tres, Some(&base()), &examen, dir.path()).is_empty());
        un_error(&tres[..2], "rgpd-brecha-plazo: 2 preguntas, y no 3");
        un_error(&[tres, vec![pregunta("rgpd-004")]].concat(), "rgpd-brecha-plazo: 4 preguntas");
    }

    #[test]
    fn sin_conceptos_json_falla_salvo_en_un_examen_congelado() {
        let (dir, examen) = carpeta();
        let tres = con_pareja(pregunta("rgpd-001"));
        assert_eq!(comprobar(&tres, None, &examen, dir.path()), ["falta conceptos.json, y el examen no está congelado"]);
        let examen = Examen { congelado: true, ..examen };
        assert!(comprobar(&tres, None, &examen, dir.path()).is_empty());
        let fuera = con_pareja(Pregunta { apartado: "nada".into(), ..pregunta("rgpd-001") });
        assert_eq!(
            comprobar(&fuera, None, &examen, dir.path()),
            ["rgpd-001: el apartado nada no está en ningún tema de examen.json"]
        );
    }

    #[test]
    fn un_examen_congelado_no_falla_por_la_cuenta_de_un_concepto() {
        let (dir, examen) = carpeta();
        let examen = Examen { congelado: true, ..examen };
        assert!(comprobar(&[pregunta("rgpd-001")], None, &examen, dir.path()).is_empty());
        assert_eq!(cuentas(&[pregunta("rgpd-001")], None), ["rgpd-brecha-plazo: 1 pregunta, y no 3"]);
    }

    #[test]
    fn un_concepto_de_sigla_tiene_1_pregunta() {
        let (dir, examen) = carpeta();
        let sigla = |id: &str| Pregunta { concepto: "sigla-rgpd".into(), ..pregunta(id) };
        let conceptos = [concepto("sigla-rgpd", "rgpd", 4)];
        assert!(comprobar(&[sigla("rgpd-001")], Some(&conceptos), &examen, dir.path()).is_empty());
        un_error_con(&[sigla("rgpd-001"), sigla("rgpd-002")], &conceptos, "sigla-rgpd: 2 preguntas, y no 1");
    }

    fn concepto(id: &str, apartado: &str, pagina: u32) -> Concepto {
        let fuente = Fuente { pdf: "1_Legislacion.pdf".into(), pagina };
        Concepto { id: id.into(), apartado: apartado.into(), idea: format!("La idea de {id}."), fuente }
    }

    /// Las 3 preguntas de `rgpd-brecha-plazo`, y su concepto.
    fn con_conceptos() -> (Vec<Pregunta>, Vec<Concepto>) {
        (con_pareja(pregunta("rgpd-001")), vec![concepto("rgpd-brecha-plazo", "rgpd", 5)])
    }

    fn un_error_con(preguntas: &[Pregunta], conceptos: &[Concepto], texto: &str) {
        let (dir, examen) = carpeta();
        let errores = comprobar(preguntas, Some(conceptos), &examen, dir.path());
        assert_eq!(errores.len(), 1, "{errores:?}");
        assert!(errores[0].contains(texto), "{errores:?}");
    }

    #[test]
    fn un_concepto_repetido_fuera_de_su_pdf_o_de_sus_paginas_falla() {
        let (preguntas, c) = con_conceptos();
        let otro = |id: &str, apartado: &str, pagina| [c.clone(), vec![concepto(id, apartado, pagina)]].concat();
        un_error_con(&preguntas, &otro("rgpd-brecha-plazo", "rgpd", 5), "rgpd-brecha-plazo: id repetido");
        un_error_con(&preguntas, &otro("ens-x", "ens", 5), "ens-x: el apartado ens no es un apartado de 1_Legislacion.pdf");
        un_error_con(&preguntas, &otro("lssi-x", "lssi", 6), "lssi-x: la página 6 no está entre las 5");
        let mut fuera = otro("ens-x", "ens", 1);
        fuera[1].fuente.pdf = "2_Teoria.pdf".into();
        un_error_con(&preguntas, &fuera, "ens-x: el pdf no está en examen.json o en pdfs/: 2_Teoria.pdf");
    }

    #[test]
    fn una_pregunta_de_un_concepto_que_no_esta_en_conceptos_json_falla() {
        let (mut preguntas, conceptos) = con_conceptos();
        preguntas.push(Pregunta { concepto: "sigla-rgpd".into(), ..pregunta("rgpd-004") });
        un_error_con(&preguntas, &conceptos, "rgpd-004: el concepto sigla-rgpd no está en conceptos.json");
    }

    #[test]
    fn una_pregunta_de_otro_apartado_que_su_concepto_falla() {
        let (mut preguntas, conceptos) = con_conceptos();
        preguntas[0].apartado = "lssi".into();
        let (dir, examen) = carpeta();
        let errores = comprobar(&preguntas, Some(&conceptos), &examen, dir.path());
        assert!(
            errores.contains(&"rgpd-001: el apartado lssi no es el de su concepto rgpd-brecha-plazo, rgpd".to_string()),
            "{errores:?}"
        );
    }

    #[test]
    fn un_apartado_sin_preguntas_acepta_conceptos_sin_preguntas_y_uno_con_preguntas_no() {
        let (preguntas, c) = con_conceptos();
        let (dir, examen) = carpeta();
        let lssi = [c.clone(), vec![concepto("lssi-cookies", "lssi", 4)]].concat();
        assert!(comprobar(&preguntas, Some(&lssi), &examen, dir.path()).is_empty());
        let rgpd = [c.clone(), vec![concepto("rgpd-dpd", "rgpd", 4)]].concat();
        un_error_con(&preguntas, &rgpd, "rgpd-dpd: 0 preguntas, y no 3");
        let sigla = [c, vec![concepto("sigla-rgpd", "rgpd", 4)]].concat();
        let con_sigla = [preguntas, vec![Pregunta { concepto: "sigla-rgpd".into(), ..pregunta("rgpd-004") }]].concat();
        assert!(comprobar(&con_sigla, Some(&sigla), &examen, dir.path()).is_empty());
    }

    #[test]
    fn la_cobertura_de_paginas_mira_las_fuentes_de_los_conceptos() {
        let (dir, examen) = carpeta();
        let (_, mut conceptos) = con_conceptos();
        conceptos.push(concepto("lssi-cookies", "lssi", 4));
        assert_eq!(
            paginas_sin_pregunta(&conceptos, &examen, dir.path()),
            ["1_Legislacion.pdf: páginas sin pregunta y sin razón en sin_pregunta: 1, 2, 3"]
        );
    }

    /// 9 conceptos de `rgpd`, `rgpd-0` a `rgpd-8`, y uno de `lssi`.
    fn conceptos_de_desarrollo() -> Vec<Concepto> {
        (0..9).map(|i| concepto(&format!("rgpd-{i}"), "rgpd", 1)).chain([concepto("lssi-0", "lssi", 1)]).collect()
    }

    /// Una pregunta de desarrollo de `rgpd` con los puntos `rgpd-<i>`.
    fn desarrollo(id: &str, forma: Forma, puntos: std::ops::Range<usize>) -> PreguntaDeDesarrollo {
        PreguntaDeDesarrollo {
            id: id.into(),
            apartado: "rgpd".into(),
            forma,
            enunciado: "Describe el RGPD.".into(),
            modelo: "Uno. Dos. Tres. Cuatro.".into(),
            puntos: puntos.map(|i| format!("rgpd-{i}")).collect(),
        }
    }

    fn errores_de_desarrollo(d: &[PreguntaDeDesarrollo]) -> Vec<String> {
        comprobar_desarrollo(d, &[pregunta("rgpd-001")], &conceptos_de_desarrollo())
    }

    #[test]
    fn una_pregunta_de_desarrollo_valida_no_da_errores() {
        assert!(
            errores_de_desarrollo(&[desarrollo("rgpd-d001", Forma::Objeto, 0..4), desarrollo("rgpd-d002", Forma::Proceso, 4..9)])
                .is_empty()
        );
    }

    #[test]
    fn un_id_de_desarrollo_repetido_o_del_test_falla() {
        let d = |id: &str, r| desarrollo(id, Forma::Objeto, r);
        assert_eq!(errores_de_desarrollo(&[d("rgpd-d001", 0..4), d("rgpd-d001", 4..8)]), ["rgpd-d001: id repetido"]);
        assert_eq!(errores_de_desarrollo(&[d("rgpd-001", 0..4)]), ["rgpd-001: id repetido en preguntas.json"]);
    }

    #[test]
    fn un_punto_que_no_es_un_concepto_de_su_apartado_falla() {
        let mut d = desarrollo("rgpd-d001", Forma::Objeto, 0..4);
        d.puntos[0] = "lssi-0".into();
        d.puntos[1] = "rgpd-no-esta".into();
        assert_eq!(
            errores_de_desarrollo(&[d]),
            [
                "rgpd-d001: el punto lssi-0 no es un concepto de rgpd",
                "rgpd-d001: el punto rgpd-no-esta no es un concepto de rgpd"
            ]
        );
    }

    #[test]
    fn menos_de_4_puntos_o_mas_de_8_falla() {
        assert_eq!(
            errores_de_desarrollo(&[desarrollo("rgpd-d001", Forma::Objeto, 0..3)]),
            ["rgpd-d001: 3 puntos, y no de 4 a 8"]
        );
        assert_eq!(
            errores_de_desarrollo(&[desarrollo("rgpd-d001", Forma::Objeto, 0..9)]),
            ["rgpd-d001: 9 puntos, y no de 4 a 8"]
        );
        assert!(errores_de_desarrollo(&[desarrollo("rgpd-d001", Forma::Objeto, 0..8)]).is_empty());
    }

    #[test]
    fn un_concepto_en_2_preguntas_que_no_son_de_comparacion_falla() {
        let objeto = desarrollo("rgpd-d001", Forma::Objeto, 0..4);
        let comparacion = desarrollo("rgpd-d002", Forma::Comparacion, 2..6);
        assert!(errores_de_desarrollo(&[objeto.clone(), comparacion]).is_empty());
        let proceso = desarrollo("rgpd-d002", Forma::Proceso, 3..7);
        assert_eq!(
            errores_de_desarrollo(&[objeto, proceso]),
            ["rgpd-3: punto de rgpd-d001 y de rgpd-d002, que no son de la forma comparación"]
        );
    }

    #[test]
    fn una_respuesta_modelo_de_menos_de_4_frases_o_de_mas_de_8_es_un_aviso() {
        let con = |modelo: &str| PreguntaDeDesarrollo { modelo: modelo.into(), ..desarrollo("rgpd-d001", Forma::Objeto, 0..4) };
        assert!(frases(&[con("TLS 1.3 cifra. ¿Cómo? Con AES! Y más.")]).is_empty());
        assert_eq!(frases(&[con("Una. Dos. Tres")]), ["rgpd-d001: la respuesta modelo tiene 3 frases, y no de 4 a 8"]);
        assert_eq!(frases(&[con(&"Una. ".repeat(9))]), ["rgpd-d001: la respuesta modelo tiene 9 frases, y no de 4 a 8"]);
    }

    #[test]
    fn una_forma_desconocida_falla_y_la_nombra() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../servidor/tests/datos/prueba");
        let copia = tempfile::tempdir().unwrap();
        fs::create_dir(copia.path().join("pdfs")).unwrap();
        for archivo in ["examen.json", "conceptos.json", "preguntas.json", "desarrollo.json", "pdfs/1_Redes.pdf"] {
            fs::copy(dir.join(archivo), copia.path().join(archivo)).unwrap();
        }
        let texto = fs::read_to_string(copia.path().join("desarrollo.json")).unwrap().replacen("\"proceso\"", "\"caso\"", 1);
        fs::write(copia.path().join("desarrollo.json"), texto).unwrap();
        assert!(informe(copia.path()).unwrap_err().contains("caso"));
    }

    #[test]
    fn el_examen_de_prueba_del_servidor_pasa_la_comprobacion() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../servidor/tests/datos/prueba");
        assert!(informe(&dir).unwrap());
    }

    #[test]
    fn una_clave_que_no_esta_en_su_distractor_falla() {
        let (dir, examen) = carpeta();
        let con = |explicacion: &str| con_pareja(Pregunta { explicacion: explicacion.into(), ..pregunta("rgpd-001") });
        assert!(comprobar(&con("✓ x\n✗ «24 HORAS»: a\n✗ «un mes»: b"), Some(&base()), &examen, dir.path()).is_empty());
        un_error(&con("✓ x\n✗ «24 horas»: a\n✗ «48 horas»: b"), "rgpd-001: la línea ✗ 2");
        un_error(&con("✓ x\n✗ 24 horas: a"), "rgpd-001: la línea ✗ 1");
        un_error(&con("✓ x\n✗ «horas»: a\n✗ «un mes»: b\n✗ «Horas»: c"), "rgpd-001: la línea ✗ 3 repite");
        un_error(&con("✓ x\n✗ «»: a"), "rgpd-001: la línea ✗ 1");
        un_error(&con("✓ x\n✗ «24»: a\n✗ «Un»: b\n✗ «48»: c\n✗ «9»: d"), "rgpd-001: 4 líneas ✗, y 3 distractores");
    }

    #[test]
    fn una_linea_corta_o_un_matiz_repetido_en_el_apartado_es_un_aviso() {
        let con =
            |id: &str, apartado: &str, e: &str| Pregunta { apartado: apartado.into(), explicacion: e.into(), ..pregunta(id) };
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
        let razones =
            BTreeMap::from([("1".into(), "portada".into()), ("2".into(), "índice".into()), ("5".into(), "repetida".into())]);
        examen.sin_pregunta = BTreeMap::from([("1_Legislacion.pdf".into(), razones)]);
        assert_eq!(
            paginas_sin_pregunta(&[concepto("a", "rgpd", 3), concepto("b", "rgpd", 5)], &examen, dir.path()),
            [
                "1_Legislacion.pdf: páginas sin pregunta y sin razón en sin_pregunta: 4",
                "1_Legislacion.pdf: páginas con pregunta y con razón en sin_pregunta: 5"
            ]
        );
    }

    const BORRADOR: &str = "\
## rgpd-brecha-plazo
¿En qué plazo notifica el responsable una brecha?
+ 72 horas
✓ El RGPD da 72 horas desde que el responsable conoce la brecha.
- «24 horas» desde la brecha | es el plazo de otra norma
- «Un mes» | es el plazo de respuesta al interesado
- «48 horas» | no es un plazo del RGPD

## sigla-dpd
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
        let ids = importar(dir.path(), Tipo::Test, "rgpd", &dir.path().join("borrador.md")).unwrap();
        assert_eq!(ids, ["rgpd-008", "rgpd-s03"]);
        let banco: Vec<serde_json::Value> = leer(&dir.path().join("preguntas.json")).unwrap();
        assert_eq!(banco.len(), 4);
        assert_eq!(
            banco[2],
            serde_json::json!({
                "id": "rgpd-008",
                "concepto": "rgpd-brecha-nuevo",
                "apartado": "rgpd",
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
            (nuevo.replace("- «Un mes» |", "- Un mes |"), "rgpd", "rgpd-brecha-nuevo: el distractor no marca"),
            (nuevo.replace("- «Un mes» |", "- «Un mes»"), "rgpd", "rgpd-brecha-nuevo: el distractor no tiene un"),
            (nuevo.replace("| x", "| x | y"), "rgpd", "sigla-dpd: el distractor no tiene un"),
            (nuevo.replace("«Director»", "«»Director"), "rgpd", "sigla-dpd: el distractor no marca"),
            ("\n".into(), "rgpd", "el borrador no tiene bloques"),
            (nuevo.replace("✓ El DPD", "El DPD"), "rgpd", "sigla-dpd: el bloque no tiene una línea ✓"),
            (nuevo.clone(), "nada", "el apartado nada no está en examen.json"),
        ];
        for (borrador, apartado, error) in casos {
            let dir = para_importar(&borrador);
            let antes = fs::read_to_string(dir.path().join("preguntas.json")).unwrap();
            let e = importar(dir.path(), Tipo::Test, apartado, &dir.path().join("borrador.md")).unwrap_err();
            assert!(e.contains(error), "{e}");
            assert_eq!(fs::read_to_string(dir.path().join("preguntas.json")).unwrap(), antes);
        }
    }

    const CONCEPTOS: &str = "\
## rgpd-brecha-nueva
El responsable notifica una brecha a la autoridad en 72 horas desde que la conoce.
@ 4

## sigla-dpd
DPD es el Delegado de Protección de Datos.
@ 2
";

    /// La carpeta con `examen.json`, `conceptos.json` con `rgpd-brecha-plazo`, y el borrador de conceptos dado.
    fn para_importar_conceptos(borrador: &str) -> tempfile::TempDir {
        let dir = para_importar(borrador);
        escribir(&dir.path().join("conceptos.json"), &base()).unwrap();
        dir
    }

    #[test]
    fn importar_anade_cada_concepto_del_borrador_con_la_pagina_de_su_pdf() {
        let dir = para_importar_conceptos(CONCEPTOS);
        let ids = importar(dir.path(), Tipo::Conceptos, "rgpd", &dir.path().join("borrador.md")).unwrap();
        assert_eq!(ids, ["rgpd-brecha-nueva", "sigla-dpd"]);
        let conceptos: Vec<serde_json::Value> = leer(&dir.path().join("conceptos.json")).unwrap();
        assert_eq!(conceptos.len(), 3);
        assert_eq!(
            conceptos[1..],
            [
                serde_json::json!({
                    "id": "rgpd-brecha-nueva",
                    "apartado": "rgpd",
                    "idea": "El responsable notifica una brecha a la autoridad en 72 horas desde que la conoce.",
                    "fuente": { "pdf": "1_Legislacion.pdf", "pagina": 4 }
                }),
                serde_json::json!({
                    "id": "sigla-dpd",
                    "apartado": "rgpd",
                    "idea": "DPD es el Delegado de Protección de Datos.",
                    "fuente": { "pdf": "1_Legislacion.pdf", "pagina": 2 }
                })
            ]
        );
    }

    #[test]
    fn importar_conceptos_falla_sin_tocar_conceptos_json() {
        let casos = [
            (CONCEPTOS.replace("brecha-nueva", "brecha-plazo"), "rgpd", "el concepto rgpd-brecha-plazo ya está en el banco"),
            (CONCEPTOS.replace("sigla-dpd", "rgpd-brecha-nueva"), "rgpd", "el concepto rgpd-brecha-nueva ya está en el banco"),
            (CONCEPTOS.replace("## rgpd-", "## lssi-"), "rgpd", "lssi-brecha-nueva: el id no empieza por rgpd- ni por sigla-"),
            (
                CONCEPTOS.replace("@ 4", "@ cuatro"),
                "rgpd",
                "rgpd-brecha-nueva: el bloque no tiene una línea de idea y una línea @ <página>",
            ),
            (CONCEPTOS.replace("@ 2\n", ""), "rgpd", "sigla-dpd: el bloque no tiene una línea de idea"),
            (CONCEPTOS.replace("@ 2", "Otra idea.\n@ 2"), "rgpd", "sigla-dpd: el bloque no tiene una línea de idea"),
            (CONCEPTOS.replace("## rgpd-", "## lssi-"), "lssi", "el apartado lssi está en 2 PDFs de examen.json"),
            (CONCEPTOS.to_string(), "nada", "el apartado nada no está en examen.json"),
        ];
        for (borrador, apartado, error) in casos {
            let dir = para_importar_conceptos(&borrador);
            let antes = fs::read_to_string(dir.path().join("conceptos.json")).unwrap();
            let e = importar(dir.path(), Tipo::Conceptos, apartado, &dir.path().join("borrador.md")).unwrap_err();
            assert!(e.contains(error), "{e}");
            assert_eq!(fs::read_to_string(dir.path().join("conceptos.json")).unwrap(), antes);
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
    fn sacar_da_un_texto_por_pdf_y_una_imagen_por_pagina_y_deja_las_que_ya_existen() {
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
        // Un form feed cierra cada página del texto.
        assert_eq!(fs::read_to_string(dir.path().join("texto/2 _Teoría.txt")).unwrap().matches('\x0c').count(), 2);
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
    fn aplicar_a_un_concepto_cambia_su_idea_y_su_fuente() {
        let mut banco = vec![concepto("rgpd-brecha-plazo", "rgpd", 5)];
        let fallos: Vec<Fallo> = serde_json::from_str(
            r#"[
                {"id": "rgpd-brecha-plazo", "regla": 0, "gravedad": "leve", "fallo": "x",
                 "arreglo": {"idea": "Otra idea.", "fuente": {"pdf": "1_Legislacion.pdf", "pagina": 7}}},
                {"id": "rgpd-brecha-plazo", "regla": 2, "gravedad": "leve", "fallo": "y", "arreglo": {"correcta": "1"}}
            ]"#,
        )
        .unwrap();
        let pendientes: Vec<String> = aplicar(&mut banco, fallos).into_iter().map(|f| f.fallo).collect();
        assert_eq!(pendientes, ["y"]);
        assert_eq!(banco[0].idea, "Otra idea.");
        assert_eq!(banco[0].fuente.pagina, 7);
    }

    const DESARROLLO: &str = "\
## proceso
Describe cómo notifica el responsable una brecha de datos.
> El responsable notifica la brecha a la autoridad en 72 horas.
> Después avisa a los interesados.
- rgpd-1
- rgpd-2

## comparacion
Compara la LSSI y el RGPD.
> La LSSI regula los servicios.
- rgpd-3
";

    /// La carpeta con `examen.json`, `preguntas.json` con `rgpd-007`, `desarrollo.json` con `rgpd-d002`, y el borrador.
    fn para_importar_desarrollo(borrador: &str) -> tempfile::TempDir {
        let dir = para_importar(borrador);
        escribir(&dir.path().join("desarrollo.json"), &vec![desarrollo("rgpd-d002", Forma::Objeto, 0..4)]).unwrap();
        dir
    }

    #[test]
    fn importar_desarrollo_anade_cada_bloque_con_el_siguiente_id_libre() {
        let dir = para_importar_desarrollo(DESARROLLO);
        let ids = importar(dir.path(), Tipo::Desarrollo, "rgpd", &dir.path().join("borrador.md")).unwrap();
        assert_eq!(ids, ["rgpd-d003", "rgpd-d004"]);
        let banco: Vec<serde_json::Value> = leer(&dir.path().join("desarrollo.json")).unwrap();
        assert_eq!(banco.len(), 3);
        assert_eq!(
            banco[1],
            serde_json::json!({
                "id": "rgpd-d003",
                "apartado": "rgpd",
                "forma": "proceso",
                "enunciado": "Describe cómo notifica el responsable una brecha de datos.",
                "modelo": "El responsable notifica la brecha a la autoridad en 72 horas. Después avisa a los interesados.",
                "puntos": ["rgpd-1", "rgpd-2"]
            })
        );
        assert_eq!(banco[2]["forma"], "comparacion");

        let dir = para_importar(DESARROLLO);
        let ids = importar(dir.path(), Tipo::Desarrollo, "nada", &dir.path().join("borrador.md"));
        assert!(ids.unwrap_err().contains("el apartado nada no está en examen.json"));
        let ids = importar(dir.path(), Tipo::Desarrollo, "rgpd", &dir.path().join("borrador.md")).unwrap();
        assert_eq!(ids, ["rgpd-d001", "rgpd-d002"]);
    }

    #[test]
    fn importar_desarrollo_falla_sin_tocar_desarrollo_json() {
        let casos = [
            (DESARROLLO.replace("## proceso", "## caso"), "el bloque 1 (caso): la forma caso no es una de las 9"),
            (DESARROLLO.replace("> La LSSI", "La LSSI"), "el bloque 2 (comparacion): el bloque no tiene"),
            (DESARROLLO.replace("- rgpd-3\n", ""), "el bloque 2 (comparacion): el bloque no tiene"),
            (DESARROLLO.replace("> El responsable", "El responsable"), "el bloque 1 (proceso): el bloque no tiene"),
            ("\n".into(), "el borrador no tiene bloques"),
        ];
        for (borrador, error) in casos {
            let dir = para_importar_desarrollo(&borrador);
            let antes = fs::read_to_string(dir.path().join("desarrollo.json")).unwrap();
            let e = importar(dir.path(), Tipo::Desarrollo, "rgpd", &dir.path().join("borrador.md")).unwrap_err();
            assert!(e.contains(error), "{e}");
            assert_eq!(fs::read_to_string(dir.path().join("desarrollo.json")).unwrap(), antes);
        }
    }

    /// La carpeta con `examen.json` con una razón de `sin_pregunta` en `1_Legislacion.pdf`, y `preguntas.json` con
    /// `r1` de `rgpd` y `e1` de `ens`.
    fn para_un_lote() -> (tempfile::TempDir, Examen) {
        let (dir, mut examen) = carpeta();
        let c = dir.path();
        examen.sin_pregunta.insert("1_Legislacion.pdf".into(), BTreeMap::from([("1".into(), "portada".into())]));
        escribir(&c.join("examen.json"), &examen).unwrap();
        let otra = Pregunta { apartado: "ens".into(), ..pregunta("e1") };
        escribir(&c.join("preguntas.json"), &vec![pregunta("r1"), otra]).unwrap();
        (dir, examen)
    }

    #[test]
    fn preparar_test_vacia_las_preguntas_del_lote_y_montar_las_devuelve() {
        let (dir, examen) = para_un_lote();
        let c = dir.path();
        let apartados = vec!["rgpd".to_string()];
        assert!(preparar(c, Tipo::Test, &apartados).unwrap_err().contains("el apartado rgpd no tiene conceptos"));
        escribir(&c.join("conceptos.json"), &base()).unwrap();
        fs::write(c.join("respuestas.jsonl"), "{\"id\":\"r1\",\"ts\":1,\"resultado\":\"Acierto\",\"seguro\":true}\n").unwrap();
        assert!(preparar(c, Tipo::Test, &apartados).unwrap_err().contains("respuestas.jsonl"));
        fs::remove_file(c.join("respuestas.jsonl")).unwrap();

        preparar(c, Tipo::Test, &apartados).unwrap();
        let banco: Vec<Pregunta> = leer(&c.join("preguntas.json")).unwrap();
        assert_eq!(banco.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), ["e1"]);
        // `sin_pregunta` mira las páginas de los conceptos, y es del lote de conceptos.
        assert_eq!(leer::<Examen>(&c.join("examen.json")).unwrap().sin_pregunta, examen.sin_pregunta);
        let t = c.join("trabajo/rgpd");
        assert!(t.join("pdfs/1_Legislacion.pdf").exists());
        assert_eq!(leer::<Vec<Concepto>>(&t.join("conceptos.json")).unwrap(), base());
        assert!(preparar(c, Tipo::Test, &apartados).unwrap_err().contains("ya existe"));

        escribir(&t.join("preguntas.json"), &vec![pregunta("r2")]).unwrap();
        assert_eq!(montar(c, Tipo::Test, &apartados).unwrap(), 1);
        assert_eq!(leer::<Vec<Pregunta>>(&c.join("preguntas.json")).unwrap().len(), 2);
        assert!(montar(c, Tipo::Test, &apartados).unwrap_err().contains("r2"));
    }

    #[test]
    fn preparar_conceptos_falla_con_preguntas_del_lote_y_montar_une_conceptos_y_sin_pregunta() {
        let (dir, examen) = para_un_lote();
        let c = dir.path();
        let otro = concepto("ens-otro", "ens", 1);
        escribir(&c.join("conceptos.json"), &vec![concepto("rgpd-brecha-plazo", "rgpd", 5), otro.clone()]).unwrap();
        let rgpd = vec!["rgpd".to_string()];
        let e = preparar(c, Tipo::Conceptos, &rgpd).unwrap_err();
        assert!(e.contains("el apartado rgpd tiene preguntas de test o de desarrollo"), "{e}");
        let desarrollo = PreguntaDeDesarrollo { apartado: "lssi".into(), ..desarrollo("lssi-d001", Forma::Objeto, 0..4) };
        escribir(&c.join("desarrollo.json"), &vec![desarrollo]).unwrap();
        let lssi = vec!["lssi".to_string()];
        assert!(preparar(c, Tipo::Conceptos, &lssi).unwrap_err().contains("el apartado lssi tiene preguntas"));
        fs::remove_file(c.join("desarrollo.json")).unwrap();
        escribir(&c.join("preguntas.json"), &Vec::<Pregunta>::new()).unwrap();

        preparar(c, Tipo::Conceptos, &rgpd).unwrap();
        assert_eq!(leer::<Vec<Concepto>>(&c.join("conceptos.json")).unwrap(), std::slice::from_ref(&otro));
        assert!(leer::<Examen>(&c.join("examen.json")).unwrap().sin_pregunta.is_empty());
        let t = c.join("trabajo/rgpd");
        assert!(t.join("pdfs/1_Legislacion.pdf").exists());
        assert_eq!(leer::<Vec<Concepto>>(&t.join("conceptos.json")).unwrap(), []);
        assert!(leer::<Vec<Pregunta>>(&t.join("preguntas.json")).unwrap().is_empty());

        escribir(&t.join("conceptos.json"), &vec![concepto("rgpd-nuevo", "rgpd", 2), otro]).unwrap();
        assert!(montar(c, Tipo::Conceptos, &rgpd).unwrap_err().contains("ens-otro"));
        escribir(&t.join("conceptos.json"), &vec![concepto("rgpd-nuevo", "rgpd", 2)]).unwrap();
        escribir(&t.join("examen.json"), &examen).unwrap();
        assert_eq!(montar(c, Tipo::Conceptos, &rgpd).unwrap(), 1);
        assert_eq!(leer::<Vec<Concepto>>(&c.join("conceptos.json")).unwrap().len(), 2);
        assert_eq!(leer::<Examen>(&c.join("examen.json")).unwrap().sin_pregunta, examen.sin_pregunta);
    }

    #[test]
    fn preparar_desarrollo_vacia_las_preguntas_del_lote_y_montar_y_revision_las_devuelven() {
        let (dir, _) = para_un_lote();
        let c = dir.path();
        let apartados = vec!["rgpd".to_string()];
        let otra = PreguntaDeDesarrollo { apartado: "ens".into(), ..desarrollo("ens-d001", Forma::Objeto, 0..4) };
        escribir(&c.join("desarrollo.json"), &vec![desarrollo("rgpd-d001", Forma::Objeto, 0..4), otra.clone()]).unwrap();
        assert!(preparar(c, Tipo::Desarrollo, &apartados).unwrap_err().contains("el apartado rgpd no tiene conceptos"));
        escribir(&c.join("conceptos.json"), &base()).unwrap();
        fs::write(c.join("desarrollo.jsonl"), "{\"id\":\"rgpd-d001\",\"ts\":1,\"texto\":\"x\"}\n").unwrap();
        let e = preparar(c, Tipo::Desarrollo, &apartados).unwrap_err();
        assert!(e.contains("desarrollo.jsonl tiene 1 id de los apartados del lote"), "{e}");
        fs::write(c.join("desarrollo.jsonl"), "{\"id\":\"rgpd-d009\",\"ts\":1,\"texto\":\"x\"}\n").unwrap();
        let e = preparar(c, Tipo::Desarrollo, &apartados).unwrap_err();
        assert!(e.contains("desarrollo.jsonl tiene 1 id de los apartados del lote"), "{e}");
        fs::write(c.join("desarrollo.jsonl"), "{\"id\":\"ens-d001\",\"ts\":1,\"texto\":\"x\"}\n").unwrap();
        fs::write(c.join("ronda.json"), "{}").unwrap();
        assert!(preparar(c, Tipo::Desarrollo, &apartados).unwrap_err().contains("ronda.json"));
        fs::remove_file(c.join("ronda.json")).unwrap();

        preparar(c, Tipo::Desarrollo, &apartados).unwrap();
        assert_eq!(leer::<Vec<PreguntaDeDesarrollo>>(&c.join("desarrollo.json")).unwrap(), std::slice::from_ref(&otra));
        assert_eq!(leer::<Vec<Pregunta>>(&c.join("preguntas.json")).unwrap().len(), 2);
        let t = c.join("trabajo/rgpd");
        assert!(t.join("pdfs/1_Legislacion.pdf").exists());
        assert_eq!(leer::<Vec<Concepto>>(&t.join("conceptos.json")).unwrap(), base());
        assert_eq!(leer::<Vec<PreguntaDeDesarrollo>>(&t.join("desarrollo.json")).unwrap(), []);

        escribir(&t.join("desarrollo.json"), &vec![otra.clone()]).unwrap();
        assert!(montar(c, Tipo::Desarrollo, &apartados).unwrap_err().contains("ens-d001"));
        escribir(&t.join("desarrollo.json"), &vec![desarrollo("rgpd-d001", Forma::Proceso, 0..5)]).unwrap();
        assert_eq!(montar(c, Tipo::Desarrollo, &apartados).unwrap(), 1);
        assert_eq!(leer::<Vec<PreguntaDeDesarrollo>>(&c.join("desarrollo.json")).unwrap().len(), 2);

        let informe = c.join("trabajo/revision-rgpd.json");
        fs::write(&informe, r#"[{"id": "rgpd-d001", "gravedad": "leve", "fallo": "x", "arreglo": {"modelo": "Otra."}}]"#)
            .unwrap();
        revision(c, Tipo::Desarrollo, &[informe.display().to_string()]).unwrap();
        let banco: Vec<PreguntaDeDesarrollo> = leer(&c.join("desarrollo.json")).unwrap();
        assert_eq!(banco.iter().find(|d| d.id == "rgpd-d001").unwrap().modelo, "Otra.");
    }

    #[test]
    fn montar_conceptos_une_el_sin_pregunta_de_dos_apartados_de_un_pdf() {
        let (dir, _) = para_un_lote();
        let c = dir.path();
        escribir(&c.join("preguntas.json"), &Vec::<Pregunta>::new()).unwrap();
        let apartados = vec!["rgpd".to_string(), "lssi".to_string()];
        preparar(c, Tipo::Conceptos, &apartados).unwrap();
        for (a, pagina) in [("rgpd", "2"), ("lssi", "3")] {
            let copia = c.join("trabajo").join(a).join("examen.json");
            let mut examen: Examen = leer(&copia).unwrap();
            examen.sin_pregunta.insert("1_Legislacion.pdf".into(), BTreeMap::from([(pagina.into(), a.into())]));
            escribir(&copia, &examen).unwrap();
        }
        montar(c, Tipo::Conceptos, &apartados).unwrap();
        let razones = BTreeMap::from([("2".into(), "rgpd".into()), ("3".into(), "lssi".into())]);
        assert_eq!(leer::<Examen>(&c.join("examen.json")).unwrap().sin_pregunta["1_Legislacion.pdf"], razones);
    }
}
