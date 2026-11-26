//! La lógica de la app. `nucleo` no lee archivos ni la red.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use tsify::Tsify;

pub mod wasm;

/// Las cifras de un examen, de `examen.json`.
#[derive(Deserialize, Serialize, Tsify)]
#[serde(deny_unknown_fields)]
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
    /// Los temas, por el nombre de su PDF de `pdfs/`.
    pub pdfs: std::collections::BTreeMap<String, Tema>,
    /// El nombre de cada apartado, por su clave. Un apartado tiene un solo nombre, aunque salga en 2 temas.
    pub apartados: std::collections::BTreeMap<String, String>,
    /// Para cada PDF, las páginas sin pregunta y la razón de cada una. `banco comprobar` las lee. La página es texto, como cada clave de un objeto de JSON: el WASM no convierte `"1"` en un número.
    #[serde(default)]
    pub sin_pregunta: std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>>,
    /// El banco es anterior a la regla de 3 preguntas por concepto, y no se reescribe: `banco comprobar` da esa cuenta como aviso, y no como error.
    #[serde(default)]
    pub congelado: bool,
}

/// Un tema: un PDF del temario, con su nombre y las claves de sus apartados.
#[derive(Deserialize, Serialize, Tsify, Clone)]
#[serde(deny_unknown_fields)]
pub struct Tema {
    pub nombre: String,
    pub apartados: Vec<String>,
}

/// Un examen de la lista de la raíz, que abre `/<carpeta>/`.
#[derive(Serialize, Tsify)]
pub struct ExamenDeLaLista {
    pub carpeta: String,
    pub titulo: String,
}

/// Una pregunta del banco, de `preguntas.json`.
#[derive(Deserialize, Serialize, Tsify, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Pregunta {
    pub id: String,
    pub concepto: String,
    /// La clave del apartado, uno de los apartados del tema de `fuente.pdf`.
    pub apartado: String,
    pub fuente: Fuente,
    pub pregunta: String,
    pub correcta: String,
    pub distractores: Vec<String>,
    pub explicacion: String,
}

/// La página del PDF de `pdfs/` que respalda una pregunta.
#[derive(Deserialize, Serialize, Tsify, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Fuente {
    pub pdf: String,
    pub pagina: u32,
}

/// Una línea de `respuestas.jsonl`. `ts` es la hora en ISO 8601 con `Z`, como `2026-11-04T12:41:22Z`.
#[derive(Deserialize, Serialize, Tsify, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Respuesta {
    pub id: String,
    #[tsify(type = "string")]
    pub ts: Timestamp,
    pub resultado: Resultado,
    pub seguro: bool,
}

/// La nota que el usuario escribe en una pregunta. `notas.json` guarda un objeto de `id` a `texto`.
#[derive(Deserialize, Serialize, Tsify)]
#[serde(deny_unknown_fields)]
pub struct Nota {
    pub id: String,
    pub texto: String,
}

/// Lo que el servidor da de un examen: sus cifras, su banco y su historial.
#[derive(Deserialize, Serialize, Tsify)]
pub struct Datos {
    pub examen: Examen,
    pub banco: Vec<Pregunta>,
    pub historial: Vec<Respuesta>,
}

/// El estado de una pregunta: sin ver, o su paso, el número de aciertos con "seguro" seguidos.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Estado {
    SinVer,
    Paso(u32),
}

impl Estado {
    /// Una pregunta está en "sabes" desde el paso 1.
    pub fn sabes(self) -> bool {
        matches!(self, Estado::Paso(n) if n > 0)
    }

    /// Un acierto con "seguro" sube el paso, aunque la pregunta no haya vencido. Otra respuesta lo pone a 0.
    /// Una pregunta sin ver acertada con "seguro" salta al paso 3, que vence después de la semana del examen.
    pub fn tras(self, acierto: bool, seguro: bool) -> Estado {
        match (self, sube(acierto, seguro)) {
            (Estado::SinVer, true) => Estado::Paso(3),
            (Estado::Paso(n), true) => Estado::Paso(n.saturating_add(1)),
            (_, false) => Estado::Paso(0),
        }
    }

    /// El hueco en segundos: 1 día con el paso 1, y después ×2,5 por paso.
    /// Con el paso a 0, el hueco es 0.
    pub fn hueco(self) -> i64 {
        match self {
            Estado::SinVer | Estado::Paso(0) => 0,
            // Un `f64` demasiado grande pasa a `i64::MAX`.
            Estado::Paso(n) => (86_400.0 * 2.5f64.powf(n as f64 - 1.0)) as i64,
        }
    }
}

impl std::fmt::Display for Estado {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Estado::SinVer => write!(f, "sin ver"),
            Estado::Paso(n) => write!(f, "paso {n}"),
        }
    }
}

/// Solo un acierto con "seguro" sube el paso de una pregunta.
fn sube(acierto: bool, seguro: bool) -> bool {
    acierto && seguro
}

/// La respuesta a una pregunta de la ronda, con el paso de la pregunta tras ella y su aviso.
#[derive(Serialize, Tsify, PartialEq, Debug)]
pub struct Correccion {
    pub respuesta: Respuesta,
    pub paso: String,
    pub aviso: Option<String>,
}

/// Corrige una pregunta de la ronda. Sin `opcion`, la pregunta queda en blanco, sin "seguro".
pub fn corregir(q: &PreguntaDeSesion, opcion: Option<u32>, seguro: bool, ahora: Timestamp) -> Correccion {
    let resultado = match opcion {
        None => Resultado::Blanco,
        Some(o) if o == q.correcta => Resultado::Acierto,
        Some(_) => Resultado::Fallo,
    };
    let seguro = seguro && opcion.is_some();
    let aviso = match (resultado, seguro) {
        (Resultado::Fallo, true) => Some("Fallo con «seguro»: una idea falsa. Repásala."),
        (Resultado::Acierto, false) => Some("Acierto con «dudo»: cuenta como un fallo."),
        _ => None,
    };
    Correccion {
        paso: if sube(resultado == Resultado::Acierto, seguro) { &q.paso_tras_seguro } else { &q.paso_tras_otra }.clone(),
        aviso: aviso.map(Into::into),
        respuesta: Respuesta { id: q.id.clone(), ts: ahora, resultado, seguro },
    }
}

/// Los segundos que quedan de un simulacro que termina en `limite`. Las horas van en milisegundos.
pub fn restante(limite: f64, ahora: f64) -> u32 {
    ((limite - ahora) / 1000.0).round().max(0.0) as u32
}

/// El paso de cada pregunta vista del banco, por su `id`, y la hora en que vence.
fn estados<'a>(banco: &'a [Pregunta], historial: &[Respuesta]) -> std::collections::HashMap<&'a str, (Estado, i64)> {
    let ids: std::collections::HashSet<&str> = banco.iter().map(|q| q.id.as_str()).collect();
    let mut ordenado: Vec<(i64, &Respuesta)> = historial.iter().map(|r| (r.ts.as_second(), r)).collect();
    ordenado.sort_by_key(|(t, _)| *t);
    let mut estado = std::collections::HashMap::new();
    for (t, r) in ordenado {
        let Some(&id) = ids.get(r.id.as_str()) else {
            continue;
        };
        let antes = estado.get(id).map_or(Estado::SinVer, |e: &(Estado, i64)| e.0);
        let despues = antes.tras(r.resultado == Resultado::Acierto, r.seguro);
        estado.insert(id, (despues, t.saturating_add(despues.hueco())));
    }
    estado
}

