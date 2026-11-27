//! La sesión de desarrollo, con su historial `desarrollo.jsonl` y su estado de FSRS, aparte del test (ADR 0006).

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use tsify::Tsify;

use crate::{Azar, Concepto, Estado, Examen, OTRA_SESION, Pagina, Par, PreguntaDeDesarrollo, Resultado, en_porcentaje};

/// La nota de FSRS de una respuesta de desarrollo: "No la sabía", "A medias" o "La sabía".
#[derive(Deserialize, Serialize, Tsify, Clone, Copy, PartialEq, Debug)]
pub enum NotaDeDesarrollo {
    Again,
    Hard,
    Good,
}

/// Una línea de `desarrollo.jsonl`: una respuesta con su texto, o la corrección de la respuesta de `id` y `ts`.
#[derive(Deserialize, Serialize, Tsify, Clone, PartialEq, Debug)]
#[serde(untagged)]
pub enum LineaDeDesarrollo {
    Respuesta(RespuestaDeDesarrollo),
    Correccion(CorreccionDeDesarrollo),
}

#[derive(Deserialize, Serialize, Tsify, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct RespuestaDeDesarrollo {
    pub id: String,
    #[tsify(type = "string")]
    pub ts: Timestamp,
    pub texto: String,
}

/// `ts` es el de su respuesta. Una respuesta con el cuadro vacío no tiene línea de respuesta, solo su corrección.
#[derive(Deserialize, Serialize, Tsify, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct CorreccionDeDesarrollo {
    pub id: String,
    #[tsify(type = "string")]
    pub ts: Timestamp,
    pub nota: NotaDeDesarrollo,
    #[tsify(type = "string")]
    pub corregida: Timestamp,
}

impl LineaDeDesarrollo {
    /// La clave de unión: el `id` y el `ts` de una respuesta, y además `corregida` en una corrección.
    pub fn clave(&self) -> (&str, Timestamp, Option<Timestamp>) {
        match self {
            LineaDeDesarrollo::Respuesta(r) => (&r.id, r.ts, None),
            LineaDeDesarrollo::Correccion(c) => (&c.id, c.ts, Some(c.corregida)),
        }
    }

    fn ts(&self) -> Timestamp {
        match self {
            LineaDeDesarrollo::Respuesta(r) => r.ts,
            LineaDeDesarrollo::Correccion(c) => c.ts,
        }
    }
}

/// Las líneas de una respuesta corregida en `ahora`. Un texto vacío no lleva línea de respuesta.
pub fn calificar(id: &str, texto: &str, nota: NotaDeDesarrollo, ahora: Timestamp) -> Vec<LineaDeDesarrollo> {
    let respuesta = (!texto.trim().is_empty())
        .then(|| LineaDeDesarrollo::Respuesta(RespuestaDeDesarrollo { id: id.into(), ts: ahora, texto: texto.into() }));
    let correccion = LineaDeDesarrollo::Correccion(CorreccionDeDesarrollo { id: id.into(), ts: ahora, nota, corregida: ahora });
    respuesta.into_iter().chain([correccion]).collect()
}

/// Las líneas de 2 historiales de desarrollo, en orden de hora. Una respuesta con el mismo `id` y `ts` entra una vez,
/// y una corrección con el mismo `id`, `ts` y `corregida` también.
pub fn unir_desarrollo(a: &[LineaDeDesarrollo], b: &[LineaDeDesarrollo]) -> Vec<LineaDeDesarrollo> {
    let mut vistas = std::collections::HashSet::new();
    let mut u: Vec<LineaDeDesarrollo> = a.iter().chain(b).filter(|l| vistas.insert(l.clave())).cloned().collect();
    u.sort_by_key(LineaDeDesarrollo::ts);
    u
}

type Estados<'a> = std::collections::HashMap<&'a str, Estado>;

/// El estado de cada pregunta corregida del banco, por su `id`. De cada respuesta cuenta su corrección más nueva,
/// a la hora de la respuesta. Again es una fallada, Hard un acierto con duda y Good un acierto seguro.
fn estados<'a>(desarrollo: &'a [PreguntaDeDesarrollo], historial: &[LineaDeDesarrollo]) -> Estados<'a> {
    let mut nota: std::collections::BTreeMap<(Timestamp, &str), &CorreccionDeDesarrollo> = Default::default();
    for l in historial {
        if let LineaDeDesarrollo::Correccion(c) = l {
            let k = (c.ts, c.id.as_str());
            if nota.get(&k).is_none_or(|v| c.corregida > v.corregida) {
                nota.insert(k, c);
            }
        }
    }
    let mut estado = Estados::new();
    for ((ts, id), c) in nota {
        let Some(q) = desarrollo.iter().find(|q| q.id == id) else {
            continue;
        };
        let (resultado, seguro) = como_respuesta(c.nota);
        let despues = Estado::tras(estado.get(q.id.as_str()).copied(), resultado, seguro, ts.as_second());
        estado.insert(q.id.as_str(), despues);
    }
    estado
}

