//! Las funciones de `nucleo` que llama la interfaz. `ahora` va en milisegundos desde 1970, como `Date.now()`.
//! Un dato que no encaja con su tipo lanza un error en JavaScript.

use serde::Serialize;
use serde::de::DeserializeOwned;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::{JsError, wasm_bindgen};

use crate::desarrollo::{
    CifraDeDesarrollo, DesarrolloAMedias, LineaDeDesarrollo, NotaDeDesarrollo, PreguntaDeDesarrolloDeSesion,
};
use crate::{
    Cierre, Columnas, Correccion, Datos, Edicion, Editada, Examen, Guardado, Inicio, NotaDelSimulacro, Par, PreguntaDeSesion,
    Respuesta, RondaAMedias, Simulacro, Tirador,
};

type Js<T> = Result<Ts<T>, JsError>;

fn leer<T: Tsify + DeserializeOwned>(js: Ts<T>) -> Result<T, JsError>
where
    T::JsType: Clone,
{
    Ok(js.to_rust()?)
}

fn dar<T: Tsify + Serialize>(rust: T) -> Js<T> {
    Ok(Ts::from_rust(&rust)?)
}

/// Los pares marcados. Sin selección, todo está marcado.
fn pares(seleccion: Option<Vec<Ts<Par>>>) -> Result<Option<Vec<Par>>, JsError> {
    seleccion.map(|s| s.into_iter().map(leer).collect()).transpose()
}

/// Los datos con solo los bancos de la selección.
fn de_la_seleccion(datos: Ts<Datos>, seleccion: Option<Vec<Ts<Par>>>) -> Result<Datos, JsError> {
    let d = leer(datos)?;
    let s = pares(seleccion)?;
    Ok(Datos {
        banco: crate::de_la_seleccion(&d.examen, &d.banco, &d.conceptos, s.as_deref()),
        desarrollo: crate::desarrollo::desarrollo_de_la_seleccion(&d.examen, &d.desarrollo, &d.conceptos, s.as_deref()),
        ..d
    })
}

fn en_segundos(ahora: f64) -> i64 {
    (ahora / 1000.0) as i64
}

/// La semilla del azar es la hora.
fn azar(ahora: f64) -> crate::Azar {
    crate::Azar::new(ahora as u64)
}

#[wasm_bindgen]
pub fn inicio(datos: Ts<Datos>, seleccion: Option<Vec<Ts<Par>>>, objetivo: f64, ahora: f64) -> Js<Inicio> {
    let d = leer(datos)?;
    dar(crate::inicio(
        &d.examen,
        &d.banco,
        &d.conceptos,
        &d.historial,
        pares(seleccion)?.as_deref(),
        objetivo,
        en_segundos(ahora),
    ))
}

#[wasm_bindgen]
pub fn sesion(datos: Ts<Datos>, seleccion: Option<Vec<Ts<Par>>>, ahora: f64) -> Result<Vec<Ts<PreguntaDeSesion>>, JsError> {
    let d = de_la_seleccion(datos, seleccion)?;
    crate::sesion(&d.examen, &d.banco, &d.conceptos, &d.historial, en_segundos(ahora), &mut azar(ahora))
        .into_iter()
        .map(dar)
        .collect()
}

#[wasm_bindgen]
pub fn simulacro(datos: Ts<Datos>, seleccion: Option<Vec<Ts<Par>>>, ahora: f64) -> Js<Simulacro> {
    let d = de_la_seleccion(datos, seleccion)?;
    dar(crate::simulacro(&d, en_segundos(ahora), &mut azar(ahora)))
}

#[wasm_bindgen]
pub fn corregir(pregunta: Ts<PreguntaDeSesion>, opcion: Option<u32>, seguro: bool, ahora: f64) -> Js<Correccion> {
    dar(crate::corregir(&leer(pregunta)?, opcion, seguro, jiff::Timestamp::from_second(en_segundos(ahora))?))
}

#[wasm_bindgen]
pub fn editar(datos: Ts<Datos>, pregunta: Ts<PreguntaDeSesion>, edicion: Ts<Edicion>) -> Result<Option<Ts<Editada>>, JsError> {
    crate::editar(&leer(datos)?.banco, &leer(pregunta)?, &leer(edicion)?).map(dar).transpose()
}

#[wasm_bindgen]
pub fn restante(limite: f64, ahora: f64) -> u32 {
    crate::restante(limite, ahora)
}

#[wasm_bindgen]
pub fn nota(cierre: Ts<Cierre>, examen: Ts<Examen>) -> Js<NotaDelSimulacro> {
    dar(crate::nota_del_simulacro(&leer(cierre)?, &leer(examen)?))
}

#[wasm_bindgen]
pub fn gana(a: Ts<Guardado>, b: Ts<Guardado>) -> Js<Guardado> {
    dar(crate::gana(leer(a)?, leer(b)?))
}

