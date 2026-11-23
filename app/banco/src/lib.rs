//! Las herramientas del banco de un examen: la comprobación de `preguntas.json`, las imágenes de las páginas
//! de los PDFs y los arreglos de una revisión. Las 3 leen la carpeta del examen. La comprobación y las
//! imágenes llaman a `pdfinfo` y `pdftoppm` de poppler.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use nucleo::{Examen, Pregunta};
use serde::Deserialize;

fn leer<T: serde::de::DeserializeOwned>(archivo: &Path) -> Result<T, String> {
    let texto = fs::read_to_string(archivo).map_err(|e| format!("{}: {e}", archivo.display()))?;
    serde_json::from_str(&texto).map_err(|e| format!("{}: {e}", archivo.display()))
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
        let nombre = p.fuente.pdf.as_str();
        let pdf = carpeta.join("pdfs").join(nombre);
        let Some(temas) = examen.pdfs.get(nombre).filter(|_| pdf.is_file()) else {
            errores.push(format!("{id}: el pdf no está en examen.json o en pdfs/: {nombre}"));
            continue;
        };
        if !temas.contains(&p.tema) {
            errores.push(format!("{id}: el tema {} no es un tema de {nombre} en examen.json", p.tema));
        }
        match paginas_de.entry(nombre).or_insert_with(|| paginas(&pdf)) {
            Ok(n) if p.fuente.pagina == 0 || p.fuente.pagina > *n => {
                errores.push(format!("{id}: la página {} no está entre las {n} de {nombre}", p.fuente.pagina))
            }
            Ok(_) => {}
            Err(e) => errores.push(format!("{id}: {e}")),
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
    for aviso in pistas(&preguntas) {
        println!("Aviso: {aviso}");
    }
    let sin_banco = huerfanas(&preguntas, &carpeta.join("respuestas.jsonl"))?;
    if !sin_banco.is_empty() {
        println!("Aviso: {} id del historial no están en el banco: {}", sin_banco.len(), sin_banco.join(", "));
    }
    Ok(errores.is_empty())
}

/// Saca `paginas/<pdf sin .pdf>-<página>.png`, a 80 ppp, de cada página de los PDFs de `pdfs/`.
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
            if nombre.with_added_extension("png").exists() {
                continue;
            }
            let n = pagina.to_string();
            let estado = Command::new("pdftoppm")
                .args(["-png", "-r", "80", "-f", &n, "-l", &n, "-singlefile"])
                .arg(&pdf)
                .arg(&nombre)
                .status()
                .map_err(|e| format!("pdftoppm: {e}"))?;
            if !estado.success() {
                return Err(format!("pdftoppm falla en la página {pagina} de {}", pdf.display()));
            }
        }
    }
    Ok(())
}

/// Un fallo del informe de un revisor. La revisión de `BANCO.md` define cada campo.
#[derive(Deserialize)]
pub struct Fallo {
    #[serde(default)]
    pub id: String,
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
        let (Some(a), Some(p)) = (arreglo, banco.iter_mut().find(|p| p.id == f.id)) else {
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
    let texto = serde_json::to_string_pretty(&banco).map_err(|e| e.to_string())? + "\n";
    fs::write(&archivo, texto).map_err(|e| format!("{}: {e}", archivo.display()))?;
    println!("{} arreglos aplicados.", total - pendientes.len());
    for f in pendientes {
        println!("A mano: {} ({}): {}", f.id, f.gravedad, f.fallo);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nucleo::Fuente;
    use std::collections::BTreeMap;

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

    /// Una carpeta de examen con `1_Legislacion.pdf` de 5 páginas, para los temas `rgpd` y `lssi`.
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
                ("1_Legislacion.pdf".into(), vec!["rgpd".into(), "lssi".into()]),
                ("2_Teoria.pdf".into(), vec!["ens".into()]),
            ]),
        };
        (dir, examen)
    }

    fn pregunta(id: &str) -> Pregunta {
        Pregunta {
            id: id.into(),
            concepto: "rgpd-brecha-plazo".into(),
            tema: "rgpd".into(),
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
    fn un_tema_que_no_es_de_su_pdf_falla() {
        un_error(&con_pareja(Pregunta { tema: "ens".into(), ..pregunta("rgpd-001") }), "el tema ens");
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
        fs::write(paginas.join("1_Legislacion-1.png"), "ya estaba").unwrap();
        sacar(dir.path()).unwrap();
        let mut nombres: Vec<String> =
            fs::read_dir(&paginas).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
        nombres.sort();
        assert_eq!(
            nombres,
            [
                "1_Legislacion-1.png",
                "1_Legislacion-2.png",
                "1_Legislacion-3.png",
                "1_Legislacion-4.png",
                "1_Legislacion-5.png",
                "2 _Teoría-1.png",
                "2 _Teoría-2.png"
            ]
        );
        assert_eq!(fs::read_to_string(paginas.join("1_Legislacion-1.png")).unwrap(), "ya estaba");
        // 100 puntos a 80 ppp son 112 píxeles: el ancho va en los bytes 16 a 19 del PNG.
        let png = fs::read(paginas.join("1_Legislacion-2.png")).unwrap();
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 112);
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
                {"id": "rgpd-001", "regla": 4, "gravedad": "leve", "fallo": "v", "arreglo": {}}
            ]"#,
        )
        .unwrap();
        let pendientes: Vec<String> = aplicar(&mut banco, fallos).into_iter().map(|f| f.fallo).collect();
        assert_eq!(pendientes, ["y", "z", "w", "v"]);
        assert_eq!(banco[0].correcta, "3 días");
        assert_eq!(banco[0].id, "rgpd-001");
    }
}