/// Lo que muestra la pantalla del inicio.
#[derive(Serialize, Tsify, PartialEq, Debug)]
#[serde(tag = "estado")]
pub enum Inicio {
    BancoVacio {
        titulo: String,
    },
    /// `sesion` y `simulacro` son el número de preguntas de cada ronda con la selección, y la tarjeta usa solo la selección.
    /// `parte` dice "con 3 de 5 temas" cuando falta algún par.
    Listo {
        titulo: String,
        sesion: u32,
        simulacro: u32,
        tarjeta: Tarjeta,
        temas: Vec<TemaDelInicio>,
        parte: Option<String>,
    },
}

/// Un par de la selección: un apartado dentro de un tema, por el PDF del tema.
#[derive(Deserialize, Serialize, Tsify, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Par {
    pub pdf: String,
    pub apartado: String,
}

/// Un tema en el inicio, con su número de preguntas, las que están en "sabes", y sus apartados.
#[derive(Serialize, Tsify, PartialEq, Debug)]
pub struct TemaDelInicio {
    pub pdf: String,
    pub nombre: String,
    pub preguntas: u32,
    pub sabes: u32,
    pub apartados: Vec<ApartadoDelInicio>,
}

#[derive(Serialize, Tsify, PartialEq, Debug)]
pub struct ApartadoDelInicio {
    pub apartado: String,
    pub nombre: String,
    pub preguntas: u32,
    pub sabes: u32,
    pub marcado: bool,
}

/// Las preguntas de los pares marcados. Sin selección, todo el banco.
pub fn de_la_seleccion(banco: &[Pregunta], seleccion: Option<&[Par]>) -> Vec<Pregunta> {
    banco.iter().filter(|q| marcado(seleccion, &q.fuente.pdf, &q.apartado)).cloned().collect()
}

fn marcado(seleccion: Option<&[Par]>, pdf: &str, apartado: &str) -> bool {
    seleccion.is_none_or(|s| s.iter().any(|p| p.pdf == pdf && p.apartado == apartado))
}

fn conceptos(banco: &[Pregunta]) -> u32 {
    banco.iter().map(|q| q.concepto.as_str()).collect::<std::collections::HashSet<_>>().len() as u32
}

/// La pantalla de la ronda. Del inicio sale una sesión, cada pregunta pasa de sin contestar a contestada,
/// y la última lleva al final. El botón ✕ y el final vuelven al inicio.
#[derive(Serialize, Tsify)]
#[serde(tag = "estado")]
pub enum Ronda {
    Inicio,
    Pregunta { contestada: bool },
    Final,
}

/// El inicio, con la tarjeta de la nota para sacar más de `objetivo` sobre 10.
/// Sin selección, todo está marcado. Un par de la selección que no existe no cuenta.
pub fn inicio(examen: &Examen, banco: &[Pregunta], historial: &[Respuesta], seleccion: Option<&[Par]>, objetivo: f64) -> Inicio {
    let titulo = examen.titulo.clone();
    if banco.is_empty() {
        return Inicio::BancoVacio { titulo };
    }
    let estado = estados(banco, historial);
    let sabe = |q: &&Pregunta| estado.get(q.id.as_str()).is_some_and(|e| e.0.sabes());
    let temas: Vec<TemaDelInicio> = examen
        .pdfs
        .iter()
        .map(|(pdf, tema)| TemaDelInicio {
            pdf: pdf.clone(),
            nombre: tema.nombre.clone(),
            preguntas: banco.iter().filter(|q| q.fuente.pdf == *pdf).count() as u32,
            sabes: banco.iter().filter(|q| q.fuente.pdf == *pdf).filter(sabe).count() as u32,
            apartados: tema
                .apartados
                .iter()
                .map(|a| ApartadoDelInicio {
                    apartado: a.clone(),
                    nombre: examen.apartados.get(a).unwrap_or(a).clone(),
                    preguntas: banco.iter().filter(|q| q.fuente.pdf == *pdf && q.apartado == *a).count() as u32,
                    sabes: banco.iter().filter(|q| q.fuente.pdf == *pdf && q.apartado == *a).filter(sabe).count() as u32,
                    marcado: marcado(seleccion, pdf, a),
                })
                .collect(),
        })
        .collect();
    let con = temas.iter().filter(|t| t.apartados.iter().any(|a| a.marcado)).count();
    let parte = temas.iter().any(|t| t.apartados.iter().any(|a| !a.marcado)).then(|| format!("con {con} de {} temas", temas.len()));
    let banco = de_la_seleccion(banco, seleccion);
    let tarjeta = tarjeta(examen, &banco, historial, objetivo);
    let sesion = conceptos(&banco).min(TAMANO_SESION as u32);
    let simulacro = examen.preguntas.min(banco.len() as u32);
    Inicio::Listo { titulo, sesion, simulacro, tarjeta, temas, parte }
}

const TAMANO_SESION: usize = 12;

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
#[derive(Serialize, Deserialize, Tsify, Clone)]
pub struct PreguntaDeSesion {
    pub id: String,
    /// El nombre del apartado.
    pub apartado: String,
    pub concepto: String,
    pub pregunta: String,
    pub opciones: Vec<String>,
    /// El índice de la opción correcta en `opciones`.
    pub correcta: u32,
    pub explicacion: String,
    /// El paso de la pregunta antes de contestar.
    pub paso: String,
    /// El paso tras un acierto con "seguro".
    pub paso_tras_seguro: String,
    /// El paso tras un fallo, un acierto con "dudo" o una pregunta en blanco.
    pub paso_tras_otra: String,
    /// El PDF y la página, como `2A _Teoría.pdf · pág. 7`.
    pub fuente: String,
    pub pagina: u32,
    /// La imagen de la página en `paginas/`, relativa a la página del examen.
    pub imagen: String,
    /// El PDF abierto en la página.
    pub pdf: String,
    /// La pregunta y sus opciones con su letra, sin la solución, para preguntar a otra herramienta.
    pub copiar: String,
}

/// La sesión de 12, con una pregunta por concepto como mucho. Recorre las preguntas en este orden, y toma la primera de cada concepto:
/// las falladas, las vencidas con paso 1 o más, las hermanas sin ver de un concepto visto, las de los conceptos sin ver,
/// y el relleno con las que vencen antes. Cada grupo va del que vence antes al que vence después, y el azar rompe los empates.
/// `ahora` va en segundos desde 1970.
pub fn sesion(examen: &Examen, banco: &[Pregunta], historial: &[Respuesta], ahora: i64, azar: &mut Azar) -> Vec<PreguntaDeSesion> {
    let estado = estados(banco, historial);
    let vistos: std::collections::HashSet<&str> =
        banco.iter().filter(|q| estado.contains_key(q.id.as_str())).map(|q| q.concepto.as_str()).collect();
    let mut orden: Vec<&Pregunta> = banco.iter().collect();
    azar.barajar(&mut orden);
    orden.sort_by_key(|q| match estado.get(q.id.as_str()) {
        Some(&(Estado::Paso(0), vence)) => (0, vence),
        Some(&(_, vence)) if vence <= ahora => (1, vence),
        None if vistos.contains(q.concepto.as_str()) => (2, 0),
        None => (3, 0),
        Some(&(_, vence)) => (4, vence),
    });
    let mut usados = std::collections::HashSet::new();
    let mut elegidas: Vec<&Pregunta> = orden.into_iter().filter(|q| usados.insert(q.concepto.as_str())).take(TAMANO_SESION).collect();
    azar.barajar(&mut elegidas);
    elegidas.into_iter().map(|q| de_la_ronda(examen, q, &estado, azar)).collect()
}

