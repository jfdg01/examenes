//! La lógica de la app. `nucleo` no lee archivos ni la red.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Las cifras de un examen, de `examen.json`.
#[derive(Deserialize)]
pub struct Examen {
    pub titulo: String,
    /// Las preguntas del examen real.
    pub preguntas: u32,
    /// Las opciones de cada pregunta.
    pub opciones: u32,
    /// La duración del examen real.
    pub minutos: u32,
    /// La parte de un acierto que resta un fallo.
    pub penalizacion: f64,
}

/// Una pregunta del banco, de `preguntas.json`.
#[derive(Deserialize, Clone)]
pub struct Pregunta {
    pub id: String,
    pub concepto: String,
    pub tema: String,
    pub pregunta: String,
    pub correcta: String,
    pub distractores: Vec<String>,
    pub explicacion: String,
}

/// Una línea de `respuestas.jsonl`. `ts` es la hora en UTC, como `2026-11-04T12:41:22Z`.
#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Respuesta {
    pub id: String,
    pub ts: String,
    pub acierto: bool,
    pub seguro: bool,
}

/// El paso de un concepto: el número de aciertos con "seguro" seguidos.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Concepto {
    SinVer,
    Paso(u32),
}

impl Concepto {
    /// Un acierto con "seguro" sube el paso, aunque el concepto no haya vencido. Otra respuesta lo pone a 0.
    pub fn tras(self, acierto: bool, seguro: bool) -> Concepto {
        match (self, acierto && seguro) {
            (Concepto::SinVer, true) => Concepto::Paso(1),
            (Concepto::Paso(n), true) => Concepto::Paso(n.saturating_add(1)),
            (_, false) => Concepto::Paso(0),
        }
    }

    /// El hueco en segundos: 10 minutos × 3^(paso − 1), y 0 con el paso a 0.
    pub fn hueco(self) -> i64 {
        match self {
            Concepto::SinVer | Concepto::Paso(0) => 0,
            Concepto::Paso(n) => 600i64.saturating_mul(3i64.saturating_pow(n - 1)),
        }
    }
}

impl std::fmt::Display for Concepto {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Concepto::SinVer => write!(f, "sin ver"),
            Concepto::Paso(n) => write!(f, "paso {n}"),
        }
    }
}

/// Los segundos desde 1970 de una hora `AAAA-MM-DDTHH:MM:SSZ`.
pub fn segundos(ts: &str) -> Option<i64> {
    let b = ts.as_bytes();
    if b.len() != 20 || b[4] != b'-' || b[7] != b'-' || b[10] != b'T' || b[13] != b':' || b[16] != b':' || b[19] != b'Z' {
        return None;
    }
    let num = |a: usize, z: usize| ts.get(a..z)?.parse::<i64>().ok();
    let (a, m, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (h, mi, s) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) || h > 23 || mi > 59 || s > 59 {
        return None;
    }
    // Los días desde 1970 del calendario civil, de Howard Hinnant.
    let a = if m <= 2 { a - 1 } else { a };
    let era = a.div_euclid(400);
    let ano = a - era * 400;
    let dia_del_ano = (153 * ((m + 9) % 12) + 2) / 5 + d - 1;
    let dias = era * 146097 + ano * 365 + ano / 4 - ano / 100 + dia_del_ano - 719468;
    Some(dias * 86400 + h * 3600 + mi * 60 + s)
}

/// El paso de cada concepto visto, y la hora en que vence.
fn estados<'a>(banco: &'a [Pregunta], historial: &[Respuesta]) -> std::collections::HashMap<&'a str, (Concepto, i64)> {
    let concepto: std::collections::HashMap<&str, &str> = banco.iter().map(|q| (q.id.as_str(), q.concepto.as_str())).collect();
    let mut ordenado: Vec<(i64, &Respuesta)> = historial.iter().filter_map(|r| Some((segundos(&r.ts)?, r))).collect();
    ordenado.sort_by_key(|(t, _)| *t);
    let mut estado = std::collections::HashMap::new();
    for (t, r) in ordenado {
        let Some(&c) = concepto.get(r.id.as_str()) else {
            continue;
        };
        let antes = estado.get(c).map_or(Concepto::SinVer, |e: &(Concepto, i64)| e.0);
        let despues = antes.tras(r.acierto, r.seguro);
        estado.insert(c, (despues, t.saturating_add(despues.hueco())));
    }
    estado
}

/// Lo que muestra la pantalla del inicio.
#[derive(Serialize, TS, PartialEq, Debug)]
#[serde(tag = "estado")]
#[ts(export)]
pub enum Inicio {
    BancoVacio {
        titulo: String,
    },
    /// `sesion` y `simulacro` son el número de preguntas de cada ronda.
    Listo {
        titulo: String,
        sesion: u32,
        simulacro: u32,
        tarjeta: Tarjeta,
    },
}

/// La pantalla de la ronda. Del inicio sale una sesión, cada pregunta pasa de sin contestar a contestada,
/// y la última lleva al final. El botón ✕ y el final vuelven al inicio.
#[derive(Serialize, TS)]
#[serde(tag = "estado")]
#[ts(export)]
pub enum Ronda {
    Inicio,
    Pregunta { contestada: bool },
    Final,
}