/// La nota como una respuesta de test: Again es un fallo, Hard un acierto con "dudo", y Good un acierto con "seguro".
fn como_respuesta(nota: NotaDeDesarrollo) -> (Resultado, bool) {
    match nota {
        NotaDeDesarrollo::Again => (Resultado::Fallo, false),
        NotaDeDesarrollo::Hard => (Resultado::Acierto, false),
        NotaDeDesarrollo::Good => (Resultado::Acierto, true),
    }
}

/// Un punto de la solución: su concepto, la idea y su página.
#[derive(Serialize, Deserialize, Tsify, Clone, PartialEq, Debug)]
pub struct Punto {
    pub concepto: String,
    pub idea: String,
    pub pagina: Option<Pagina>,
}

/// Una pregunta de la sesión de desarrollo, con lo que muestran la pregunta y su solución.
#[derive(Serialize, Deserialize, Tsify, Clone, PartialEq, Debug)]
pub struct PreguntaDeDesarrolloDeSesion {
    pub id: String,
    /// El nombre del apartado.
    pub apartado: String,
    pub enunciado: String,
    pub modelo: String,
    pub puntos: Vec<Punto>,
    /// La probabilidad de acierto antes de contestar, como `62 %`, o `sin ver`.
    pub probabilidad: String,
    /// Cuándo vuelve la pregunta con su estado de ahora, como `vuelve en 3 días`.
    pub vuelve: String,
    pub vuelve_tras_again: String,
    pub vuelve_tras_hard: String,
    pub vuelve_tras_good: String,
}

/// El PDF de cada concepto, por su `id`.
fn pdfs(conceptos: &[Concepto]) -> std::collections::HashMap<&str, &str> {
    conceptos.iter().map(|c| (c.id.as_str(), c.fuente.pdf.as_str())).collect()
}

/// Si la pregunta está en el apartado `apartado` del tema `pdf`: es de ese apartado, y algún punto sale de ese PDF.
fn en(
    examen: &Examen,
    pdfs: &std::collections::HashMap<&str, &str>,
    q: &PreguntaDeDesarrollo,
    pdf: &str,
    apartado: &str,
) -> bool {
    q.apartado == apartado && q.puntos.iter().any(|c| pdfs.get(c.as_str()) == Some(&pdf)) && examen.pdfs.contains_key(pdf)
}

/// Las preguntas de desarrollo de los pares marcados. Sin selección, todo el banco.
pub fn desarrollo_de_la_seleccion(
    examen: &Examen,
    desarrollo: &[PreguntaDeDesarrollo],
    conceptos: &[Concepto],
    seleccion: Option<&[Par]>,
) -> Vec<PreguntaDeDesarrollo> {
    let pdfs = pdfs(conceptos);
    desarrollo
        .iter()
        .filter(|q| seleccion.is_none_or(|s| s.iter().any(|p| en(examen, &pdfs, q, &p.pdf, &p.apartado))))
        .cloned()
        .collect()
}

/// Las preguntas de desarrollo que sabes y las que hay, para el inicio.
#[derive(Serialize, Tsify, PartialEq, Debug)]
pub struct CifraDeDesarrollo {
    /// Las preguntas de la sesión de desarrollo de la selección.
    pub sesion: u32,
    /// Las preguntas de desarrollo del simulacro de la selección.
    pub simulacro: u32,
    /// Las respuestas sin corregir del examen entero, de cualquier selección.
    pub sin_corregir: u32,
    /// Las preguntas de la selección vistas y no vencidas, como el "sabes" del test.
    pub sabes: u32,
    pub hay: u32,
    /// La cuenta de cada apartado de cada tema, de todo el banco.
    pub pares: Vec<CuentaDelPar>,
}

#[derive(Serialize, Tsify, PartialEq, Debug)]
pub struct CuentaDelPar {
    pub pdf: String,
    pub apartado: String,
    pub sabes: u32,
    pub hay: u32,
}

/// La cifra de la selección en `ahora`, en segundos desde 1970. Sin banco de desarrollo, no hay cifra.
pub fn cifra_de_desarrollo(
    examen: &Examen,
    desarrollo: &[PreguntaDeDesarrollo],
    conceptos: &[Concepto],
    historial: &[LineaDeDesarrollo],
    seleccion: Option<&[Par]>,
    ahora: i64,
) -> Option<CifraDeDesarrollo> {
    if desarrollo.is_empty() {
        return None;
    }
    let estado = estados(desarrollo, historial);
    let sabe = |q: &PreguntaDeDesarrollo| estado.get(q.id.as_str()).is_some_and(|e| !e.vencida(ahora, examen.retencion));
    let elegidas = desarrollo_de_la_seleccion(examen, desarrollo, conceptos, seleccion);
    let pdfs = pdfs(conceptos);
    let pares = examen
        .pdfs
        .iter()
        .flat_map(|(pdf, tema)| tema.apartados.iter().map(move |a| (pdf, a)))
        .map(|(pdf, a)| {
            let del_par: Vec<&PreguntaDeDesarrollo> = desarrollo.iter().filter(|q| en(examen, &pdfs, q, pdf, a)).collect();
            CuentaDelPar {
                pdf: pdf.clone(),
                apartado: a.clone(),
                sabes: del_par.iter().filter(|q| sabe(q)).count() as u32,
                hay: del_par.len() as u32,
            }
        })
        .collect();
    Some(CifraDeDesarrollo {
        sesion: TAMANO.min(elegidas.len()) as u32,
        simulacro: examen.desarrollo.min(elegidas.len() as u32),
        sin_corregir: sin_corregir(historial).len() as u32,
        sabes: elegidas.iter().filter(|q| sabe(q)).count() as u32,
        hay: elegidas.len() as u32,
        pares,
    })
}