/// La pregunta con sus opciones al azar y su paso.
fn de_la_ronda(examen: &Examen, q: &Pregunta, estado: &std::collections::HashMap<&str, (Estado, i64)>, azar: &mut Azar) -> PreguntaDeSesion {
    let paso = estado.get(q.id.as_str()).map_or(Estado::SinVer, |e| e.0);
    let mut opciones: Vec<String> = std::iter::once(&q.correcta).chain(&q.distractores).cloned().collect();
    azar.barajar(&mut opciones);
    PreguntaDeSesion {
        id: q.id.clone(),
        apartado: examen.apartados.get(&q.apartado).unwrap_or(&q.apartado).clone(),
        concepto: q.concepto.clone(),
        pregunta: q.pregunta.clone(),
        correcta: opciones.iter().position(|o| *o == q.correcta).unwrap() as u32,
        copiar: para_copiar(&q.pregunta, &opciones),
        opciones,
        explicacion: q.explicacion.clone(),
        paso: paso.to_string(),
        paso_tras_seguro: paso.tras(true, true).to_string(),
        paso_tras_otra: paso.tras(false, false).to_string(),
        fuente: format!("{} · pág. {}", q.fuente.pdf, q.fuente.pagina),
        pagina: q.fuente.pagina,
        imagen: format!("paginas/{}-{}.webp", q.fuente.pdf.strip_suffix(".pdf").unwrap_or(&q.fuente.pdf), q.fuente.pagina),
        pdf: format!("pdfs/{}#page={}", q.fuente.pdf, q.fuente.pagina),
    }
}

/// La pregunta y sus opciones con su letra.
fn para_copiar(pregunta: &str, opciones: &[String]) -> String {
    std::iter::once(format!("{pregunta}\n"))
        .chain(opciones.iter().zip('A'..).map(|(o, letra)| format!("{letra}) {o}")))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Los textos de una pregunta editada durante el test, con las opciones en el orden de la pantalla.
#[derive(Deserialize, Tsify)]
#[serde(deny_unknown_fields)]
pub struct Edicion {
    pub pregunta: String,
    pub opciones: Vec<String>,
    pub explicacion: String,
}

/// La pregunta editada: la del banco, para `preguntas.json`, y la de la ronda, con sus opciones en el mismo sitio.
#[derive(Serialize, Tsify)]
pub struct Editada {
    pub banco: Pregunta,
    pub ronda: PreguntaDeSesion,
}

/// Cada opción vuelve a su sitio del banco por su texto antes de la edición, así `distractores` conserva su orden.
/// Una pregunta que no está en el banco, o con otras opciones, no se edita.
pub fn editar(banco: &[Pregunta], q: &PreguntaDeSesion, e: &Edicion) -> Option<Editada> {
    let original = banco.iter().find(|b| b.id == q.id)?;
    if e.opciones.len() != q.opciones.len() {
        return None;
    }
    let nueva = |viejo: &String| Some(e.opciones[q.opciones.iter().position(|o| o == viejo)?].clone());
    let editada = Pregunta {
        pregunta: e.pregunta.clone(),
        correcta: nueva(&original.correcta)?,
        distractores: original.distractores.iter().map(nueva).collect::<Option<_>>()?,
        explicacion: e.explicacion.clone(),
        ..original.clone()
    };
    let ronda = PreguntaDeSesion {
        pregunta: e.pregunta.clone(),
        opciones: e.opciones.clone(),
        explicacion: e.explicacion.clone(),
        copiar: para_copiar(&e.pregunta, &e.opciones),
        ..q.clone()
    };
    Some(Editada { banco: editada, ronda })
}

/// El simulacro: las preguntas del examen, al azar del banco que recibe, y su duración. Un banco más pequeño da un simulacro más corto.
/// Lleva una pregunta por concepto, y rellena con hermanas solo cuando faltan conceptos.
#[derive(Serialize, Tsify)]
pub struct Simulacro {
    pub segundos: u32,
    pub preguntas: Vec<PreguntaDeSesion>,
}

pub fn simulacro(examen: &Examen, banco: &[Pregunta], historial: &[Respuesta], azar: &mut Azar) -> Simulacro {
    let estado = estados(banco, historial);
    let mut barajado: Vec<&Pregunta> = banco.iter().collect();
    azar.barajar(&mut barajado);
    let mut usados = std::collections::HashSet::new();
    let (mut elegidas, hermanas): (Vec<&Pregunta>, Vec<&Pregunta>) = barajado.into_iter().partition(|q| usados.insert(q.concepto.as_str()));
    elegidas.extend(hermanas);
    elegidas.truncate(examen.preguntas as usize);
    azar.barajar(&mut elegidas);
    Simulacro {
        segundos: examen.minutos.saturating_mul(60),
        preguntas: elegidas.into_iter().map(|q| de_la_ronda(examen, q, &estado, azar)).collect(),
    }
}

/// `ronda.json`, y la ronda de `localStorage`: la ronda a medias, la selección y la hora del guardado, en ISO 8601 con `Z`.
/// Sin ronda a medias, `ronda` es `None`.
#[derive(Deserialize, Serialize, Tsify, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Guardado {
    #[tsify(type = "string")]
    pub guardada: Timestamp,
    pub ronda: Option<RondaGuardada>,
    /// Los pares marcados. Sin selección, todo está marcado.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seleccion: Option<Vec<Par>>,
}

/// Una ronda a medias: sus preguntas, el resultado de cada una y el límite del simulacro, en milisegundos.
/// `resultados` va en el orden de `ids`, con `null` en una pregunta sin contestar.
/// `respuestas` lleva la última respuesta de cada pregunta contestada del simulacro, que entra en el historial al terminar.
/// Una sesión manda cada respuesta al historial al contestar, y su lista va vacía.
#[derive(Deserialize, Serialize, Tsify, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct RondaGuardada {
    pub ids: Vec<String>,
    pub resultados: Vec<Option<Resultado>>,
    pub respuestas: Vec<Respuesta>,
    pub simulacro: bool,
    pub limite: Option<u64>,
}

/// El guardado más nuevo. Con la misma hora gana `a`.
pub fn gana(a: Guardado, b: Guardado) -> Guardado {
    if b.guardada > a.guardada { b } else { a }
}

/// Las respuestas de 2 historiales en orden de hora. Una respuesta con el mismo `id` y `ts` entra una vez.
pub fn unir(a: &[Respuesta], b: &[Respuesta]) -> Vec<Respuesta> {
    let mut vistas = std::collections::HashSet::new();
    let mut u: Vec<Respuesta> = a.iter().chain(b).filter(|r| vistas.insert((&r.id, &r.ts))).cloned().collect();
    u.sort_by_key(|r| r.ts);
    u
}

/// La ronda a medias de un guardado, con las preguntas del banco y un resultado por pregunta.
/// La pregunta actual es la primera sin resultado.
#[derive(Serialize, Tsify)]
pub struct RondaAMedias {
    pub preguntas: Vec<PreguntaDeSesion>,
    pub resultados: Vec<Option<Resultado>>,
    pub respuestas: Vec<Respuesta>,
    pub simulacro: bool,
    pub limite: Option<u64>,
}