/// El inicio, con la tarjeta de la nota para sacar más de `objetivo` sobre 10.
pub fn inicio(examen: &Examen, banco: &[Pregunta], historial: &[Respuesta], objetivo: f64) -> Inicio {
    let titulo = examen.titulo.clone();
    if banco.is_empty() {
        return Inicio::BancoVacio { titulo };
    }
    let tarjeta = tarjeta(examen, banco, historial, objetivo);
    let sesion = (tarjeta.sabes + tarjeta.no_sabes + tarjeta.sin_ver).min(TAMANO_SESION as u32);
    let simulacro = examen.preguntas.min(banco.len() as u32);
    Inicio::Listo { titulo, sesion, simulacro, tarjeta }
}

const TAMANO_SESION: usize = 12;
const HUECOS_SIN_VER: usize = 4;

/// El azar de la sesión, de una semilla: la misma semilla da el mismo orden.
pub struct Azar(u64);

impl Azar {
    pub fn new(semilla: u64) -> Azar {
        Azar(semilla)
    }

    /// Un número de `0..n`, con SplitMix64.
    fn hasta(&mut self, n: usize) -> usize {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        ((z ^ (z >> 31)) % n as u64) as usize
    }

    /// Fisher-Yates.
    fn barajar<T>(&mut self, lista: &mut [T]) {
        for i in (1..lista.len()).rev() {
            lista.swap(i, self.hasta(i + 1));
        }
    }
}

/// Una pregunta de la sesión, con lo que muestran la pregunta y su solución.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct PreguntaDeSesion {
    pub id: String,
    pub tema: String,
    pub concepto: String,
    pub pregunta: String,
    pub opciones: Vec<String>,
    /// El índice de la opción correcta en `opciones`.
    pub correcta: u32,
    pub explicacion: String,
    /// El paso del concepto antes de contestar.
    pub paso: String,
    /// El paso tras un acierto con "seguro".
    pub paso_tras_seguro: String,
    /// El paso tras un fallo, un acierto con "dudo" o una pregunta en blanco.
    pub paso_tras_otra: String,
}

/// La sesión de 12: los vencidos, 4 huecos para los conceptos sin ver, y el relleno con los que vencen antes.
/// `ahora` va en segundos desde 1970.
pub fn sesion(banco: &[Pregunta], historial: &[Respuesta], ahora: i64, azar: &mut Azar) -> Vec<PreguntaDeSesion> {
    let estado = estados(banco, historial);
    let mut ultima: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
    for r in historial {
        let ts = ultima.entry(r.id.as_str()).or_insert(r.ts.as_str());
        *ts = (*ts).max(r.ts.as_str());
    }
    // Una pregunta por concepto: la que no tiene respuestas, o la de la respuesta más vieja. El azar rompe los empates.
    let mut barajado: Vec<&Pregunta> = banco.iter().collect();
    azar.barajar(&mut barajado);
    let mut conceptos: Vec<&str> = Vec::new();
    let mut pregunta: std::collections::HashMap<&str, &Pregunta> = std::collections::HashMap::new();
    for q in barajado {
        let ultima_de = |q: &Pregunta| ultima.get(q.id.as_str()).copied();
        match pregunta.get(q.concepto.as_str()) {
            None => conceptos.push(&q.concepto),
            Some(otra) => {
                let gana = match (ultima_de(otra), ultima_de(q)) {
                    (Some(_), None) => true,
                    (Some(a), Some(b)) => b < a,
                    (None, _) => false,
                };
                if !gana {
                    continue;
                }
            }
        }
        pregunta.insert(&q.concepto, q);
    }
    let (mut vistos, sin_ver): (Vec<&str>, Vec<&str>) = conceptos.into_iter().partition(|c| estado.contains_key(c));
    vistos.sort_by_key(|c| estado[c].1);
    let huecos = TAMANO_SESION - if sin_ver.is_empty() { 0 } else { HUECOS_SIN_VER };
    let vencidos = vistos.iter().take_while(|c| estado[*c].1 <= ahora).count().min(huecos);
    let mut elegidos: Vec<&str> =
        vistos[..vencidos].iter().chain(&sin_ver).chain(&vistos[vencidos..]).take(TAMANO_SESION).copied().collect();
    azar.barajar(&mut elegidos);
    elegidos.into_iter().map(|c| de_la_ronda(pregunta[c], &estado, azar)).collect()
}

/// La pregunta con sus opciones al azar y el paso de su concepto.
fn de_la_ronda(q: &Pregunta, estado: &std::collections::HashMap<&str, (Concepto, i64)>, azar: &mut Azar) -> PreguntaDeSesion {
    let paso = estado.get(q.concepto.as_str()).map_or(Concepto::SinVer, |e| e.0);
    let mut opciones: Vec<String> = std::iter::once(&q.correcta).chain(&q.distractores).cloned().collect();
    azar.barajar(&mut opciones);
    PreguntaDeSesion {
        id: q.id.clone(),
        tema: q.tema.clone(),
        concepto: q.concepto.clone(),
        pregunta: q.pregunta.clone(),
        correcta: opciones.iter().position(|o| *o == q.correcta).unwrap() as u32,
        opciones,
        explicacion: q.explicacion.clone(),
        paso: paso.to_string(),
        paso_tras_seguro: paso.tras(true, true).to_string(),
        paso_tras_otra: paso.tras(false, false).to_string(),
    }
}