/// Las líneas de una respuesta del simulacro, dada en `ahora`. Un texto vacío lleva su corrección Again, y un texto
/// con contenido queda sin corregir.
pub fn responder(id: &str, texto: &str, ahora: Timestamp) -> Vec<LineaDeDesarrollo> {
    if texto.trim().is_empty() {
        return calificar(id, texto, NotaDeDesarrollo::Again, ahora);
    }
    vec![LineaDeDesarrollo::Respuesta(RespuestaDeDesarrollo { id: id.into(), ts: ahora, texto: texto.into() })]
}

/// La corrección en `ahora` de la respuesta de `id` y `ts`.
pub fn corregir_respuesta(id: &str, ts: Timestamp, nota: NotaDeDesarrollo, ahora: Timestamp) -> LineaDeDesarrollo {
    LineaDeDesarrollo::Correccion(CorreccionDeDesarrollo { id: id.into(), ts, nota, corregida: ahora })
}

/// Las respuestas sin ninguna corrección, en orden de hora.
pub fn sin_corregir(historial: &[LineaDeDesarrollo]) -> Vec<&RespuestaDeDesarrollo> {
    let corregidas: std::collections::HashSet<(&str, Timestamp)> = historial
        .iter()
        .filter_map(|l| match l {
            LineaDeDesarrollo::Correccion(c) => Some((c.id.as_str(), c.ts)),
            LineaDeDesarrollo::Respuesta(_) => None,
        })
        .collect();
    let mut r: Vec<&RespuestaDeDesarrollo> = historial
        .iter()
        .filter_map(|l| match l {
            LineaDeDesarrollo::Respuesta(r) if !corregidas.contains(&(r.id.as_str(), r.ts)) => Some(r),
            _ => None,
        })
        .collect();
    r.sort_by_key(|r| r.ts);
    r
}

/// `n` preguntas de desarrollo al azar, para el simulacro.
pub(crate) fn al_azar(
    examen: &Examen,
    desarrollo: &[PreguntaDeDesarrollo],
    conceptos: &[Concepto],
    historial: &[LineaDeDesarrollo],
    n: usize,
    ahora: i64,
    azar: &mut Azar,
) -> Vec<PreguntaDeDesarrolloDeSesion> {
    let estado = estados(desarrollo, historial);
    let mut orden: Vec<&PreguntaDeDesarrollo> = desarrollo.iter().collect();
    azar.barajar(&mut orden);
    orden.into_iter().take(n).map(|q| de_la_ronda(examen, conceptos, q, &estado, ahora)).collect()
}

const TAMANO: usize = 4;

