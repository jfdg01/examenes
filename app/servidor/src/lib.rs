//! El servidor guarda y sirve los archivos de cada examen. Valida con los tipos de `nucleo` lo que recibe.

// Un handler devuelve `Result<_, Response>`, y un `Response` de axum pesa más de 128 bytes.
// Copiar esos bytes en un error no se nota en un servidor local.
#![allow(clippy::result_large_err)]

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use nucleo::{Datos, Examen, Guardado, Pregunta, Respuesta};
use tower_http::compression::CompressionLayer;
use tower_http::services::ServeDir;

struct Carpetas {
    /// La carpeta con una carpeta por examen.
    raiz: PathBuf,
    /// La build de la interfaz, con `index.html` y `assets/`.
    interfaz: PathBuf,
    /// El cerrojo del historial y de la ronda de todos los exámenes.
    /// ponytail: un solo cerrojo, uno por examen si un día hay muchos usuarios.
    cerrojo: tokio::sync::Mutex<()>,
}

type Estado = State<Arc<Carpetas>>;

pub fn app(raiz: PathBuf, interfaz: PathBuf) -> Router {
    let assets = ServeDir::new(interfaz.join("assets"));
    Router::new()
        .route("/{examen}", get(|Path(examen): Path<String>| async move { Redirect::permanent(&format!("/{examen}/")) }))
        .route("/{examen}/", get(pagina))
        .route("/{examen}/api/inicio", get(inicio))
        .route("/{examen}/api/respuestas", axum::routing::post(responder))
        .route("/{examen}/api/ronda", get(leer_ronda).put(guardar_ronda))
        .nest_service("/assets", assets)
        .with_state(Arc::new(Carpetas { raiz, interfaz, cerrojo: Default::default() }))
        .layer(CompressionLayer::new().gzip(true))
}

/// Un examen es una carpeta de la raíz con `examen.json`, y se lee en cada petición.
async fn leer_examen(carpetas: &Carpetas, examen: &str) -> Result<Examen, Response> {
    // El nombre es una sola carpeta de la raíz: sin `/`, sin `..` y sin las carpetas ocultas.
    let mut partes = std::path::Path::new(examen).components();
    let una_carpeta = matches!((partes.next(), partes.next()), (Some(std::path::Component::Normal(_)), None));
    if !una_carpeta || examen.starts_with('.') {
        return Err(StatusCode::NOT_FOUND.into_response());
    }
    let texto = tokio::fs::read(carpetas.raiz.join(examen).join("examen.json"))
        .await
        .map_err(|_| StatusCode::NOT_FOUND.into_response())?;
    serde_json::from_slice(&texto).map_err(error_interno)
}

async fn pagina(State(carpetas): Estado, Path(examen): Path<String>) -> Result<Html<Vec<u8>>, Response> {
    leer_examen(&carpetas, &examen).await?;
    let html = tokio::fs::read(carpetas.interfaz.join("index.html")).await.map_err(error_interno)?;
    Ok(Html(html))
}

/// Un archivo que no existe es un archivo vacío.
async fn leer_si_existe(ruta: PathBuf) -> Result<Vec<u8>, Response> {
    match tokio::fs::read(ruta).await {
        Ok(texto) => Ok(texto),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(error_interno(e)),
    }
}

async fn leer_banco(carpetas: &Carpetas, examen: &str) -> Result<Vec<Pregunta>, Response> {
    let texto = leer_si_existe(carpetas.raiz.join(examen).join("preguntas.json")).await?;
    if texto.is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_slice(&texto).map_err(error_interno)
}

/// `respuestas.jsonl` tiene una respuesta por línea. Quien llama tiene el cerrojo.
async fn leer_historial(carpetas: &Carpetas, examen: &str) -> Result<Vec<Respuesta>, Response> {
    let texto = leer_si_existe(carpetas.raiz.join(examen).join("respuestas.jsonl")).await?;
    texto
        .split(|&b| b == b'\n')
        .filter(|linea| !linea.trim_ascii().is_empty())
        .map(|linea| serde_json::from_slice(linea).map_err(error_interno))
        .collect()
}