/// El simulacro: las preguntas del examen, al azar de todo el banco, y su duración.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct Simulacro {
    pub segundos: u32,
    pub preguntas: Vec<PreguntaDeSesion>,
}

pub fn simulacro(examen: &Examen, banco: &[Pregunta], historial: &[Respuesta], azar: &mut Azar) -> Simulacro {
    let estado = estados(banco, historial);
    let mut barajado: Vec<&Pregunta> = banco.iter().collect();
    azar.barajar(&mut barajado);
    barajado.truncate(examen.preguntas as usize);
    Simulacro {
        segundos: examen.minutos.saturating_mul(60),
        preguntas: barajado.into_iter().map(|q| de_la_ronda(q, &estado, azar)).collect(),
    }
}

/// Lo que pasó en una pregunta del simulacro.
#[derive(Deserialize, TS, Clone, Copy, PartialEq, Debug)]
#[ts(export)]
pub enum Resultado {
    Acierto,
    Fallo,
    Blanco,
}

/// El cierre de un simulacro: sus preguntas, y el resultado de las que tuvieron respuesta antes del límite.
#[derive(Deserialize, TS)]
#[ts(export)]
pub struct Cierre {
    pub preguntas: u32,
    pub resultados: Vec<Resultado>,
}

/// El final del simulacro. `nota` va sobre 10, con 2 decimales como mucho.
#[derive(Serialize, TS, PartialEq, Debug)]
#[ts(export)]
pub struct NotaDelSimulacro {
    pub aciertos: u32,
    pub fallos: u32,
    pub en_blanco: u32,
    pub nota: String,
}

/// La nota del simulacro: un fallo resta `penalizacion` aciertos, y cada pregunta sin resultado cuenta en blanco.
pub fn nota_del_simulacro(cierre: &Cierre, penalizacion: f64) -> NotaDelSimulacro {
    let cuenta = |r| cierre.resultados.iter().filter(|&&x| x == r).count() as u32;
    let (aciertos, fallos) = (cuenta(Resultado::Acierto), cuenta(Resultado::Fallo));
    let nota = (aciertos as f64 - penalizacion * fallos as f64) / cierre.preguntas.max(1) as f64 * 10.0;
    let nota = format!("{:.2}", (nota * 100.0).round() / 100.0);
    NotaDelSimulacro {
        aciertos,
        fallos,
        en_blanco: cierre.preguntas.saturating_sub(aciertos + fallos),
        nota: nota.trim_end_matches('0').trim_end_matches('.').replace('.', ","),
    }
}

/// La tarjeta de la nota, con la barra del camino. Los porcentajes van redondeados.
#[derive(Serialize, TS, PartialEq, Debug)]
#[ts(export)]
pub struct Tarjeta {
    pub sabes: u32,
    pub no_sabes: u32,
    pub sin_ver: u32,
    /// La probabilidad de sacar el objetivo hoy, con la forma de las dudas que da más.
    pub hoy: u32,
    /// Los conceptos en "sabes" que dan un 95 %.
    pub meta: u32,
    /// "✓", "≈1 sesión", "≈n sesiones" o "? sesiones".
    pub sesiones: String,
    pub consejo: String,
    /// La probabilidad de sacar el objetivo si contestas las dudas.
    pub si_contestas: u32,
    /// La probabilidad de sacar el objetivo si dejas las dudas en blanco.
    pub en_blanco: u32,
    /// Los aciertos en una duda: en lo que viste, en lo que no viste y de media.
    pub dudas: Vec<Dudas>,
}

#[derive(Serialize, TS, PartialEq, Debug)]
#[ts(export)]
pub struct Dudas {
    pub texto: String,
    pub conceptos: u32,
    pub porcentaje: u32,
    pub detalle: Option<String>,
}

