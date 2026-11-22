//! Las funciones de `nucleo` que llama la interfaz. `ahora` va en milisegundos desde 1970, como `Date.now()`.
//! Un dato que no encaja con su tipo lanza un error en JavaScript.

use serde::Serialize;
use serde::de::DeserializeOwned;
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::{JsError, wasm_bindgen};

use crate::{
    Cierre, Columnas, Correccion, Datos, Guardado, Inicio, NotaDelSimulacro, PreguntaDeSesion, Respuesta, RondaAMedias,
    Simulacro, Tirador,
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
pub fn corregir(pregunta: Ts<PreguntaDeSesion>, opcion: Option<u32>, seguro: bool, ahora: f64) -> Js<Correccion> {
    dar(crate::corregir(&leer(pregunta)?, opcion, seguro, jiff::Timestamp::from_second(en_segundos(ahora))?))
}

#[wasm_bindgen]
pub fn restante(limite: f64, ahora: f64) -> u32 {
    crate::restante(limite, ahora)
}

#[wasm_bindgen]
pub fn nota(cierre: Ts<Cierre>, penalizacion: f64) -> Js<NotaDelSimulacro> {
    dar(crate::nota_del_simulacro(&leer(cierre)?, penalizacion))
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
    let d = leer(datos)?;
    crate::restaurar(&d.banco, &d.historial, &leer(guardado)?, &mut azar(ahora)).map(dar).transpose()
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