/// Las cifras, el banco y el historial del examen. La interfaz calcula todo con `nucleo`.
async fn inicio(State(carpetas): Estado, Path(examen): Path<String>) -> Result<axum::Json<Datos>, Response> {
    let datos = leer_examen(&carpetas, &examen).await?;
    let banco = leer_banco(&carpetas, &examen).await?;
    let _cerrojo = carpetas.cerrojo.lock().await;
    let historial = leer_historial(&carpetas, &examen).await?;
    Ok(axum::Json(Datos { examen: datos, banco, historial }))
}

/// Añade al historial cada respuesta nueva. Una respuesta con el mismo `id` y `ts` ya está, y no entra otra vez.
async fn responder(
    State(carpetas): Estado,
    Path(examen): Path<String>,
    axum::Json(respuestas): axum::Json<Vec<Respuesta>>,
) -> Result<StatusCode, Response> {
    leer_examen(&carpetas, &examen).await?;
    if respuestas.iter().any(|r| nucleo::segundos(&r.ts).is_none()) {
        return Err((StatusCode::BAD_REQUEST, "La hora de una respuesta va como AAAA-MM-DDTHH:MM:SSZ").into_response());
    }
    let _cerrojo = carpetas.cerrojo.lock().await;
    let mut vistas: std::collections::HashSet<(String, String)> =
        leer_historial(&carpetas, &examen).await?.into_iter().map(|r| (r.id, r.ts)).collect();
    let mut nuevas = String::new();
    for r in respuestas {
        if vistas.insert((r.id.clone(), r.ts.clone())) {
            nuevas += &serde_json::to_string(&r).map_err(error_interno)?;
            nuevas += "\n";
        }
    }
    if !nuevas.is_empty() {
        use tokio::io::AsyncWriteExt;
        let ruta = carpetas.raiz.join(&examen).join("respuestas.jsonl");
        let mut archivo = tokio::fs::OpenOptions::new().create(true).append(true).open(ruta).await.map_err(error_interno)?;
        archivo.write_all(nuevas.as_bytes()).await.map_err(error_interno)?;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Sin `ronda.json`, no hay ronda a medias.
async fn leer_guardado(ruta: PathBuf) -> Result<Guardado, Response> {
    let texto = leer_si_existe(ruta).await?;
    if texto.is_empty() {
        return Ok(Guardado { guardada: 0, ronda: None });
    }
    serde_json::from_slice(&texto).map_err(error_interno)
}

async fn leer_ronda(State(carpetas): Estado, Path(examen): Path<String>) -> Result<axum::Json<Guardado>, Response> {
    leer_examen(&carpetas, &examen).await?;
    let _cerrojo = carpetas.cerrojo.lock().await;
    leer_guardado(carpetas.raiz.join(&examen).join("ronda.json")).await.map(axum::Json)
}

/// Guarda el guardado más nuevo entre `ronda.json` y el que llega, porque dos `PUT` pueden llegar en desorden.
async fn guardar_ronda(
    State(carpetas): Estado,
    Path(examen): Path<String>,
    axum::Json(guardado): axum::Json<Guardado>,
) -> Result<StatusCode, Response> {
    leer_examen(&carpetas, &examen).await?;
    let ruta = carpetas.raiz.join(&examen).join("ronda.json");
    let _cerrojo = carpetas.cerrojo.lock().await;
    let gana = nucleo::gana(leer_guardado(ruta.clone()).await?, guardado);
    let texto = serde_json::to_vec(&gana).map_err(error_interno)?;
    tokio::fs::write(ruta, texto).await.map_err(error_interno)?;
    Ok(StatusCode::NO_CONTENT)
}

fn error_interno(e: impl std::fmt::Display) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
}