#[wasm_bindgen]
pub fn unir(a: Vec<Ts<Respuesta>>, b: Vec<Ts<Respuesta>>) -> Result<Vec<Ts<Respuesta>>, JsError> {
    let a: Vec<_> = a.into_iter().map(leer).collect::<Result<_, _>>()?;
    let b: Vec<_> = b.into_iter().map(leer).collect::<Result<_, _>>()?;
    crate::unir(&a, &b).into_iter().map(dar).collect()
}

#[wasm_bindgen]
pub fn restaurar(datos: Ts<Datos>, guardado: Ts<Guardado>, ahora: f64) -> Result<Option<Ts<RondaAMedias>>, JsError> {
    crate::restaurar(&leer(datos)?, &leer(guardado)?, en_segundos(ahora), &mut azar(ahora)).map(dar).transpose()
}

/// Las preguntas de desarrollo de la selección. Sin banco de desarrollo, `undefined`.
#[wasm_bindgen]
pub fn cifra_de_desarrollo(
    datos: Ts<Datos>,
    seleccion: Option<Vec<Ts<Par>>>,
    ahora: f64,
) -> Result<Option<Ts<CifraDeDesarrollo>>, JsError> {
    let d = leer(datos)?;
    crate::desarrollo::cifra_de_desarrollo(
        &d.examen,
        &d.desarrollo,
        &d.conceptos,
        &d.historial_de_desarrollo,
        pares(seleccion)?.as_deref(),
        en_segundos(ahora),
    )
    .map(dar)
    .transpose()
}

#[wasm_bindgen]
pub fn sesion_de_desarrollo(
    datos: Ts<Datos>,
    seleccion: Option<Vec<Ts<Par>>>,
    ahora: f64,
) -> Result<Vec<Ts<PreguntaDeDesarrolloDeSesion>>, JsError> {
    let d = leer(datos)?;
    let elegidas =
        crate::desarrollo::desarrollo_de_la_seleccion(&d.examen, &d.desarrollo, &d.conceptos, pares(seleccion)?.as_deref());
    crate::desarrollo::sesion_de_desarrollo(
        &d.examen,
        &elegidas,
        &d.conceptos,
        &d.historial_de_desarrollo,
        en_segundos(ahora),
        &mut azar(ahora),
    )
    .into_iter()
    .map(dar)
    .collect()
}

#[wasm_bindgen]
pub fn calificar(
    id: String,
    texto: String,
    nota: Ts<NotaDeDesarrollo>,
    ahora: f64,
) -> Result<Vec<Ts<LineaDeDesarrollo>>, JsError> {
    let ahora = jiff::Timestamp::from_second(en_segundos(ahora))?;
    crate::desarrollo::calificar(&id, &texto, leer(nota)?, ahora).into_iter().map(dar).collect()
}

#[wasm_bindgen]
pub fn responder(id: String, texto: String, ahora: f64) -> Result<Vec<Ts<LineaDeDesarrollo>>, JsError> {
    let ahora = jiff::Timestamp::from_second(en_segundos(ahora))?;
    crate::desarrollo::responder(&id, &texto, ahora).into_iter().map(dar).collect()
}

/// La corrección de la respuesta de `id` dada en `ts`, el texto de su línea de respuesta.
#[wasm_bindgen]
pub fn corregir_respuesta(id: String, ts: String, nota: Ts<NotaDeDesarrollo>, ahora: f64) -> Js<LineaDeDesarrollo> {
    let ahora = jiff::Timestamp::from_second(en_segundos(ahora))?;
    dar(crate::desarrollo::corregir_respuesta(&id, ts.parse()?, leer(nota)?, ahora))
}

#[wasm_bindgen]
pub fn unir_desarrollo(
    a: Vec<Ts<LineaDeDesarrollo>>,
    b: Vec<Ts<LineaDeDesarrollo>>,
) -> Result<Vec<Ts<LineaDeDesarrollo>>, JsError> {
    let a: Vec<_> = a.into_iter().map(leer).collect::<Result<_, _>>()?;
    let b: Vec<_> = b.into_iter().map(leer).collect::<Result<_, _>>()?;
    crate::desarrollo::unir_desarrollo(&a, &b).into_iter().map(dar).collect()
}

#[wasm_bindgen]
pub fn restaurar_desarrollo(
    datos: Ts<Datos>,
    guardado: Ts<Guardado>,
    ahora: f64,
) -> Result<Option<Ts<DesarrolloAMedias>>, JsError> {
    crate::desarrollo::restaurar_desarrollo(&leer(datos)?, &leer(guardado)?, en_segundos(ahora)).map(dar).transpose()
}

/// Las columnas del PC antes de mover un tirador.
#[wasm_bindgen]
pub fn columnas() -> Js<Columnas> {
    dar(Columnas::default())
}

#[wasm_bindgen]
pub fn mover(columnas: Ts<Columnas>, tirador: Ts<Tirador>, x: f64) -> Js<Columnas> {
    dar(crate::mover(leer(columnas)?, leer(tirador)?, x))
}
