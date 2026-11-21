//! La lógica de la app. `nucleo` no lee archivos ni la red.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Las cifras de un examen, de `examen.json`.
#[derive(Deserialize)]
pub struct Examen {
    pub titulo: String,
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
    /// `sesion` es el número de preguntas de la sesión.
    Listo {
        titulo: String,
        sesion: u32,
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

pub fn inicio(examen: &Examen, banco: &[Pregunta]) -> Inicio {
    let titulo = examen.titulo.clone();
    if banco.is_empty() {
        return Inicio::BancoVacio { titulo };
    }
    let conceptos: std::collections::HashSet<&str> = banco.iter().map(|q| q.concepto.as_str()).collect();
    Inicio::Listo { titulo, sesion: conceptos.len().min(TAMANO_SESION) as u32 }
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
    elegidos
        .into_iter()
        .map(|c| {
            let q = pregunta[c];
            let paso = estado.get(c).map_or(Concepto::SinVer, |e| e.0);
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
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let examen = Examen { titulo: "LN".into() };
        assert_eq!(inicio(&examen, &banco), Inicio::Listo { titulo: "LN".into(), sesion: 2 });
        assert_eq!(inicio(&examen, &[]), Inicio::BancoVacio { titulo: "LN".into() });
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