/// La tarjeta de la nota para sacar más de `objetivo` sobre 10.
pub fn tarjeta(examen: &Examen, banco: &[Pregunta], historial: &[Respuesta], objetivo: f64) -> Tarjeta {
    let a_ciegas = 1.0 / examen.opciones as f64;
    let (sabes, no_sabes, sin_ver) = cuenta(banco, historial);
    let n = sabes + no_sabes + sin_ver;
    let total = n.max(1) as f64;
    let dudas = historial.iter().filter(|r| !r.seguro).count();
    let aciertos_dudo = historial.iter().filter(|r| !r.seguro && r.acierto).count();
    let tasa_dudo = if dudas == 0 { a_ciegas } else { aciertos_dudo as f64 / dudas as f64 };
    let tasa = (no_sabes as f64 * tasa_dudo + sin_ver as f64 * a_ciegas) / (no_sabes + sin_ver).max(1) as f64;
    let p = |s: u32, acierto| probabilidad(s as f64 / total, acierto, objetivo, examen);
    let mut meta = sabes;
    while meta < n && p(meta, None).max(p(meta, Some(tasa))) < META {
        meta += 1;
    }
    let faltan = meta - sabes;
    // El ritmo: los conceptos que pasaron a "sabes" en las últimas respuestas, por sesión.
    let mut ordenado = historial.to_vec();
    ordenado.sort_by(|a, b| a.ts.cmp(&b.ts));
    let antes = cuenta(banco, &ordenado[..ordenado.len().saturating_sub(VENTANA)]).0;
    let sesiones_de_la_ventana =
        if historial.is_empty() { 1.0 } else { historial.len().min(VENTANA) as f64 / TAMANO_SESION as f64 };
    let ritmo = (sabes as f64 - antes as f64) / sesiones_de_la_ventana;
    let (blanco, contesta) = (p(sabes, None), p(sabes, Some(tasa)));
    let pc = |x: f64| (100.0 * x).round() as u32;
    let ciegas = format!("a ciegas, 1 de {}", examen.opciones);
    let fila = |texto: &str, conceptos, tasa, detalle| Dudas { texto: texto.into(), conceptos, porcentaje: pc(tasa), detalle };
    Tarjeta {
        sabes,
        no_sabes,
        sin_ver,
        hoy: pc(blanco.max(contesta)),
        meta,
        sesiones: if faltan == 0 {
            "✓".into()
        } else if ritmo > 0.0 {
            match (faltan as f64 / ritmo).ceil() {
                1.0 => "≈1 sesión".into(),
                n => format!("≈{n} sesiones"),
            }
        } else {
            "? sesiones".into()
        },
        consejo: if contesta > blanco { "Contesta las dudas" } else { "Deja las dudas en blanco" }.into(),
        si_contestas: pc(contesta),
        en_blanco: pc(blanco),
        dudas: vec![
            fila(
                "en lo que viste",
                no_sabes,
                tasa_dudo,
                Some(if dudas == 0 { ciegas.clone() } else { format!("{aciertos_dudo} de {dudas} «dudo»") }),
            ),
            fila("en lo que no viste", sin_ver, a_ciegas, Some(ciegas)),
            fila("de media", no_sabes + sin_ver, tasa, None),
        ],
    }
}

const META: f64 = 0.95;
/// Las respuestas que cuentan para el ritmo: 5 sesiones de 12.
const VENTANA: usize = 60;

/// Los conceptos en "sabes", en "no sabes" y sin ver.
fn cuenta(banco: &[Pregunta], historial: &[Respuesta]) -> (u32, u32, u32) {
    let estado = estados(banco, historial);
    let conceptos: std::collections::HashSet<&str> = banco.iter().map(|q| q.concepto.as_str()).collect();
    let sabes = estado.values().filter(|e| matches!(e.0, Concepto::Paso(n) if n > 0)).count();
    (sabes as u32, (estado.len() - sabes) as u32, (conceptos.len() - estado.len()) as u32)
}

fn binomial(n: u32, k: u32, p: f64) -> f64 {
    let c = (0..k).fold(1.0, |c, i| c * (n - i) as f64 / (i + 1) as f64);
    c * p.powi(k as i32) * (1.0 - p).powi((n - k) as i32)
}

