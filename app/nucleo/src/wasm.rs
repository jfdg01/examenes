//! Las funciones de `nucleo` que llama la interfaz. Una hora va en milisegundos desde 1970, como `Date.now()`.
//! Un dato que no encaja con su tipo lanza un error en JavaScript.

use serde::Serialize;
use serde::de::DeserializeOwned;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::{JsError, wasm_bindgen};

use crate::{Cierre, Datos, Inicio, NotaDelSimulacro, PreguntaDeSesion, Respuesta, Simulacro};

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

fn en_segundos(ahora: f64) -> i64 {
    (ahora / 1000.0) as i64
}

/// La semilla del azar es la hora.
fn azar(ahora: f64) -> crate::Azar {
    crate::Azar::new(ahora as u64)
}

#[wasm_bindgen]
pub fn inicio(datos: Ts<Datos>, objetivo: f64) -> Js<Inicio> {
    let d = leer(datos)?;
    dar(crate::inicio(&d.examen, &d.banco, &d.historial, objetivo))
}

#[wasm_bindgen]
pub fn sesion(datos: Ts<Datos>, ahora: f64) -> Result<Vec<Ts<PreguntaDeSesion>>, JsError> {
    let d = leer(datos)?;
    crate::sesion(&d.banco, &d.historial, en_segundos(ahora), &mut azar(ahora)).into_iter().map(dar).collect()
}

#[wasm_bindgen]
pub fn simulacro(datos: Ts<Datos>, ahora: f64) -> Js<Simulacro> {
    let d = leer(datos)?;
    dar(crate::simulacro(&d.examen, &d.banco, &d.historial, &mut azar(ahora)))
}

#[wasm_bindgen]
pub fn corregir(pregunta: Ts<PreguntaDeSesion>, opcion: Option<u32>, seguro: bool, ahora: f64) -> Js<Respuesta> {
    dar(crate::corregir(&leer(pregunta)?, opcion, seguro, en_segundos(ahora)))
}

#[wasm_bindgen]
pub fn restante(empieza: f64, segundos: u32, ahora: f64) -> u32 {
    crate::restante(empieza, segundos, ahora)
}

#[wasm_bindgen]
pub fn nota(cierre: Ts<Cierre>, penalizacion: f64) -> Js<NotaDelSimulacro> {
    dar(crate::nota_del_simulacro(&leer(cierre)?, penalizacion))
}
