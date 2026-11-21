//! El servidor lee la carpeta de cada examen y llama a `nucleo`.

use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use nucleo::{Examen, Pregunta};
use tower_http::services::ServeDir;

struct Carpetas {
    /// La carpeta con una carpeta por examen.
    raiz: PathBuf,
    /// La build de la interfaz, con `index.html` y `assets/`.
    interfaz: PathBuf,
}

type Estado = State<Arc<Carpetas>>;

pub fn app(raiz: PathBuf, interfaz: PathBuf) -> Router {
    let assets = ServeDir::new(interfaz.join("assets"));
    Router::new()
        .route("/{examen}", get(|Path(examen): Path<String>| async move { Redirect::permanent(&format!("/{examen}/")) }))
        .route("/{examen}/", get(pagina))
        .route("/{examen}/api/inicio", get(inicio))
        .nest_service("/assets", assets)
        .with_state(Arc::new(Carpetas { raiz, interfaz }))
}

/// Un examen es una carpeta de la raíz con `examen.json`, y se lee en cada petición.
async fn leer_examen(carpetas: &Carpetas, examen: &str) -> Result<Examen, Response> {
    if examen.starts_with('.') {
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

async fn inicio(State(carpetas): Estado, Path(examen): Path<String>) -> Result<Response, Response> {
    let datos = leer_examen(&carpetas, &examen).await?;
    let banco: Vec<Pregunta> = match tokio::fs::read(carpetas.raiz.join(&examen).join("preguntas.json")).await {
        Ok(texto) => serde_json::from_slice(&texto).map_err(error_interno)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(error_interno(e)),
    };
    Ok(axum::Json(nucleo::inicio(&datos, &banco)).into_response())
}

fn error_interno(e: impl std::fmt::Display) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
}