/// Una ronda con un id que no está en el banco, o con más resultados que preguntas, no se restaura.
/// Una lista de resultados más corta que `ids` acaba en preguntas sin contestar.
pub fn restaurar(examen: &Examen, banco: &[Pregunta], historial: &[Respuesta], guardado: &Guardado, azar: &mut Azar) -> Option<RondaAMedias> {
    let r = guardado.ronda.as_ref()?;
    let estado = estados(banco, historial);
    let preguntas: Vec<_> =
        r.ids.iter().map(|id| Some(de_la_ronda(examen, banco.iter().find(|q| q.id == *id)?, &estado, azar))).collect::<Option<_>>()?;
    if r.resultados.len() > preguntas.len() {
        return None;
    }
    let mut resultados = r.resultados.clone();
    resultados.resize(preguntas.len(), None);
    Some(RondaAMedias { preguntas, resultados, respuestas: r.respuestas.clone(), simulacro: r.simulacro, limite: r.limite })
}

/// Lo que pasó en una pregunta: un acierto, un fallo o una pregunta en blanco.
#[derive(Deserialize, Serialize, Tsify, Clone, Copy, PartialEq, Debug)]
pub enum Resultado {
    Acierto,
    Fallo,
    Blanco,
}

/// El cierre de un simulacro: sus preguntas, y el resultado de las que tuvieron respuesta antes del límite.
#[derive(Deserialize, Serialize, Tsify)]
pub struct Cierre {
    pub preguntas: u32,
    pub resultados: Vec<Resultado>,
}