/// La sesión de 4: las falladas, del fallo más viejo al más nuevo, las demás vencidas, las sin ver, y las no vencidas.
/// Las vencidas y las no vencidas van de la menor probabilidad de acierto a la mayor, y el azar rompe los empates.
/// `ahora` va en segundos desde 1970.
pub fn sesion_de_desarrollo(
    examen: &Examen,
    desarrollo: &[PreguntaDeDesarrollo],
    conceptos: &[Concepto],
    historial: &[LineaDeDesarrollo],
    ahora: i64,
    azar: &mut Azar,
) -> Vec<PreguntaDeDesarrolloDeSesion> {
    let estado = estados(desarrollo, historial);
    let mut orden: Vec<&PreguntaDeDesarrollo> = desarrollo.iter().collect();
    azar.barajar(&mut orden);
    let clave = |q: &PreguntaDeDesarrollo| match estado.get(q.id.as_str()) {
        Some(e) if e.fallada => (0, e.ultima as f64),
        Some(e) if e.vencida(ahora, examen.retencion) => (1, e.probabilidad(ahora)),
        None => (2, 0.0),
        Some(e) => (3, e.probabilidad(ahora)),
    };
    let mut orden: Vec<((u8, f64), &PreguntaDeDesarrollo)> = orden.into_iter().map(|q| (clave(q), q)).collect();
    orden.sort_by(|(a, _), (b, _)| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    orden.into_iter().take(TAMANO).map(|(_, q)| de_la_ronda(examen, conceptos, q, &estado, ahora)).collect()
}

fn de_la_ronda(
    examen: &Examen,
    conceptos: &[Concepto],
    q: &PreguntaDeDesarrollo,
    estado: &Estados,
    ahora: i64,
) -> PreguntaDeDesarrolloDeSesion {
    let punto = |id: &String| {
        let c = conceptos.iter().find(|c| c.id == *id);
        Punto {
            concepto: id.clone(),
            idea: c.map_or_else(|| id.clone(), |c| c.idea.clone()),
            pagina: c.map(|c| Pagina::new(&c.fuente)),
        }
    };
    let antes = estado.get(q.id.as_str()).copied();
    let vuelve_tras = |nota| {
        let (resultado, seguro) = como_respuesta(nota);
        Estado::tras(antes, resultado, seguro, ahora).vuelve(ahora, examen.retencion)
    };
    PreguntaDeDesarrolloDeSesion {
        id: q.id.clone(),
        apartado: examen.apartados.get(&q.apartado).unwrap_or(&q.apartado).clone(),
        enunciado: q.enunciado.clone(),
        modelo: q.modelo.clone(),
        puntos: q.puntos.iter().map(punto).collect(),
        probabilidad: en_porcentaje(antes.as_ref(), ahora),
        vuelve: antes.map_or(OTRA_SESION.into(), |e| e.vuelve(ahora, examen.retencion)),
        vuelve_tras_again: vuelve_tras(NotaDeDesarrollo::Again),
        vuelve_tras_hard: vuelve_tras(NotaDeDesarrollo::Hard),
        vuelve_tras_good: vuelve_tras(NotaDeDesarrollo::Good),
    }
}

/// Lo que el estudiante hizo con una pregunta de una sesión de desarrollo: su nota, o «Corregir luego».
#[derive(Deserialize, Serialize, Tsify, Clone, Copy, PartialEq, Debug)]
pub enum Hecha {
    Again,
    Hard,
    Good,
    SinCorregir,
}

/// Las preguntas de desarrollo de una ronda a medias, con su texto en el orden de `ids`. Una lista de `textos` o de
/// `hechas` más corta que `ids` acaba en preguntas sin texto o sin hacer. En una sesión de desarrollo, una pregunta
/// hecha se ve sin cambiarla, y la sesión acaba cuando todas están hechas.
#[derive(Deserialize, Serialize, Tsify, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct RondaDeDesarrollo {
    pub ids: Vec<String>,
    pub textos: Vec<String>,
    #[serde(default)]
    pub hechas: Vec<Option<Hecha>>,
}

/// Una sesión de desarrollo a medias, con una entrada de `hechas` por pregunta.
#[derive(Serialize, Tsify)]
pub struct DesarrolloAMedias {
    pub preguntas: Vec<PreguntaDeDesarrolloDeSesion>,
    pub textos: Vec<String>,
    pub hechas: Vec<Option<Hecha>>,
}

/// Las preguntas y los textos de una ronda guardada, o nada si un id no está en el banco de desarrollo.
pub(crate) fn de_la_ronda_guardada(
    d: &crate::Datos,
    r: &RondaDeDesarrollo,
    ahora: i64,
) -> Option<(Vec<PreguntaDeDesarrolloDeSesion>, Vec<String>)> {
    let estado = estados(&d.desarrollo, &d.historial_de_desarrollo);
    let preguntas: Vec<_> = r
        .ids
        .iter()
        .map(|id| Some(de_la_ronda(&d.examen, &d.conceptos, d.desarrollo.iter().find(|q| q.id == *id)?, &estado, ahora)))
        .collect::<Option<_>>()?;
    if r.textos.len() > preguntas.len() {
        return None;
    }
    let mut textos = r.textos.clone();
    textos.resize(preguntas.len(), String::new());
    Some((preguntas, textos))
}