/// La probabilidad de sacar más de `objetivo` si cada pregunta es de un concepto en "sabes" con probabilidad `sabes`.
/// Una pregunta de otro concepto acierta con probabilidad `acierto`, o queda en blanco con `None`.
/// El objetivo pide `objetivo × preguntas / 10` aciertos netos, y un fallo resta la penalización.
fn probabilidad(sabes: f64, acierto: Option<f64>, objetivo: f64, examen: &Examen) -> f64 {
    let n = examen.preguntas;
    let netos = objetivo * n as f64 / 10.0;
    let mut total = 0.0;
    for k in 0..=n {
        let resto = n - k;
        match acierto {
            None if k as f64 >= netos => total += binomial(n, k, sabes),
            None => {}
            Some(p) => {
                for a in 0..=resto {
                    if (k + a) as f64 - examen.penalizacion * (resto - a) as f64 >= netos {
                        total += binomial(n, k, sabes) * binomial(resto, a, p);
                    }
                }
            }
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    const LN: Examen = Examen { titulo: String::new(), preguntas: 20, opciones: 4, minutos: 25, penalizacion: 0.25 };

    /// Un banco de `n` conceptos `c0`, `c1`…, con una pregunta `q0`, `q1`… cada uno.
    fn banco_de(n: usize) -> Vec<Pregunta> {
        (0..n).map(|i| pregunta(&format!("q{i}"), &format!("c{i}"))).collect()
    }

    /// Una respuesta por minuto, de `(pregunta, acierto, seguro)`.
    fn historial_de(respuestas: &[(usize, bool, bool)]) -> Vec<Respuesta> {
        respuestas
            .iter()
            .enumerate()
            .map(|(k, &(i, acierto, seguro))| {
                let ts = format!("1970-01-01T{:02}:{:02}:00Z", k / 60, k % 60);
                respuesta(&format!("q{i}"), &ts, acierto, seguro)
            })
            .collect()
    }

    fn cerca(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} ≠ {b}");
    }

    #[test]
    fn la_probabilidad_de_una_nota_cuenta_los_aciertos_menos_la_penalizacion_de_los_fallos() {
        cerca(probabilidad(1.0, None, 5.0, &LN), 1.0);
        cerca(probabilidad(0.0, None, 5.0, &LN), 0.0);
        cerca(probabilidad(0.5, None, 5.0, &LN), 0.5880985260009766);
        cerca(probabilidad(0.5, Some(0.25), 5.0, &LN), 0.6828711582842681);
        cerca(probabilidad(0.0, Some(0.25), 5.0, &LN), 0.000935391579332645);
        cerca(probabilidad(0.5, None, 7.5, &LN), 0.020694732666015625);
        cerca(probabilidad(0.5, Some(0.25), 7.5, &LN), 0.07904140886862707);
        cerca(probabilidad(0.9, Some(0.25), 10.0, &LN), 0.21029776386659405);
    }

    #[test]
    fn una_duda_acierta_con_el_porcentaje_de_dudo_en_lo_que_viste_y_a_ciegas_en_lo_que_no() {
        let h = historial_de(&[(0, true, true), (1, false, false), (2, true, false)]);
        let t = tarjeta(&LN, &banco_de(10), &h, 5.0);
        assert_eq!((t.sabes, t.no_sabes, t.sin_ver), (1, 2, 7));
        assert_eq!(t.dudas[0].detalle.as_deref(), Some("1 de 2 «dudo»"));
        // 2 conceptos al 50 % y 7 al 25 %: 11/36.
        assert_eq!([0, 1, 2].map(|i| t.dudas[i].porcentaje), [50, 25, 31]);
    }

    #[test]
    fn la_tarjeta_sigue_la_forma_con_la_probabilidad_mas_alta_y_el_consejo_da_las_dos() {
        // 10 conceptos que sabes y 10 con un fallo con dudo: contestar solo resta, y una duda acierta 0 %.
        let respuestas: Vec<_> = (0..10).map(|i| (i, true, true)).chain((10..20).map(|i| (i, false, false))).collect();
        let t = tarjeta(&LN, &banco_de(20), &historial_de(&respuestas), 5.0);
        assert_eq!((t.hoy, t.si_contestas, t.en_blanco), (59, 25, 59));
        assert_eq!(t.consejo, "Deja las dudas en blanco");
    }

    #[test]
    fn el_camino_cuenta_los_conceptos_para_un_95_y_las_sesiones_salen_del_ritmo() {
        // Para un 10 con 20 preguntas solo valen 20 aciertos, así que el 95 % pide todos los conceptos.
        // 4 conceptos nuevos en 4 respuestas son 12 por sesión, y faltan 16.
        let h = historial_de(&[(0, true, true), (1, true, true), (2, true, true), (3, true, true)]);
        let t = tarjeta(&LN, &banco_de(20), &h, 10.0);
        assert_eq!((t.sabes, t.meta), (4, 20));
        assert_eq!(t.sesiones, "≈2 sesiones");
    }

    #[test]
    fn una_sola_sesion_va_en_singular() {
        // 19 conceptos nuevos en 19 respuestas son 12 por sesión, y falta 1.
        let respuestas: Vec<_> = (0..19).map(|i| (i, true, true)).collect();
        let t = tarjeta(&LN, &banco_de(20), &historial_de(&respuestas), 10.0);
        assert_eq!(t.sesiones, "≈1 sesión");
    }

    #[test]
    fn el_ritmo_cuenta_solo_las_60_ultimas_respuestas() {
        // 10 conceptos nuevos en las 60 últimas respuestas son 2 por sesión, y faltan 50.
        let respuestas: Vec<_> = (0..50).map(|i| (i, true, true)).chain((0..50).map(|_| (0, true, true))).collect();
        let t = tarjeta(&LN, &banco_de(100), &historial_de(&respuestas), 10.0);
        assert_eq!((t.sabes, t.meta), (50, 100));
        assert_eq!(t.sesiones, "≈25 sesiones");
    }

    #[test]
    fn con_un_95_hoy_no_faltan_conceptos_ni_sesiones() {
        let respuestas: Vec<_> = (0..20).map(|i| (i, true, true)).collect();
        let t = tarjeta(&LN, &banco_de(20), &historial_de(&respuestas), 5.0);
        assert_eq!((t.sabes, t.meta, t.hoy), (20, 20, 100));
        assert_eq!(t.sesiones, "✓");
    }

    #[test]
    fn sin_ritmo_las_sesiones_no_se_pueden_calcular() {
        let t = tarjeta(&LN, &banco_de(20), &historial_de(&[(0, false, true)]), 10.0);
        assert_eq!((t.sabes, t.meta), (0, 20));
        assert_eq!(t.sesiones, "? sesiones");
    }

    #[test]
    fn sabes_es_un_paso_de_1_o_mas() {
        // c0 sube a paso 2, c1 vuelve a paso 0 tras un acierto con dudo, y una respuesta huérfana no cuenta.
        let mut h = historial_de(&[(0, true, true), (0, true, true), (1, true, true), (1, true, false)]);
        h.push(respuesta("x", "1970-01-01T01:00:00Z", true, true));
        let t = tarjeta(&LN, &banco_de(3), &h, 5.0);
        assert_eq!((t.sabes, t.no_sabes, t.sin_ver), (1, 1, 1));
    }

    #[test]
    fn un_examen_de_3_opciones_sin_penalizacion_aconseja_contestar() {
        let asn = Examen { titulo: String::new(), preguntas: 35, opciones: 3, minutos: 45, penalizacion: 0.0 };
        let t = tarjeta(&asn, &banco_de(20), &[], 5.0);
        assert_eq!(t.dudas[2].porcentaje, 33);
        assert_eq!(t.dudas[1].detalle.as_deref(), Some("a ciegas, 1 de 3"));
        assert_eq!(t.consejo, "Contesta las dudas");
    }

    #[test]
    fn la_tarjeta_sin_historial_acierta_una_duda_a_ciegas_y_no_tiene_ritmo() {
        let t = tarjeta(&LN, &banco_de(20), &[], 5.0);
        assert_eq!((t.sabes, t.no_sabes, t.sin_ver, t.hoy, t.si_contestas, t.en_blanco), (0, 0, 20, 0, 0, 0));
        assert_eq!(t.consejo, "Contesta las dudas");
        assert_eq!(t.sesiones, "? sesiones");
        let a_ciegas = Some("a ciegas, 1 de 4".to_string());
        assert_eq!(
            t.dudas,
            [
                Dudas { texto: "en lo que viste".into(), conceptos: 0, porcentaje: 25, detalle: a_ciegas.clone() },
                Dudas { texto: "en lo que no viste".into(), conceptos: 20, porcentaje: 25, detalle: a_ciegas },
                Dudas { texto: "de media".into(), conceptos: 20, porcentaje: 25, detalle: None },
            ]
        );
    }

    fn cierre(preguntas: u32, resultados: &[Resultado]) -> Cierre {
        Cierre { preguntas, resultados: resultados.to_vec() }
    }

    #[test]
    fn la_nota_suma_un_acierto_resta_la_penalizacion_de_un_fallo_y_no_cuenta_una_en_blanco() {
        use Resultado::*;
        let n = nota_del_simulacro(&cierre(4, &[Acierto, Acierto, Fallo, Blanco]), 0.25);
        // (2 − 0,25) / 4 × 10 = 4,375.
        assert_eq!((n.aciertos, n.fallos, n.en_blanco, n.nota.as_str()), (2, 1, 1, "4,38"));
        assert_eq!(nota_del_simulacro(&cierre(2, &[Acierto, Acierto]), 0.25).nota, "10");
        assert_eq!(nota_del_simulacro(&cierre(2, &[Fallo, Blanco]), 0.25).nota, "-1,25");
        assert_eq!(nota_del_simulacro(&cierre(3, &[Acierto, Fallo]), 0.0).nota, "3,33");
    }

    #[test]
    fn el_cierre_al_pasar_el_limite_cuenta_en_blanco_las_preguntas_sin_contestar() {
        let n = nota_del_simulacro(&cierre(20, &[Resultado::Acierto, Resultado::Fallo]), 0.25);
        assert_eq!((n.aciertos, n.fallos, n.en_blanco, n.nota.as_str()), (1, 1, 18, "0,38"));
        assert_eq!(nota_del_simulacro(&cierre(20, &[]), 0.25).en_blanco, 20);
    }

    #[test]
    fn el_simulacro_saca_al_azar_las_preguntas_del_examen_de_todo_el_banco_con_su_reloj() {
        // 3 preguntas por concepto: el simulacro no lleva una sola por concepto.
        let banco: Vec<Pregunta> = (0..30).map(|i| pregunta(&format!("q{i}"), &format!("c{}", i / 3))).collect();
        let s = simulacro(&LN, &banco, &[], &mut Azar::new(1));
        assert_eq!((s.segundos, s.preguntas.len()), (25 * 60, 20));
        let mut ids: Vec<&str> = s.preguntas.iter().map(|q| q.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 20);
        let otro = simulacro(&LN, &banco, &[], &mut Azar::new(2));
        assert_ne!(
            s.preguntas.iter().map(|q| &q.id).collect::<Vec<_>>(),
            otro.preguntas.iter().map(|q| &q.id).collect::<Vec<_>>()
        );
        assert_eq!(simulacro(&LN, &banco[..5], &[], &mut Azar::new(1)).preguntas.len(), 5);
    }

    #[test]
    fn una_pregunta_del_simulacro_da_el_paso_de_su_concepto() {
        let banco = [pregunta("a1", "a"), pregunta("a2", "a")];
        let historial = [respuesta("a1", &minuto(0, 0), true, true)];
        let s = simulacro(&LN, &banco, &historial, &mut Azar::new(1));
        assert!(s.preguntas.iter().all(|q| q.paso == "paso 1" && q.paso_tras_otra == "paso 0"));
    }

    pub(crate) fn pregunta(id: &str, concepto: &str) -> Pregunta {
        Pregunta {
            id: id.into(),
            concepto: concepto.into(),
            tema: "rgpd".into(),
            pregunta: format!("¿{id}?"),
            correcta: "sí".into(),
            distractores: vec!["no".into(), "a veces".into(), "nunca".into()],
            explicacion: "porque sí".into(),
        }
    }

    pub(crate) fn respuesta(id: &str, ts: &str, acierto: bool, seguro: bool) -> Respuesta {
        Respuesta { id: id.into(), ts: ts.into(), acierto, seguro }
    }

    #[test]
    fn el_inicio_da_el_titulo_y_las_preguntas_de_la_sesion() {
        let banco = [pregunta("a1", "a"), pregunta("a2", "a"), pregunta("b1", "b")];
        let examen = Examen { titulo: "LN".into(), ..LN };
        let Inicio::Listo { titulo, sesion, simulacro, .. } = inicio(&examen, &banco, &[], 5.0) else { panic!() };
        assert_eq!((titulo.as_str(), sesion, simulacro), ("LN", 2, 3));
        assert_eq!(inicio(&examen, &[], &[], 5.0), Inicio::BancoVacio { titulo: "LN".into() });
    }

    #[test]
    fn la_hora_pasa_a_segundos() {
        assert_eq!(segundos("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(segundos("2026-11-04T12:41:22Z"), Some(1_793_796_082));
        assert_eq!(segundos("2024-02-29T00:00:00Z"), Some(1_709_164_800));
        assert_eq!(segundos("2026-11-04 12:41:22Z"), None);
        assert_eq!(segundos("2026-13-04T12:41:22Z"), None);
        assert_eq!(segundos("hoy"), None);
    }

    /// El paso del concepto de la única pregunta del banco, como lo muestra la sesión.
    fn paso(banco: &[Pregunta], historial: &[Respuesta]) -> String {
        sesion(banco, historial, 0, &mut Azar::new(1)).remove(0).paso
    }

    #[test]
    fn un_acierto_con_seguro_sube_el_paso_aunque_no_haya_vencido() {
        let banco = [pregunta("a1", "a")];
        let historial =
            [respuesta("a1", "2026-11-04T12:00:00Z", true, true), respuesta("a1", "2026-11-04T12:01:00Z", true, true)];
        assert_eq!(paso(&banco, &historial), "paso 2");
        assert_eq!(paso(&banco, &[]), "sin ver");
    }

    #[test]
    fn cualquier_otra_respuesta_pone_el_paso_a_0() {
        let banco = [pregunta("a1", "a")];
        for (acierto, seguro) in [(true, false), (false, true), (false, false)] {
            let historial =
                [respuesta("a1", "2026-11-04T12:00:00Z", true, true), respuesta("a1", "2026-11-04T12:01:00Z", acierto, seguro)];
            assert_eq!(paso(&banco, &historial), "paso 0", "{acierto} {seguro}");
        }
    }

    #[test]
    fn los_huecos_son_10_30_y_90_minutos() {
        let minutos = |c: Concepto| c.hueco() / 60;
        assert_eq!([0, 1, 2, 3].map(|n| minutos(Concepto::Paso(n))), [0, 10, 30, 90]);
        assert_eq!(Concepto::Paso(u32::MAX).hueco(), i64::MAX);
    }
    fn minuto(m: i64, s: i64) -> String {
        format!("2026-11-04T12:{m:02}:{s:02}Z")
    }

    const AHORA: &str = "2026-11-04T13:00:00Z";

    /// Un banco de `n` conceptos `<prefijo>0`, `<prefijo>1`…, con una pregunta cada uno.
    fn conceptos(prefijo: &str, n: usize) -> Vec<Pregunta> {
        (0..n).map(|i| pregunta(&format!("{prefijo}{i}-1"), &format!("{prefijo}{i}"))).collect()
    }

    fn de_la_sesion(banco: &[Pregunta], historial: &[Respuesta]) -> Vec<String> {
        let mut ids: Vec<String> =
            sesion(banco, historial, segundos(AHORA).unwrap(), &mut Azar::new(1)).into_iter().map(|q| q.id).collect();
        ids.sort();
        ids
    }

    fn ids(banco: &[Pregunta]) -> Vec<String> {
        let mut ids: Vec<String> = banco.iter().map(|q| q.id.clone()).collect();
        ids.sort();
        ids
    }

    /// Un fallo en cada pregunta, una por minuto: cada concepto vence en el acto.
    fn fallos(banco: &[Pregunta]) -> Vec<Respuesta> {
        banco.iter().enumerate().map(|(i, q)| respuesta(&q.id, &minuto(i as i64, 0), false, false)).collect()
    }

    #[test]
    fn la_sesion_saca_primero_los_vencidos_hace_mas_tiempo() {
        let banco = conceptos("v", 15);
        assert_eq!(de_la_sesion(&banco, &fallos(&banco)), ids(&banco[..12]));
    }

    #[test]
    fn la_sesion_guarda_4_huecos_para_los_conceptos_sin_ver() {
        let vencidos = conceptos("v", 20);
        let sin_ver = conceptos("s", 10);
        let sesion = de_la_sesion(&[vencidos.clone(), sin_ver].concat(), &fallos(&vencidos));
        assert_eq!(sesion.iter().filter(|id| id.starts_with('s')).count(), 4);
        assert_eq!(sesion.iter().filter(|id| id.starts_with('v')).cloned().collect::<Vec<_>>(), ids(&vencidos[..8]));
    }

    #[test]
    fn los_huecos_que_sobran_se_llenan_con_sin_ver_y_despues_con_los_que_vencen_antes() {
        let vencidos = conceptos("v", 2);
        let sin_ver = conceptos("s", 2);
        let al_dia = conceptos("d", 20);
        let mut historial = fallos(&vencidos);
        historial.extend(al_dia.iter().enumerate().map(|(i, q)| respuesta(&q.id, &minuto(30, i as i64), true, true)));
        let banco = [vencidos.clone(), sin_ver.clone(), al_dia.clone()].concat();
        assert_eq!(de_la_sesion(&banco, &historial), ids(&[vencidos, sin_ver, al_dia[..8].to_vec()].concat()));
    }

    #[test]
    fn la_sesion_lleva_una_pregunta_por_concepto() {
        let banco = [pregunta("a1", "a"), pregunta("a2", "a"), pregunta("a3", "a"), pregunta("b1", "b")];
        let sesion = sesion(&banco, &[], 0, &mut Azar::new(1));
        let mut conceptos: Vec<&str> = sesion.iter().map(|q| q.concepto.as_str()).collect();
        conceptos.sort();
        assert_eq!(conceptos, ["a", "b"]);
    }

    #[test]
    fn de_cada_concepto_sale_la_pregunta_sin_respuestas_y_despues_la_de_la_respuesta_mas_vieja() {
        let banco = [pregunta("a1", "a"), pregunta("a2", "a"), pregunta("a3", "a")];
        let viejas = [respuesta("a1", &minuto(2, 0), false, false), respuesta("a3", &minuto(1, 0), false, false)];
        for semilla in 0..20 {
            let mut azar = Azar::new(semilla);
            assert_eq!(sesion(&banco, &viejas, 0, &mut azar)[0].id, "a2");
            let todas = [&viejas[..], &[respuesta("a2", &minuto(3, 0), false, false)]].concat();
            assert_eq!(sesion(&banco, &todas, 0, &mut azar)[0].id, "a3");
        }
    }

    #[test]
    fn con_300_conceptos_acertados_en_el_mismo_minuto_salen_los_12_que_vencen_antes() {
        let banco = conceptos("c", 300);
        let historial: Vec<Respuesta> =
            banco.iter().enumerate().map(|(i, q)| respuesta(&q.id, &minuto(0, i as i64 / 5), true, true)).collect();
        let ahora = segundos(&minuto(5, 0)).unwrap();
        let sesion = sesion(&banco, &historial, ahora, &mut Azar::new(1));
        let mut indices: Vec<usize> = sesion.iter().map(|q| q.concepto[1..].parse().unwrap()).collect();
        indices.sort();
        assert_eq!(indices.len(), 12);
        assert_eq!(indices[..10], (0..10).collect::<Vec<_>>()[..]);
        assert!(indices[10..].iter().all(|&i| i < 15), "{indices:?}");
    }

    #[test]
    fn la_misma_semilla_da_siempre_el_mismo_orden() {
        let banco = conceptos("c", 30);
        let orden =
            |semilla| sesion(&banco, &[], 0, &mut Azar::new(semilla)).into_iter().map(|q| (q.id, q.opciones)).collect::<Vec<_>>();
        assert_eq!(orden(7), orden(7));
        assert_ne!(orden(7), orden(8));
    }

    #[test]
    fn la_pregunta_da_las_opciones_al_azar_la_correcta_y_el_paso() {
        let banco = [pregunta("a1", "a")];
        let historial = [respuesta("a1", &minuto(0, 0), true, true)];
        let q = &sesion(&banco, &historial, 0, &mut Azar::new(3))[0];
        assert_eq!(q.opciones[q.correcta as usize], "sí");
        let mut opciones = q.opciones.clone();
        opciones.sort();
        assert_eq!(opciones, ["a veces", "no", "nunca", "sí"]);
        assert_eq!((q.paso.as_str(), q.paso_tras_seguro.as_str(), q.paso_tras_otra.as_str()), ("paso 1", "paso 2", "paso 0"));
        let sin_ver = &sesion(&[pregunta("b1", "b")], &[], 0, &mut Azar::new(3))[0];
        assert_eq!((sin_ver.paso.as_str(), sin_ver.paso_tras_seguro.as_str()), ("sin ver", "paso 1"));
    }
}