/// El final del simulacro. `nota` va sobre 10, con 2 decimales como mucho.
#[derive(Serialize, Tsify, PartialEq, Debug)]
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
#[derive(Serialize, Tsify, PartialEq, Debug)]
pub struct Tarjeta {
    pub sabes: u32,
    pub no_sabes: u32,
    pub sin_ver: u32,
    /// La probabilidad de sacar el objetivo hoy, con la forma de las dudas que da más.
    pub hoy: u32,
    /// Las preguntas en "sabes" que dan un 95 %.
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

#[derive(Serialize, Tsify, PartialEq, Debug)]
pub struct Dudas {
    pub texto: String,
    pub preguntas: u32,
    pub porcentaje: u32,
    pub detalle: Option<String>,
}

/// La tarjeta de la nota para sacar más de `objetivo` sobre 10.
pub fn tarjeta(examen: &Examen, banco: &[Pregunta], historial: &[Respuesta], objetivo: f64) -> Tarjeta {
    let a_ciegas = 1.0 / examen.opciones as f64;
    let (sabes, no_sabes, sin_ver) = cuenta(banco, historial);
    let n = sabes + no_sabes + sin_ver;
    let total = n.max(1) as f64;
    // Una pregunta en blanco no es una duda contestada.
    let dudas = historial.iter().filter(|r| !r.seguro && r.resultado != Resultado::Blanco).count();
    let aciertos_dudo = historial.iter().filter(|r| !r.seguro && r.resultado == Resultado::Acierto).count();
    let tasa_dudo = if dudas == 0 { a_ciegas } else { aciertos_dudo as f64 / dudas as f64 };
    let tasa = (no_sabes as f64 * tasa_dudo + sin_ver as f64 * a_ciegas) / (no_sabes + sin_ver).max(1) as f64;
    let p = |s: u32, acierto| probabilidad(s as f64 / total, acierto, objetivo, examen);
    let mut meta = sabes;
    while meta < n && p(meta, None).max(p(meta, Some(tasa))) < META {
        meta += 1;
    }
    let faltan = meta - sabes;
    // El ritmo: las preguntas que pasaron a "sabes" en las últimas respuestas, por sesión.
    let mut ordenado = historial.to_vec();
    ordenado.sort_by_key(|r| r.ts);
    let antes = cuenta(banco, &ordenado[..ordenado.len().saturating_sub(VENTANA)]).0;
    let sesiones_de_la_ventana =
        if historial.is_empty() { 1.0 } else { historial.len().min(VENTANA) as f64 / TAMANO_SESION as f64 };
    let ritmo = (sabes as f64 - antes as f64) / sesiones_de_la_ventana;
    let (blanco, contesta) = (p(sabes, None), p(sabes, Some(tasa)));
    let pc = |x: f64| (100.0 * x).round() as u32;
    let ciegas = format!("a ciegas, 1 de {}", examen.opciones);
    let fila = |texto: &str, preguntas, tasa, detalle| Dudas { texto: texto.into(), preguntas, porcentaje: pc(tasa), detalle };
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

/// Las preguntas en "sabes", en "no sabes" y sin ver.
fn cuenta(banco: &[Pregunta], historial: &[Respuesta]) -> (u32, u32, u32) {
    let estado = estados(banco, historial);
    let sabes = estado.values().filter(|e| e.0.sabes()).count();
    (sabes as u32, (estado.len() - sabes) as u32, (banco.len() - estado.len()) as u32)
}

fn binomial(n: u32, k: u32, p: f64) -> f64 {
    let c = (0..k).fold(1.0, |c, i| c * (n - i) as f64 / (i + 1) as f64);
    c * p.powi(k as i32) * (1.0 - p).powi((n - k) as i32)
}

/// La probabilidad de sacar más de `objetivo` si cada pregunta está en "sabes" con probabilidad `sabes`.
/// Otra pregunta acierta con probabilidad `acierto`, o queda en blanco con `None`.
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

/// Las 2 columnas del PC, en partes del ancho de la ventana: el borde izquierdo, el separador y el borde derecho.
#[derive(Deserialize, Serialize, Tsify, Clone, Copy, PartialEq, Debug)]
pub struct Columnas {
    pub l: f64,
    pub s: f64,
    pub r: f64,
}

impl Default for Columnas {
    fn default() -> Self {
        Columnas { l: 0.08, s: 0.5, r: 0.92 }
    }
}

#[derive(Deserialize, Serialize, Tsify, Clone, Copy)]
pub enum Tirador {
    L,
    S,
    R,
}

const COLUMNA_MINIMA: f64 = 0.15;

/// Lleva un tirador a `x`, sin dejar una columna más estrecha que `COLUMNA_MINIMA`.
pub fn mover(c: Columnas, tirador: Tirador, x: f64) -> Columnas {
    match tirador {
        Tirador::L => Columnas { l: x.clamp(0.0, c.s - COLUMNA_MINIMA), ..c },
        Tirador::S => Columnas { s: x.clamp(c.l + COLUMNA_MINIMA, c.r - COLUMNA_MINIMA), ..c },
        Tirador::R => Columnas { r: x.clamp(c.s + COLUMNA_MINIMA, 1.0), ..c },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LN: Examen = Examen {
        titulo: String::new(),
        preguntas: 20,
        opciones: 4,
        minutos: 25,
        penalizacion: 0.25,
        pdfs: std::collections::BTreeMap::new(),
        sin_pregunta: std::collections::BTreeMap::new(),
        apartados: std::collections::BTreeMap::new(),
        congelado: false,
    };

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
        // 2 preguntas al 50 % y 7 al 25 %: 11/36.
        assert_eq!([0, 1, 2].map(|i| t.dudas[i].porcentaje), [50, 25, 31]);
    }

    #[test]
    fn la_tarjeta_sigue_la_forma_con_la_probabilidad_mas_alta_y_el_consejo_da_las_dos() {
        // 10 preguntas que sabes y 10 con un fallo con dudo: contestar solo resta, y una duda acierta 0 %.
        let respuestas: Vec<_> = (0..10).map(|i| (i, true, true)).chain((10..20).map(|i| (i, false, false))).collect();
        let t = tarjeta(&LN, &banco_de(20), &historial_de(&respuestas), 5.0);
        assert_eq!((t.hoy, t.si_contestas, t.en_blanco), (59, 25, 59));
        assert_eq!(t.consejo, "Deja las dudas en blanco");
    }

    #[test]
    fn el_camino_cuenta_las_preguntas_para_un_95_y_las_sesiones_salen_del_ritmo() {
        // Para un 10 con 20 preguntas solo valen 20 aciertos, así que el 95 % pide todas las preguntas.
        // 4 preguntas nuevas en 4 respuestas son 12 por sesión, y faltan 16.
        let h = historial_de(&[(0, true, true), (1, true, true), (2, true, true), (3, true, true)]);
        let t = tarjeta(&LN, &banco_de(20), &h, 10.0);
        assert_eq!((t.sabes, t.meta), (4, 20));
        assert_eq!(t.sesiones, "≈2 sesiones");
    }

    #[test]
    fn una_sola_sesion_va_en_singular() {
        // 19 preguntas nuevas en 19 respuestas son 12 por sesión, y falta 1.
        let respuestas: Vec<_> = (0..19).map(|i| (i, true, true)).collect();
        let t = tarjeta(&LN, &banco_de(20), &historial_de(&respuestas), 10.0);
        assert_eq!(t.sesiones, "≈1 sesión");
    }

    #[test]
    fn el_ritmo_cuenta_solo_las_60_ultimas_respuestas() {
        // 10 preguntas nuevas en las 60 últimas respuestas son 2 por sesión, y faltan 50.
        let respuestas: Vec<_> = (0..50).map(|i| (i, true, true)).chain((0..50).map(|_| (0, true, true))).collect();
        let t = tarjeta(&LN, &banco_de(100), &historial_de(&respuestas), 10.0);
        assert_eq!((t.sabes, t.meta), (50, 100));
        assert_eq!(t.sesiones, "≈25 sesiones");
    }

    #[test]
    fn con_un_95_hoy_no_faltan_preguntas_ni_sesiones() {
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
        // q0 sube a paso 2, q1 vuelve a paso 0 tras un acierto con dudo, y una respuesta huérfana no cuenta.
        let mut h = historial_de(&[(0, true, true), (0, true, true), (1, true, true), (1, true, false)]);
        h.push(respuesta("x", "1970-01-01T01:00:00Z", true, true));
        let t = tarjeta(&LN, &banco_de(3), &h, 5.0);
        assert_eq!((t.sabes, t.no_sabes, t.sin_ver), (1, 1, 1));
    }

    #[test]
    fn un_examen_de_3_opciones_sin_penalizacion_aconseja_contestar() {
        let asn = Examen { preguntas: 35, opciones: 3, minutos: 45, penalizacion: 0.0, ..LN };
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
                Dudas { texto: "en lo que viste".into(), preguntas: 0, porcentaje: 25, detalle: a_ciegas.clone() },
                Dudas { texto: "en lo que no viste".into(), preguntas: 20, porcentaje: 25, detalle: a_ciegas },
                Dudas { texto: "de media".into(), preguntas: 20, porcentaje: 25, detalle: None },
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
    fn el_simulacro_saca_al_azar_las_preguntas_del_examen_con_su_reloj() {
        let banco: Vec<Pregunta> = (0..90).map(|i| pregunta(&format!("q{i}"), &format!("c{}", i / 3))).collect();
        let s = simulacro(&LN, &banco, &[], &mut Azar::new(1));
        assert_eq!((s.segundos, s.preguntas.len()), (25 * 60, 20));
        let otro = simulacro(&LN, &banco, &[], &mut Azar::new(2));
        assert_ne!(
            s.preguntas.iter().map(|q| &q.id).collect::<Vec<_>>(),
            otro.preguntas.iter().map(|q| &q.id).collect::<Vec<_>>()
        );
        assert_eq!(simulacro(&LN, &banco[..5], &[], &mut Azar::new(1)).preguntas.len(), 5);
    }

    #[test]
    fn el_simulacro_lleva_una_pregunta_por_concepto_y_rellena_con_hermanas_cuando_faltan_conceptos() {
        let conceptos_de = |s: &Simulacro| {
            let mut c: Vec<&str> = s.preguntas.iter().map(|q| q.concepto.as_str()).collect();
            c.sort();
            c.dedup();
            c.len()
        };
        for semilla in 0..20 {
            // 30 conceptos de 3 preguntas: 20 conceptos distintos.
            let grande: Vec<Pregunta> = (0..90).map(|i| pregunta(&format!("q{i}"), &format!("c{}", i / 3))).collect();
            assert_eq!(conceptos_de(&simulacro(&LN, &grande, &[], &mut Azar::new(semilla))), 20);
            // 10 conceptos de 3 preguntas: los 10 conceptos, y 10 hermanas sin repetir pregunta.
            let corto = &grande[..30];
            let s = simulacro(&LN, corto, &[], &mut Azar::new(semilla));
            let mut ids: Vec<&str> = s.preguntas.iter().map(|q| q.id.as_str()).collect();
            ids.sort();
            ids.dedup();
            assert_eq!((conceptos_de(&s), ids.len()), (10, 20));
        }
    }

    #[test]
    fn una_pregunta_del_simulacro_da_su_paso_y_no_el_de_sus_hermanas() {
        let banco = [pregunta("a1", "a"), pregunta("a2", "a")];
        let historial = [respuesta("a1", &minuto(0, 0), true, true)];
        let s = simulacro(&LN, &banco, &historial, &mut Azar::new(1));
        let paso = |id: &str| s.preguntas.iter().find(|q| q.id == id).unwrap().paso.clone();
        assert_eq!((paso("a1").as_str(), paso("a2").as_str()), ("paso 3", "sin ver"));
    }

    pub(crate) fn pregunta(id: &str, concepto: &str) -> Pregunta {
        Pregunta {
            id: id.into(),
            concepto: concepto.into(),
            apartado: "rgpd".into(),
            pregunta: format!("¿{id}?"),
            correcta: "sí".into(),
            distractores: vec!["no".into(), "a veces".into(), "nunca".into()],
            explicacion: "porque sí".into(),
            fuente: Fuente { pdf: "2A _Teoría.pdf".into(), pagina: 7 },
        }
    }

    #[test]
    fn el_texto_para_copiar_lleva_la_pregunta_y_sus_opciones_en_orden_sin_la_solucion() {
        let q = sesion(&LN, &[pregunta("a1", "a")], &[], 0, &mut Azar::new(1)).remove(0);
        assert_eq!(q.opciones, ["a veces", "sí", "nunca", "no"]);
        assert_eq!(q.copiar, "¿a1?\n\nA) a veces\nB) sí\nC) nunca\nD) no");
    }

    #[test]
    fn un_tirador_deja_cada_columna_con_el_15_por_ciento_como_minimo() {
        let c = Columnas::default();
        assert_eq!(mover(c, Tirador::S, 0.3), Columnas { s: 0.3, ..c });
        assert_eq!(mover(c, Tirador::S, 0.0), Columnas { s: 0.08 + 0.15, ..c });
        assert_eq!(mover(c, Tirador::S, 1.0), Columnas { s: 0.92 - 0.15, ..c });
        assert_eq!(mover(c, Tirador::L, -0.2), Columnas { l: 0.0, ..c });
        assert_eq!(mover(c, Tirador::L, 0.9), Columnas { l: 0.5 - 0.15, ..c });
        assert_eq!(mover(c, Tirador::R, 1.4), Columnas { r: 1.0, ..c });
        assert_eq!(mover(c, Tirador::R, 0.1), Columnas { r: 0.5 + 0.15, ..c });
    }

    #[test]
    fn una_pregunta_editada_lleva_cada_opcion_a_su_sitio_del_banco_y_la_ronda_conserva_su_orden() {
        let banco = [pregunta("a1", "a")];
        let q = sesion(&LN, &banco, &[], 0, &mut Azar::new(1)).remove(0);
        assert_eq!(q.opciones, ["a veces", "sí", "nunca", "no"]);
        let e = Edicion {
            pregunta: "¿a1 bien?".into(),
            opciones: vec!["a ratos".into(), "claro".into(), "jamás".into(), "no".into()],
            explicacion: "porque claro".into(),
        };
        let r = editar(&banco, &q, &e).unwrap();
        assert_eq!((r.banco.pregunta.as_str(), r.banco.correcta.as_str()), ("¿a1 bien?", "claro"));
        assert_eq!(r.banco.distractores, ["no", "a ratos", "jamás"]);
        assert_eq!(r.banco.explicacion, "porque claro");
        assert_eq!((r.banco.id.as_str(), r.banco.concepto.as_str()), ("a1", "a"));
        assert_eq!((r.ronda.opciones, r.ronda.correcta), (e.opciones.clone(), 1));
        assert_eq!(r.ronda.copiar, "¿a1 bien?\n\nA) a ratos\nB) claro\nC) jamás\nD) no");
        assert_eq!(r.ronda.paso, q.paso);
        assert!(editar(&[pregunta("b1", "b")], &q, &e).is_none(), "sin la pregunta en el banco no hay edición");
    }

    #[test]
    fn la_pregunta_da_su_fuente_la_imagen_de_la_pagina_y_el_pdf_en_la_pagina() {
        let q = sesion(&LN, &[pregunta("a1", "a")], &[], 0, &mut Azar::new(1)).remove(0);
        assert_eq!(q.fuente, "2A _Teoría.pdf · pág. 7");
        assert_eq!(q.pagina, 7);
        assert_eq!(q.imagen, "paginas/2A _Teoría-7.webp");
        assert_eq!(q.pdf, "pdfs/2A _Teoría.pdf#page=7");
    }

    pub(crate) fn respuesta(id: &str, ts: &str, acierto: bool, seguro: bool) -> Respuesta {
        let resultado = if acierto { Resultado::Acierto } else { Resultado::Fallo };
        Respuesta { id: id.into(), ts: ts.parse().unwrap(), resultado, seguro }
    }

    #[test]
    fn el_inicio_da_el_titulo_y_las_preguntas_de_la_sesion() {
        let banco = [pregunta("a1", "a"), pregunta("a2", "a"), pregunta("b1", "b")];
        let examen = Examen { titulo: "LN".into(), ..LN };
        let Inicio::Listo { titulo, sesion, simulacro, .. } = inicio(&examen, &banco, &[], None, 5.0) else { panic!() };
        assert_eq!((titulo.as_str(), sesion, simulacro), ("LN", 2, 3));
        assert_eq!(inicio(&examen, &[], &[], None, 5.0), Inicio::BancoVacio { titulo: "LN".into() });
    }

    fn par(pdf: &str, apartado: &str) -> Par {
        Par { pdf: pdf.into(), apartado: apartado.into() }
    }

    /// 2 temas: «Uno» con RGPD, y «Dos» con ENS y Riesgos. Un concepto por apartado, y 2 preguntas en ENS.
    fn temario() -> (Examen, Vec<Pregunta>) {
        let tema = |nombre: &str, apartados: &[&str]| Tema { nombre: nombre.into(), apartados: apartados.iter().map(|a| a.to_string()).collect() };
        let examen = Examen {
            pdfs: [("1.pdf".into(), tema("Uno", &["rgpd"])), ("2.pdf".into(), tema("Dos", &["ens", "riesgos"]))].into(),
            apartados: [("rgpd", "RGPD"), ("ens", "ENS"), ("riesgos", "Riesgos")].map(|(a, n)| (a.into(), n.into())).into(),
            ..LN
        };
        let q = |id: &str, concepto: &str, pdf: &str, apartado: &str| Pregunta {
            apartado: apartado.into(),
            fuente: Fuente { pdf: pdf.into(), pagina: 1 },
            ..pregunta(id, concepto)
        };
        let banco = vec![q("a1", "a", "1.pdf", "rgpd"), q("b1", "b", "2.pdf", "ens"), q("b2", "b", "2.pdf", "ens"), q("c1", "c", "2.pdf", "riesgos")];
        (examen, banco)
    }

    #[test]
    fn sin_seleccion_todo_esta_marcado() {
        let (examen, banco) = temario();
        let Inicio::Listo { sesion, temas, parte, .. } = inicio(&examen, &banco, &[], None, 5.0) else { panic!() };
        assert_eq!((sesion, parte), (3, None));
        let resumen: Vec<_> = temas.iter().map(|t| (t.nombre.as_str(), t.preguntas, t.apartados.iter().all(|a| a.marcado))).collect();
        assert_eq!(resumen, [("Uno", 1, true), ("Dos", 3, true)]);
        assert_eq!(temas[1].apartados.iter().map(|a| (a.nombre.as_str(), a.preguntas)).collect::<Vec<_>>(), [("ENS", 2), ("Riesgos", 1)]);
    }

    #[test]
    fn cada_tema_y_cada_apartado_cuentan_las_preguntas_que_sabes() {
        let (examen, banco) = temario();
        let historial = [respuesta("b1", "2026-01-01T00:00:00Z", true, true), respuesta("c1", "2026-01-01T00:00:00Z", false, false)];
        let Inicio::Listo { temas, .. } = inicio(&examen, &banco, &historial, None, 5.0) else { panic!() };
        assert_eq!(temas.iter().map(|t| (t.sabes, t.preguntas)).collect::<Vec<_>>(), [(0, 1), (1, 3)]);
        assert_eq!(temas[1].apartados.iter().map(|a| (a.sabes, a.preguntas)).collect::<Vec<_>>(), [(1, 2), (0, 1)]);
    }

    #[test]
    fn la_seleccion_marca_sus_pares_y_las_rondas_salen_de_ella() {
        let (examen, banco) = temario();
        let seleccion = [par("2.pdf", "ens"), par("9.pdf", "viejo")];
        let Inicio::Listo { sesion, simulacro, tarjeta, temas, parte, .. } = inicio(&examen, &banco, &[], Some(&seleccion), 5.0) else { panic!() };
        assert_eq!((sesion, simulacro, tarjeta.sin_ver), (1, 2, 2));
        assert_eq!(parte.as_deref(), Some("con 1 de 2 temas"));
        assert_eq!(temas[1].apartados.iter().map(|a| a.marcado).collect::<Vec<_>>(), [true, false]);
        assert!(!temas[0].apartados[0].marcado);
        let ids: Vec<_> = de_la_seleccion(&banco, Some(&seleccion)).into_iter().map(|q| q.id).collect();
        assert_eq!(ids, ["b1", "b2"]);
    }

    #[test]
    fn la_pregunta_de_la_ronda_lleva_el_nombre_del_apartado() {
        let (examen, banco) = temario();
        assert_eq!(sesion(&examen, &banco[..1], &[], 0, &mut Azar::new(1))[0].apartado, "RGPD");
    }

    #[test]
    fn una_seleccion_vacia_da_rondas_de_0_preguntas() {
        let (examen, banco) = temario();
        let Inicio::Listo { sesion, simulacro, parte, .. } = inicio(&examen, &banco, &[], Some(&[]), 5.0) else { panic!() };
        assert_eq!((sesion, simulacro, parte.as_deref()), (0, 0, Some("con 0 de 2 temas")));
    }

    /// El paso de la única pregunta del banco, como lo muestra la sesión.
    fn paso(banco: &[Pregunta], historial: &[Respuesta]) -> String {
        sesion(&LN, banco, historial, 0, &mut Azar::new(1)).remove(0).paso
    }

    #[test]
    fn un_acierto_con_seguro_sube_el_paso_aunque_no_haya_vencido() {
        let banco = [pregunta("a1", "a")];
        let historial = [
            respuesta("a1", "2026-11-04T12:00:00Z", false, false),
            respuesta("a1", "2026-11-04T12:01:00Z", true, true),
            respuesta("a1", "2026-11-04T12:02:00Z", true, true),
        ];
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
    fn los_huecos_son_1_dia_y_despues_por_2_y_medio() {
        let horas = |n| Estado::Paso(n).hueco() / 3600;
        assert_eq!([0, 1, 2, 3].map(horas), [0, 24, 60, 150]);
        assert_eq!(Estado::Paso(u32::MAX).hueco(), i64::MAX);
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
        let mut ids: Vec<String> = sesion(&LN, banco, historial, AHORA.parse::<Timestamp>().unwrap().as_second(), &mut Azar::new(1))
            .into_iter()
            .map(|q| q.id)
            .collect();
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
    fn la_sesion_saca_primero_los_fallados_hace_mas_tiempo() {
        let banco = conceptos("v", 15);
        assert_eq!(de_la_sesion(&banco, &fallos(&banco)), ids(&banco[..12]));
    }

    #[test]
    fn con_12_fallados_la_sesion_no_lleva_conceptos_sin_ver() {
        let fallados = conceptos("f", 12);
        let sin_ver = conceptos("s", 10);
        assert_eq!(de_la_sesion(&[fallados.clone(), sin_ver].concat(), &fallos(&fallados)), ids(&fallados));
    }

    #[test]
    fn los_fallados_van_antes_que_los_vencidos_con_paso_y_estos_antes_que_los_sin_ver() {
        let fallados = conceptos("f", 3);
        let vencidos = conceptos("v", 20);
        let sin_ver = conceptos("s", 5);
        // Los vencidos con paso 1, fallados y acertados el día antes, vencieron antes que los fallos, y aun así los fallados entran primero.
        let mut historial: Vec<Respuesta> = vencidos
            .iter()
            .enumerate()
            .flat_map(|(i, q)| {
                [respuesta(&q.id, &format!("2026-11-03T11:{i:02}:00Z"), false, false), respuesta(&q.id, &format!("2026-11-03T11:{i:02}:30Z"), true, true)]
            })
            .collect();
        historial.extend(fallados.iter().enumerate().map(|(i, q)| respuesta(&q.id, &minuto(40, i as i64), false, false)));
        let banco = [fallados.clone(), vencidos.clone(), sin_ver].concat();
        assert_eq!(de_la_sesion(&banco, &historial), ids(&[fallados, vencidos[..9].to_vec()].concat()));
    }

    #[test]
    fn los_huecos_que_sobran_se_llenan_con_sin_ver_y_despues_con_los_que_vencen_antes() {
        let vencidos = conceptos("v", 2);
        let sin_ver = conceptos("s", 2);
        let al_dia = conceptos("d", 20);
        let mut historial = fallos(&vencidos);
        historial.extend(al_dia.iter().enumerate().map(|(i, q)| respuesta(&q.id, &minuto(55, i as i64), true, true)));
        let banco = [vencidos.clone(), sin_ver.clone(), al_dia.clone()].concat();
        assert_eq!(de_la_sesion(&banco, &historial), ids(&[vencidos, sin_ver, al_dia[..8].to_vec()].concat()));
    }

    #[test]
    fn la_sesion_lleva_una_pregunta_por_concepto() {
        let banco = [pregunta("a1", "a"), pregunta("a2", "a"), pregunta("a3", "a"), pregunta("b1", "b")];
        let sesion = sesion(&LN, &banco, &[], 0, &mut Azar::new(1));
        let mut conceptos: Vec<&str> = sesion.iter().map(|q| q.concepto.as_str()).collect();
        conceptos.sort();
        assert_eq!(conceptos, ["a", "b"]);
    }

    #[test]
    fn una_pregunta_fallada_vuelve_aunque_su_hermana_suba_de_paso() {
        // a1 falla y a2 acierta: a1 sigue fallada, y entra en la sesión antes que su hermana vencida.
        let banco = [pregunta("a1", "a"), pregunta("a2", "a")];
        let historial = [respuesta("a2", &minuto(0, 0), true, true), respuesta("a1", &minuto(1, 0), false, false)];
        let ahora = AHORA.parse::<Timestamp>().unwrap().as_second();
        for semilla in 0..20 {
            let q = sesion(&LN, &banco, &historial, ahora, &mut Azar::new(semilla)).remove(0);
            assert_eq!((q.id.as_str(), q.paso.as_str()), ("a1", "paso 0"));
        }
    }

    #[test]
    fn una_hermana_sin_ver_entra_antes_que_un_concepto_sin_ver() {
        // a1 está al día, y 12 conceptos sin ver compiten con a2 por los 12 huecos de la sesión.
        let banco = [vec![pregunta("a1", "a"), pregunta("a2", "a")], conceptos("b", 12)].concat();
        let historial = [respuesta("a1", &minuto(59, 0), true, true)];
        let ahora = AHORA.parse::<Timestamp>().unwrap().as_second();
        for semilla in 0..20 {
            let s = sesion(&LN, &banco, &historial, ahora, &mut Azar::new(semilla));
            assert!(s.iter().any(|q| q.id == "a2"), "semilla {semilla}");
            assert_eq!(s.len(), 12);
        }
    }

    #[test]
    fn con_300_conceptos_acertados_en_el_mismo_minuto_salen_los_12_que_vencen_antes() {
        let banco = conceptos("c", 300);
        let historial: Vec<Respuesta> =
            banco.iter().enumerate().map(|(i, q)| respuesta(&q.id, &minuto(0, i as i64 / 5), true, true)).collect();
        let ahora = minuto(5, 0).parse::<Timestamp>().unwrap().as_second();
        let sesion = sesion(&LN, &banco, &historial, ahora, &mut Azar::new(1));
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
            |semilla| sesion(&LN, &banco, &[], 0, &mut Azar::new(semilla)).into_iter().map(|q| (q.id, q.opciones)).collect::<Vec<_>>();
        assert_eq!(orden(7), orden(7));
        assert_ne!(orden(7), orden(8));
    }

    #[test]
    fn la_pregunta_da_las_opciones_al_azar_la_correcta_y_el_paso() {
        let banco = [pregunta("a1", "a")];
        let historial = [respuesta("a1", &minuto(0, 0), true, true)];
        let q = &sesion(&LN, &banco, &historial, 0, &mut Azar::new(3))[0];
        assert_eq!(q.opciones[q.correcta as usize], "sí");
        let mut opciones = q.opciones.clone();
        opciones.sort();
        assert_eq!(opciones, ["a veces", "no", "nunca", "sí"]);
        assert_eq!((q.paso.as_str(), q.paso_tras_seguro.as_str(), q.paso_tras_otra.as_str()), ("paso 3", "paso 4", "paso 0"));
        let sin_ver = &sesion(&LN, &[pregunta("b1", "b")], &[], 0, &mut Azar::new(3))[0];
        assert_eq!((sin_ver.paso.as_str(), sin_ver.paso_tras_seguro.as_str()), ("sin ver", "paso 3"));
    }

    #[test]
    fn corregir_compara_la_opcion_con_la_correcta_y_sin_opcion_es_en_blanco() {
        let q = &sesion(&LN, &[pregunta("a1", "a")], &[], 0, &mut Azar::new(3))[0];
        let otra = (q.correcta + 1) % 4;
        let r = corregir(q, Some(q.correcta), true, "2026-11-04T12:41:22Z".parse().unwrap()).respuesta;
        assert_eq!(
            r,
            Respuesta {
                id: "a1".into(),
                ts: "2026-11-04T12:41:22Z".parse().unwrap(),
                resultado: Resultado::Acierto,
                seguro: true
            }
        );
        assert_eq!(corregir(q, Some(otra), false, Timestamp::UNIX_EPOCH).respuesta.resultado, Resultado::Fallo);
        let blanco = corregir(q, None, true, Timestamp::UNIX_EPOCH).respuesta;
        assert_eq!((blanco.resultado, blanco.seguro), (Resultado::Blanco, false));
    }

    #[test]
    fn corregir_da_el_paso_tras_la_respuesta_y_avisa_de_un_fallo_seguro_y_un_acierto_dudoso() {
        let q = &sesion(&LN, &[pregunta("a1", "a")], &[], 0, &mut Azar::new(3))[0];
        let otra = (q.correcta + 1) % 4;
        let fila = |c: Correccion| (c.paso, c.aviso);
        assert_eq!(fila(corregir(q, Some(q.correcta), true, Timestamp::UNIX_EPOCH)), ("paso 3".into(), None));
        assert_eq!(
            fila(corregir(q, Some(q.correcta), false, Timestamp::UNIX_EPOCH)),
            ("paso 0".into(), Some("Acierto con «dudo»: cuenta como un fallo.".into()))
        );
        assert_eq!(
            fila(corregir(q, Some(otra), true, Timestamp::UNIX_EPOCH)),
            ("paso 0".into(), Some("Fallo con «seguro»: una idea falsa. Repásala.".into()))
        );
        assert_eq!(fila(corregir(q, Some(otra), false, Timestamp::UNIX_EPOCH)), ("paso 0".into(), None));
        assert_eq!(fila(corregir(q, None, true, Timestamp::UNIX_EPOCH)), ("paso 0".into(), None));
    }

    #[test]
    fn el_reloj_cuenta_los_segundos_que_quedan_hasta_0() {
        assert_eq!(restante(1_060_000.0, 1_000_000.0), 60);
        assert_eq!(restante(1_060_000.0, 1_030_400.0), 30);
        assert_eq!(restante(1_060_000.0, 2_000_000.0), 0);
    }

    #[test]
    fn una_pregunta_en_blanco_pone_el_paso_a_0_y_no_cuenta_como_duda() {
        let mut h = historial_de(&[(0, true, true), (1, true, false)]);
        h.push(Respuesta { resultado: Resultado::Blanco, ..respuesta("q0", "1970-01-01T01:00:00Z", false, false) });
        let t = tarjeta(&LN, &banco_de(2), &h, 5.0);
        assert_eq!((t.sabes, t.no_sabes), (0, 2));
        assert_eq!(t.dudas[0].detalle.as_deref(), Some("1 de 1 «dudo»"));
    }

    fn guardado(guardada: i64, ids: &[&str], resultados: &[Option<Resultado>]) -> Guardado {
        let ids = ids.iter().map(|&i| i.into()).collect();
        Guardado {
            guardada: Timestamp::from_second(guardada).unwrap(),
            ronda: Some(RondaGuardada { ids, resultados: resultados.to_vec(), respuestas: vec![], simulacro: false, limite: None }),
            seleccion: None,
        }
    }

    #[test]
    fn gana_el_guardado_mas_nuevo() {
        let viejo = guardado(1, &["a1"], &[]);
        let nuevo = Guardado { guardada: Timestamp::from_second(2).unwrap(), ronda: None, seleccion: None };
        assert_eq!(gana(viejo.clone(), nuevo.clone()), nuevo);
        assert_eq!(gana(nuevo.clone(), viejo.clone()), nuevo);
        assert_eq!(gana(viejo.clone(), guardado(1, &["b1"], &[])), viejo);
    }

    #[test]
    fn la_ronda_restaurada_sigue_en_la_primera_pregunta_sin_resultado() {
        let banco = [pregunta("a1", "a"), pregunta("b1", "b"), pregunta("c1", "c")];
        let g = guardado(1, &["c1", "a1", "b1"], &[Some(Resultado::Fallo)]);
        let r = restaurar(&LN, &banco, &[], &g, &mut Azar::new(1)).unwrap();
        assert_eq!(r.preguntas.iter().map(|q| q.id.as_str()).collect::<Vec<_>>(), ["c1", "a1", "b1"]);
        assert_eq!(r.resultados, [Some(Resultado::Fallo), None, None]);
        assert_eq!((r.simulacro, r.limite), (false, None));
    }

    #[test]
    fn la_ronda_restaurada_guarda_cada_resultado_con_su_pregunta_aunque_haya_saltos() {
        let banco = [pregunta("a1", "a"), pregunta("b1", "b"), pregunta("c1", "c")];
        let g = guardado(1, &["c1", "a1", "b1"], &[None, Some(Resultado::Acierto), None]);
        let r = restaurar(&LN, &banco, &[], &g, &mut Azar::new(1)).unwrap();
        assert_eq!(r.resultados, [None, Some(Resultado::Acierto), None]);
        assert!(restaurar(&LN, &banco, &[], &guardado(1, &["a1"], &[None, None]), &mut Azar::new(1)).is_none());
    }

    #[test]
    fn el_simulacro_restaurado_lleva_sus_respuestas_fuera_del_historial() {
        let banco = [pregunta("a1", "a"), pregunta("b1", "b")];
        let mut g = guardado(1, &["a1", "b1"], &[Some(Resultado::Acierto)]);
        let respuestas = vec![respuesta("a1", "1970-01-01T00:00:05Z", true, true)];
        let ronda = g.ronda.as_mut().unwrap();
        ronda.simulacro = true;
        ronda.respuestas = respuestas.clone();
        let r = restaurar(&LN, &banco, &[], &g, &mut Azar::new(1)).unwrap();
        assert_eq!(r.respuestas, respuestas);
    }

    #[test]
    fn una_ronda_con_una_pregunta_que_no_esta_en_el_banco_no_se_restaura() {
        let banco = [pregunta("a1", "a")];
        assert!(restaurar(&LN, &banco, &[], &guardado(1, &["a1", "x9"], &[]), &mut Azar::new(1)).is_none());
        assert!(
            restaurar(&LN, &banco, &[], &Guardado { guardada: Timestamp::from_second(1).unwrap(), ronda: None, seleccion: None }, &mut Azar::new(1))
                .is_none()
        );
    }

    #[test]
    fn la_union_de_2_historiales_va_en_orden_de_hora_sin_repeticiones() {
        let pc = [respuesta("a1", "2026-01-01T10:00:00Z", true, true), respuesta("b1", "2026-01-01T12:00:00Z", false, true)];
        let movil = [respuesta("c1", "2026-01-01T11:00:00Z", true, false), pc[1].clone()];
        let u = unir(&pc, &movil);
        assert_eq!(u.iter().map(|r| r.id.as_str()).collect::<Vec<_>>(), ["a1", "c1", "b1"]);
        assert_eq!(unir(&movil, &pc), u);
    }
}