/// Una ronda sin `desarrollo`, de un simulacro, o con un id que no está en el banco de desarrollo, no se restaura.
pub fn restaurar_desarrollo(d: &crate::Datos, guardado: &crate::Guardado, ahora: i64) -> Option<DesarrolloAMedias> {
    let r = guardado.ronda.as_ref()?;
    if r.simulacro {
        return None;
    }
    let rd = r.desarrollo.as_ref()?;
    let (preguntas, textos) = de_la_ronda_guardada(d, rd, ahora)?;
    if rd.hechas.len() > preguntas.len() {
        return None;
    }
    let mut hechas = rd.hechas.clone();
    hechas.resize(preguntas.len(), None);
    Some(DesarrolloAMedias { hechas, preguntas, textos })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Fuente, Tema};

    fn examen() -> Examen {
        Examen {
            titulo: String::new(),
            preguntas: 20,
            opciones: 4,
            minutos: 25,
            penalizacion: 0.25,
            retencion: 0.9,
            pdfs: [
                ("1.pdf".into(), Tema { nombre: "Uno".into(), apartados: vec!["ssh".into()] }),
                ("2.pdf".into(), Tema { nombre: "Dos".into(), apartados: vec!["tls".into()] }),
            ]
            .into(),
            apartados: [("ssh".into(), "SSH".into()), ("tls".into(), "TLS".into())].into(),
            sin_pregunta: Default::default(),
            congelado: false,
            desarrollo: 2,
            pesos: crate::Pesos { test: 5.0, desarrollo: 5.0 },
        }
    }

    /// `d0`, `d1`… en SSH, con el punto `c0`, `c1`… cada una.
    fn banco_de(n: usize) -> Vec<PreguntaDeDesarrollo> {
        (0..n).map(|i| pregunta(&format!("d{i}"), "ssh", &format!("c{i}"))).collect()
    }

    fn pregunta(id: &str, apartado: &str, punto: &str) -> PreguntaDeDesarrollo {
        PreguntaDeDesarrollo {
            id: id.into(),
            apartado: apartado.into(),
            forma: crate::Forma::Proceso,
            enunciado: format!("Describe {id}."),
            modelo: format!("El modelo de {id}."),
            puntos: vec![punto.into()],
        }
    }

    fn concepto(id: &str, apartado: &str, pdf: &str) -> Concepto {
        Concepto {
            id: id.into(),
            apartado: apartado.into(),
            idea: format!("La idea de {id}."),
            fuente: Fuente { pdf: pdf.into(), pagina: 3 },
        }
    }

    const DIA: i64 = 86_400;

    fn hora(s: i64) -> Timestamp {
        Timestamp::from_second(s).unwrap()
    }

    /// Una respuesta en `ts` con su corrección en `corregida`.
    fn corregida(id: &str, ts: i64, nota: NotaDeDesarrollo, corregida: i64) -> Vec<LineaDeDesarrollo> {
        vec![
            LineaDeDesarrollo::Respuesta(RespuestaDeDesarrollo { id: id.into(), ts: hora(ts), texto: "mi texto".into() }),
            LineaDeDesarrollo::Correccion(CorreccionDeDesarrollo {
                id: id.into(),
                ts: hora(ts),
                nota,
                corregida: hora(corregida),
            }),
        ]
    }

    fn ids(s: &[PreguntaDeDesarrolloDeSesion]) -> Vec<&str> {
        s.iter().map(|q| q.id.as_str()).collect()
    }

    fn de_la_sesion(
        historial: &[LineaDeDesarrollo],
        banco: &[PreguntaDeDesarrollo],
        ahora: i64,
    ) -> Vec<PreguntaDeDesarrolloDeSesion> {
        sesion_de_desarrollo(&examen(), banco, &[], historial, ahora, &mut Azar::new(7))
    }

    fn probabilidad(historial: &[LineaDeDesarrollo], ahora: i64) -> String {
        de_la_sesion(historial, &banco_de(1), ahora).remove(0).probabilidad
    }

    #[test]
    fn cada_nota_dice_cuando_vuelve_la_pregunta_como_despues_de_guardarla() {
        let antes = de_la_sesion(&[], &banco_de(1), DIA).remove(0);
        assert_eq!(antes.vuelve, OTRA_SESION);
        assert_eq!(antes.vuelve_tras_again, OTRA_SESION);
        for (nota, tras) in [(NotaDeDesarrollo::Hard, &antes.vuelve_tras_hard), (NotaDeDesarrollo::Good, &antes.vuelve_tras_good)]
        {
            let despues = de_la_sesion(&corregida("d0", DIA, nota, DIA), &banco_de(1), DIA).remove(0);
            assert_eq!(despues.vuelve, *tras);
        }
        assert_ne!(antes.vuelve_tras_hard, antes.vuelve_tras_good);
    }

    #[test]
    fn una_linea_lee_la_respuesta_y_la_correccion_de_desarrollo_jsonl() {
        let r: LineaDeDesarrollo =
            serde_json::from_str(r#"{"id":"d0","ts":"2026-11-04T12:00:00Z","texto":"SSH cifra"}"#).unwrap();
        assert!(matches!(r, LineaDeDesarrollo::Respuesta(_)));
        let c: LineaDeDesarrollo =
            serde_json::from_str(r#"{"id":"d0","ts":"2026-11-04T12:00:00Z","nota":"Hard","corregida":"2026-11-05T09:00:00Z"}"#)
                .unwrap();
        assert!(matches!(c, LineaDeDesarrollo::Correccion(CorreccionDeDesarrollo { nota: NotaDeDesarrollo::Hard, .. })));
        for mala in [
            r#"{"id":"d0","ts":"2026-11-04T12:00:00Z","nota":"Easy","corregida":"2026-11-05T09:00:00Z"}"#,
            r#"{"id":"d0","ts":"2026-11-04T12:00:00Z"}"#,
            r#"{"id":"d0","ts":"2026-11-04T12:00:00Z","texto":"x","nota":"Good"}"#,
        ] {
            assert!(serde_json::from_str::<LineaDeDesarrollo>(mala).is_err(), "{mala}");
        }
    }

    #[test]
    fn una_respuesta_con_texto_guarda_la_respuesta_y_su_correccion_y_sin_texto_solo_la_correccion() {
        let con = calificar("d0", "SSH cifra", NotaDeDesarrollo::Good, hora(100));
        assert_eq!(
            con,
            corregida("d0", 100, NotaDeDesarrollo::Good, 100)
                .into_iter()
                .map(|l| match l {
                    LineaDeDesarrollo::Respuesta(r) =>
                        LineaDeDesarrollo::Respuesta(RespuestaDeDesarrollo { texto: "SSH cifra".into(), ..r }),
                    c => c,
                })
                .collect::<Vec<_>>()
        );
        let sin = calificar("d0", "  \n", NotaDeDesarrollo::Again, hora(100));
        assert_eq!(sin, corregida("d0", 100, NotaDeDesarrollo::Again, 100)[1..]);
    }

    #[test]
    fn la_union_cuenta_una_vez_cada_linea_con_la_misma_clave() {
        let a = [corregida("d0", 100, NotaDeDesarrollo::Hard, 100), corregida("d1", 50, NotaDeDesarrollo::Good, 50)].concat();
        // La misma respuesta de d0, con su corrección vieja y una nueva.
        let b =
            [corregida("d0", 100, NotaDeDesarrollo::Hard, 100), corregida("d0", 100, NotaDeDesarrollo::Good, 200)[1..].to_vec()]
                .concat();
        let u = unir_desarrollo(&a, &b);
        assert_eq!(u.len(), 5);
        assert_eq!(u.iter().map(|l| l.ts().as_second()).collect::<Vec<_>>(), [50, 50, 100, 100, 100]);
    }

    #[test]
    fn una_respuesta_sin_corregir_no_mueve_el_estado() {
        let sin_corregir =
            [LineaDeDesarrollo::Respuesta(RespuestaDeDesarrollo { id: "d0".into(), ts: hora(0), texto: "x".into() })];
        assert_eq!(probabilidad(&sin_corregir, DIA), "sin ver");
        assert_ne!(probabilidad(&corregida("d0", 0, NotaDeDesarrollo::Good, 0), DIA), "sin ver");
    }

    #[test]
    fn la_correccion_mas_nueva_gana_y_fsrs_la_cuenta_a_la_hora_de_la_respuesta() {
        let tarde = corregida("d0", 0, NotaDeDesarrollo::Good, 10 * DIA);
        assert_eq!(probabilidad(&tarde, DIA), probabilidad(&corregida("d0", 0, NotaDeDesarrollo::Good, 0), DIA));
        let otra = [corregida("d0", 0, NotaDeDesarrollo::Good, 20 * DIA), corregida("d0", 0, NotaDeDesarrollo::Again, 10 * DIA)]
            .concat();
        assert_eq!(probabilidad(&otra, DIA), probabilidad(&tarde, DIA));
    }

    #[test]
    fn la_sesion_lleva_4_las_falladas_primero_despues_las_vencidas_las_sin_ver_y_las_que_sabes() {
        // d0 y d1 sabidas hoy, d2 vencida hace mucho, d3 y d4 falladas (d4 primero), d5 sin ver.
        let historial = [
            corregida("d0", 100 * DIA, NotaDeDesarrollo::Good, 100 * DIA),
            corregida("d1", 100 * DIA, NotaDeDesarrollo::Good, 100 * DIA),
            corregida("d2", 0, NotaDeDesarrollo::Good, 0),
            corregida("d3", 100 * DIA, NotaDeDesarrollo::Again, 100 * DIA),
            corregida("d4", 99 * DIA, NotaDeDesarrollo::Again, 99 * DIA),
        ]
        .concat();
        let s = de_la_sesion(&historial, &banco_de(6), 100 * DIA);
        assert_eq!(ids(&s), ["d4", "d3", "d2", "d5"]);
        let s = de_la_sesion(&historial, &banco_de(2), 100 * DIA);
        assert_eq!(ids(&s).len(), 2);
    }

    #[test]
    fn las_vencidas_van_de_la_menor_probabilidad_a_la_mayor() {
        let historial =
            [corregida("d0", 50 * DIA, NotaDeDesarrollo::Good, 50 * DIA), corregida("d1", 0, NotaDeDesarrollo::Good, 0)].concat();
        assert_eq!(ids(&de_la_sesion(&historial, &banco_de(2), 400 * DIA)), ["d1", "d0"]);
    }

    #[test]
    fn la_retencion_del_examen_decide_que_pregunta_esta_vencida() {
        let historial = corregida("d0", 0, NotaDeDesarrollo::Good, 0);
        let primera = |retencion| {
            let examen = Examen { retencion, ..examen() };
            sesion_de_desarrollo(&examen, &banco_de(3), &[], &historial, DIA, &mut Azar::new(7)).remove(0).id
        };
        assert_ne!(primera(0.8), "d0");
        assert_eq!(primera(0.99), "d0");
    }

    #[test]
    fn la_semilla_rompe_los_empates() {
        let banco = banco_de(8);
        let orden = |semilla| ids(&sesion_de_desarrollo(&examen(), &banco, &[], &[], 0, &mut Azar::new(semilla))).join(" ");
        assert_eq!(orden(1), orden(1));
        assert!((2..20).any(|s| orden(s) != orden(1)));
    }

    #[test]
    fn sin_banco_de_desarrollo_no_hay_cifra() {
        assert_eq!(cifra_de_desarrollo(&examen(), &[], &[], &[], None, 0), None);
    }

    #[test]
    fn la_seleccion_toma_las_preguntas_por_el_tema_de_sus_puntos() {
        let banco = [pregunta("ssh-d1", "ssh", "a"), pregunta("tls-d1", "tls", "b")];
        let conceptos = [concepto("a", "ssh", "1.pdf"), concepto("b", "tls", "2.pdf")];
        let solo_tls = [Par { pdf: "2.pdf".into(), apartado: "tls".into() }];
        let elegidas = desarrollo_de_la_seleccion(&examen(), &banco, &conceptos, Some(&solo_tls));
        assert_eq!(elegidas, [banco[1].clone()]);
        let cifra = cifra_de_desarrollo(&examen(), &banco, &conceptos, &[], Some(&solo_tls), 0).unwrap();
        assert_eq!((cifra.sesion, cifra.simulacro, cifra.sin_corregir, cifra.hay), (1, 1, 0, 1));
        assert_eq!(desarrollo_de_la_seleccion(&examen(), &banco, &conceptos, None).len(), 2);
    }

    #[test]
    fn la_cifra_cuenta_lo_que_sabes_en_la_seleccion_y_en_cada_par() {
        let banco = [pregunta("ssh-d1", "ssh", "a"), pregunta("ssh-d2", "ssh", "a"), pregunta("tls-d1", "tls", "b")];
        let conceptos = [concepto("a", "ssh", "1.pdf"), concepto("b", "tls", "2.pdf")];
        let historial = calificar("ssh-d1", "x", NotaDeDesarrollo::Good, hora(0));
        let cuenta = |c: &CifraDeDesarrollo| c.pares.iter().map(|p| (p.pdf.clone(), p.sabes, p.hay)).collect::<Vec<_>>();
        let hoy = cifra_de_desarrollo(&examen(), &banco, &conceptos, &historial, None, 0).unwrap();
        assert_eq!((hoy.sabes, hoy.hay), (1, 3));
        assert_eq!(cuenta(&hoy), [("1.pdf".into(), 1, 2), ("2.pdf".into(), 0, 1)]);
        let solo_tls = [Par { pdf: "2.pdf".into(), apartado: "tls".into() }];
        let tls = cifra_de_desarrollo(&examen(), &banco, &conceptos, &historial, Some(&solo_tls), 0).unwrap();
        assert_eq!((tls.sabes, tls.hay), (0, 1));
        assert_eq!(cuenta(&tls), cuenta(&hoy));
        let vencida = cifra_de_desarrollo(&examen(), &banco, &conceptos, &historial, None, 400 * DIA).unwrap();
        assert_eq!(vencida.sabes, 0);
    }

    #[test]
    fn la_pregunta_lleva_el_nombre_del_apartado_y_cada_punto_su_idea_y_su_pagina() {
        let banco = [pregunta("ssh-d1", "ssh", "a")];
        let q = sesion_de_desarrollo(&examen(), &banco, &[concepto("a", "ssh", "1.pdf")], &[], 0, &mut Azar::new(1)).remove(0);
        assert_eq!(
            (q.apartado.as_str(), q.enunciado.as_str(), q.modelo.as_str()),
            ("SSH", "Describe ssh-d1.", "El modelo de ssh-d1.")
        );
        assert_eq!(q.puntos[0].idea, "La idea de a.");
        assert_eq!(q.puntos[0].pagina.as_ref().unwrap().imagen, "paginas/1-3.webp");
    }

    fn datos(desarrollo: Vec<PreguntaDeDesarrollo>, historial_de_desarrollo: Vec<LineaDeDesarrollo>) -> crate::Datos {
        crate::Datos {
            examen: examen(),
            banco: vec![],
            conceptos: vec![],
            desarrollo,
            historial: vec![],
            historial_de_desarrollo,
        }
    }

    fn guardado(simulacro: bool, desarrollo: Option<RondaDeDesarrollo>) -> crate::Guardado {
        crate::Guardado {
            guardada: hora(0),
            ronda: Some(crate::RondaGuardada {
                ids: vec![],
                resultados: vec![],
                respuestas: vec![],
                simulacro,
                limite: None,
                desarrollo,
            }),
            seleccion: None,
        }
    }

    fn ronda(ids: &[&str], textos: &[&str], hechas: Vec<Option<Hecha>>) -> RondaDeDesarrollo {
        RondaDeDesarrollo {
            ids: ids.iter().map(|&i| i.into()).collect(),
            textos: textos.iter().map(|&t| t.into()).collect(),
            hechas,
        }
    }

    #[test]
    fn la_ronda_guardada_restaura_el_texto_de_cada_pregunta_y_las_hechas() {
        let g = guardado(false, Some(ronda(&["d1", "d0"], &["SSH cifra"], vec![Some(Hecha::Hard)])));
        let json = serde_json::to_string(&g).unwrap();
        assert_eq!(serde_json::from_str::<crate::Guardado>(&json).unwrap(), g);
        let r = restaurar_desarrollo(&datos(banco_de(2), vec![]), &g, 0).unwrap();
        assert_eq!(ids(&r.preguntas), ["d1", "d0"]);
        assert_eq!((r.textos, r.hechas), (vec!["SSH cifra".to_string(), String::new()], vec![Some(Hecha::Hard), None]));
        let de_mas = guardado(false, Some(ronda(&["d0"], &[], vec![None, None])));
        assert!(restaurar_desarrollo(&datos(banco_de(2), vec![]), &de_mas, 0).is_none());
    }

    #[test]
    fn una_ronda_de_test_de_simulacro_o_con_una_pregunta_que_no_esta_no_se_restaura_como_desarrollo() {
        let d = datos(banco_de(2), vec![]);
        assert!(restaurar_desarrollo(&d, &guardado(false, None), 0).is_none());
        assert!(restaurar_desarrollo(&d, &guardado(true, Some(ronda(&["d0"], &[], vec![]))), 0).is_none());
        assert!(restaurar_desarrollo(&d, &guardado(false, Some(ronda(&["d9"], &[], vec![]))), 0).is_none());
    }

    #[test]
    fn una_respuesta_con_texto_queda_sin_corregir_y_una_vacia_lleva_again() {
        let con_texto = responder("d0", "SSH cifra", hora(5));
        assert_eq!(
            con_texto,
            [LineaDeDesarrollo::Respuesta(RespuestaDeDesarrollo { id: "d0".into(), ts: hora(5), texto: "SSH cifra".into() })]
        );
        let vacia = responder("d1", "  ", hora(5));
        assert_eq!(vacia, [corregir_respuesta("d1", hora(5), NotaDeDesarrollo::Again, hora(5))]);
        let historial = [con_texto.clone(), vacia].concat();
        assert_eq!(sin_corregir(&historial).len(), 1);
        let corregido = [historial, vec![corregir_respuesta("d0", hora(5), NotaDeDesarrollo::Good, hora(9))]].concat();
        assert!(sin_corregir(&corregido).is_empty());
    }

    #[test]
    fn una_respuesta_sin_corregir_no_mueve_el_estado_y_su_correccion_cuenta_a_la_hora_de_la_respuesta() {
        let sin = responder("d0", "SSH cifra", hora(0));
        assert!(estados(&banco_de(1), &sin).is_empty());
        let con = [sin, vec![corregir_respuesta("d0", hora(0), NotaDeDesarrollo::Good, hora(40 * DIA))]].concat();
        assert_eq!(estados(&banco_de(1), &con)["d0"].ultima, 0);
        let cifra = cifra_de_desarrollo(&examen(), &banco_de(1), &[], &responder("d0", "x", hora(0)), None, 0).unwrap();
        assert_eq!(cifra.sin_corregir, 1);
    }

    #[test]
    fn el_simulacro_mixto_saca_las_preguntas_de_desarrollo_del_examen_al_azar() {
        let d = datos(banco_de(8), vec![]);
        let s = crate::simulacro(&d, 0, &mut Azar::new(1));
        assert_eq!(s.desarrollo.len(), 2);
        assert!((2..20).any(|k| ids(&crate::simulacro(&d, 0, &mut Azar::new(k)).desarrollo) != ids(&s.desarrollo)));
        assert_eq!(crate::simulacro(&datos(banco_de(1), vec![]), 0, &mut Azar::new(1)).desarrollo.len(), 1);
    }

    #[test]
    fn el_simulacro_mixto_restaurado_lleva_sus_preguntas_de_desarrollo_con_su_texto() {
        let g = guardado(true, Some(ronda(&["d1"], &["SSH cifra"], vec![])));
        let r = crate::restaurar(&datos(banco_de(2), vec![]), &g, 0, &mut Azar::new(1)).unwrap();
        assert_eq!((ids(&r.desarrollo), r.textos), (vec!["d1"], vec!["SSH cifra".to_string()]));
        assert!(
            crate::restaurar(
                &datos(banco_de(2), vec![]),
                &guardado(false, Some(ronda(&["d1"], &[], vec![]))),
                0,
                &mut Azar::new(1)
            )
            .is_none()
        );
    }
}
