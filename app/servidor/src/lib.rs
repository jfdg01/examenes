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
use nucleo::desarrollo::LineaDeDesarrollo;
use nucleo::{Datos, Examen, ExamenDeLaLista, Guardado, Nota, Pregunta, PreguntaDeDesarrollo, Respuesta};
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
        .route("/", get(la_interfaz))
        .route("/api/examenes", get(examenes))
        .route("/{examen}", get(|Path(examen): Path<String>| async move { Redirect::permanent(&format!("/{examen}/")) }))
        .route("/{examen}/", get(pagina))
        .route("/{examen}/api/inicio", get(inicio))
        .route("/{examen}/api/respuestas", axum::routing::post(responder))
        .route("/{examen}/api/desarrollo", axum::routing::post(desarrollar))
        .route("/{examen}/api/preguntas", axum::routing::put(editar))
        .route("/{examen}/api/notas", get(leer_notas).post(anotar))
        .route("/{examen}/api/ronda", get(leer_ronda).put(guardar_ronda))
        .route(
            "/{examen}/paginas/{archivo}",
            get(|e, Path((examen, archivo))| servir(e, examen, "paginas", archivo, "image/webp")),
        )
        .route(
            "/{examen}/pdfs/{archivo}",
            get(|e, Path((examen, archivo))| servir(e, examen, "pdfs", archivo, "application/pdf")),
        )
        .nest_service("/assets", assets)
        .with_state(Arc::new(Carpetas { raiz, interfaz, cerrojo: Default::default() }))
        .layer(CompressionLayer::new().gzip(true))
}

/// Un nombre de una sola carpeta o un solo archivo: sin `/`, sin `..` y sin los ocultos.
fn un_nombre(nombre: &str) -> bool {
    let mut partes = std::path::Path::new(nombre).components();
    let uno = matches!((partes.next(), partes.next()), (Some(std::path::Component::Normal(_)), None));
    uno && !nombre.starts_with('.')
}

/// Un archivo de `paginas/` o de `pdfs/` del examen.
async fn servir(
    State(carpetas): Estado,
    examen: String,
    carpeta: &str,
    archivo: String,
    tipo: &'static str,
) -> Result<Response, Response> {
    leer_examen(&carpetas, &examen).await?;
    if !un_nombre(&archivo) {
        return Err(StatusCode::NOT_FOUND.into_response());
    }
    let bytes = tokio::fs::read(carpetas.raiz.join(&examen).join(carpeta).join(archivo))
        .await
        .map_err(|_| StatusCode::NOT_FOUND.into_response())?;
    Ok(([(axum::http::header::CONTENT_TYPE, tipo)], bytes).into_response())
}

/// Un examen es una carpeta de la raíz con `examen.json`, y se lee en cada petición.
async fn leer_examen(carpetas: &Carpetas, examen: &str) -> Result<Examen, Response> {
    if !un_nombre(examen) {
        return Err(StatusCode::NOT_FOUND.into_response());
    }
    let texto = tokio::fs::read(carpetas.raiz.join(examen).join("examen.json"))
        .await
        .map_err(|_| StatusCode::NOT_FOUND.into_response())?;
    serde_json::from_slice(&texto).map_err(error_interno)
}

/// El texto sin tildes ni otros diacríticos, y en minúsculas: «Álgebra» va delante de «bases».
fn clave_alfabetica(texto: &str) -> String {
    use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};
    texto.nfd().filter(|c| !is_combining_mark(*c)).flat_map(char::to_lowercase).collect()
}

/// Cada carpeta de la raíz con `examen.json`, en orden alfabético por título.
async fn examenes(State(carpetas): Estado) -> Result<axum::Json<Vec<ExamenDeLaLista>>, Response> {
    let mut lista = Vec::new();
    let mut entradas = tokio::fs::read_dir(&carpetas.raiz).await.map_err(error_interno)?;
    while let Some(entrada) = entradas.next_entry().await.map_err(error_interno)? {
        let Ok(carpeta) = entrada.file_name().into_string() else { continue };
        match leer_examen(&carpetas, &carpeta).await {
            Ok(examen) => lista.push(ExamenDeLaLista { carpeta, titulo: examen.titulo }),
            Err(e) if e.status() == StatusCode::NOT_FOUND => {}
            Err(e) => return Err(e),
        }
    }
    lista.sort_by_cached_key(|e| (clave_alfabetica(&e.titulo), e.carpeta.clone()));
    Ok(axum::Json(lista))
}

/// `index.html` de la interfaz, que pinta la lista en `/` y el examen en `/<carpeta>/`.
async fn la_interfaz(State(carpetas): Estado) -> Result<Html<Vec<u8>>, Response> {
    let html = tokio::fs::read(carpetas.interfaz.join("index.html")).await.map_err(error_interno)?;
    Ok(Html(html))
}

async fn pagina(State(carpetas): Estado, Path(examen): Path<String>) -> Result<Html<Vec<u8>>, Response> {
    leer_examen(&carpetas, &examen).await?;
    la_interfaz(State(carpetas)).await
}

/// Un archivo que no existe es un archivo vacío.
async fn leer_si_existe(ruta: PathBuf) -> Result<Vec<u8>, Response> {
    match tokio::fs::read(ruta).await {
        Ok(texto) => Ok(texto),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(error_interno(e)),
    }
}

/// Una lista de JSON de la carpeta del examen. Sin el archivo, la lista está vacía.
async fn leer_lista<T: serde::de::DeserializeOwned>(
    carpetas: &Carpetas,
    examen: &str,
    archivo: &str,
) -> Result<Vec<T>, Response> {
    let texto = leer_si_existe(carpetas.raiz.join(examen).join(archivo)).await?;
    if texto.is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_slice(&texto).map_err(error_interno)
}

/// Escribe un archivo entero en un temporal de la misma carpeta, y lo renombra: un corte a mitad deja el archivo viejo.
async fn escribir(ruta: PathBuf, texto: impl AsRef<[u8]>) -> Result<(), Response> {
    use tokio::io::AsyncWriteExt;
    let nombre = ruta.file_name().ok_or_else(|| error_interno("una ruta sin archivo"))?.to_string_lossy();
    let temporal = ruta.with_file_name(format!(".{nombre}.tmp"));
    let mut archivo = tokio::fs::File::create(&temporal).await.map_err(error_interno)?;
    archivo.write_all(texto.as_ref()).await.map_err(error_interno)?;
    archivo.sync_all().await.map_err(error_interno)?;
    tokio::fs::rename(temporal, ruta).await.map_err(error_interno)
}

/// Cambia los textos de una pregunta del banco por su `id`: el enunciado, `correcta`, `distractores` y `explicacion`.
/// El archivo queda con el formato de `banco revision`.
async fn editar(
    State(carpetas): Estado,
    Path(examen): Path<String>,
    axum::Json(nueva): axum::Json<Pregunta>,
) -> Result<StatusCode, Response> {
    leer_examen(&carpetas, &examen).await?;
    let _cerrojo = carpetas.cerrojo.lock().await;
    let mut banco: Vec<Pregunta> = leer_lista(&carpetas, &examen, "preguntas.json").await?;
    let q = banco.iter_mut().find(|q| q.id == nueva.id).ok_or_else(|| StatusCode::NOT_FOUND.into_response())?;
    let en_blanco = [&nueva.pregunta, &nueva.correcta, &nueva.explicacion]
        .into_iter()
        .chain(&nueva.distractores)
        .any(|t| t.trim().is_empty());
    let solo_textos =
        (&q.concepto, &q.apartado, q.distractores.len()) == (&nueva.concepto, &nueva.apartado, nueva.distractores.len());
    if !solo_textos || en_blanco {
        return Err((StatusCode::UNPROCESSABLE_ENTITY, "solo cambian los textos, y ninguno va en blanco").into_response());
    }
    *q = nueva;
    let texto = serde_json::to_string_pretty(&banco).map_err(error_interno)? + "\n";
    escribir(carpetas.raiz.join(&examen).join("preguntas.json"), texto).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Un historial tiene una línea de JSON por línea, como `respuestas.jsonl`. Quien llama tiene el cerrojo.
async fn leer_historial<T: serde::de::DeserializeOwned>(
    carpetas: &Carpetas,
    examen: &str,
    archivo: &str,
) -> Result<Vec<T>, Response> {
    let texto = leer_si_existe(carpetas.raiz.join(examen).join(archivo)).await?;
    texto
        .split(|&b| b == b'\n')
        .filter(|linea| !linea.trim_ascii().is_empty())
        .map(|linea| serde_json::from_slice(linea).map_err(error_interno))
        .collect()
}

/// Las cifras, los conceptos, los dos bancos y el historial del examen. La interfaz calcula todo con `nucleo`.
async fn inicio(State(carpetas): Estado, Path(examen): Path<String>) -> Result<axum::Json<Datos>, Response> {
    let datos = leer_examen(&carpetas, &examen).await?;
    let _cerrojo = carpetas.cerrojo.lock().await;
    let conceptos = leer_lista(&carpetas, &examen, "conceptos.json").await?;
    let banco = leer_lista(&carpetas, &examen, "preguntas.json").await?;
    let desarrollo: Vec<PreguntaDeDesarrollo> = leer_lista(&carpetas, &examen, "desarrollo.json").await?;
    let historial = leer_historial(&carpetas, &examen, "respuestas.jsonl").await?;
    let historial_de_desarrollo = leer_historial(&carpetas, &examen, "desarrollo.jsonl").await?;
    Ok(axum::Json(Datos { examen: datos, banco, conceptos, desarrollo, historial, historial_de_desarrollo }))
}

/// Añade al historial cada respuesta nueva. Una respuesta con el mismo `id` y `ts` ya está, y no entra otra vez.
async fn responder(
    State(carpetas): Estado,
    Path(examen): Path<String>,
    axum::Json(respuestas): axum::Json<Vec<Respuesta>>,
) -> Result<StatusCode, Response> {
    leer_examen(&carpetas, &examen).await?;
    let _cerrojo = carpetas.cerrojo.lock().await;
    let viejas: Vec<Respuesta> = leer_historial(&carpetas, &examen, "respuestas.jsonl").await?;
    anadir(&carpetas, &examen, "respuestas.jsonl", &viejas, &respuestas, |r| (r.id.clone(), r.ts)).await
}

/// Añade a `desarrollo.jsonl` cada línea nueva. Una línea con la misma clave ya está, y no entra otra vez.
async fn desarrollar(
    State(carpetas): Estado,
    Path(examen): Path<String>,
    axum::Json(lineas): axum::Json<Vec<LineaDeDesarrollo>>,
) -> Result<StatusCode, Response> {
    leer_examen(&carpetas, &examen).await?;
    let _cerrojo = carpetas.cerrojo.lock().await;
    let viejas: Vec<LineaDeDesarrollo> = leer_historial(&carpetas, &examen, "desarrollo.jsonl").await?;
    anadir(&carpetas, &examen, "desarrollo.jsonl", &viejas, &lineas, |l| {
        let (id, ts, corregida) = l.clave();
        (id.to_owned(), ts, corregida)
    })
    .await
}

/// Añade al historial `archivo` las líneas de `nuevas` con una clave que no está en `viejas`. Quien llama tiene el cerrojo.
async fn anadir<T: serde::Serialize, K: std::hash::Hash + Eq>(
    carpetas: &Carpetas,
    examen: &str,
    archivo: &str,
    viejas: &[T],
    nuevas: &[T],
    clave: impl Fn(&T) -> K,
) -> Result<StatusCode, Response> {
    let mut vistas: std::collections::HashSet<K> = viejas.iter().map(&clave).collect();
    let mut texto = String::new();
    for l in nuevas {
        if vistas.insert(clave(l)) {
            texto += &serde_json::to_string(l).map_err(error_interno)?;
            texto += "\n";
        }
    }
    if !texto.is_empty() {
        use tokio::io::AsyncWriteExt;
        let ruta = carpetas.raiz.join(examen).join(archivo);
        let mut f = tokio::fs::OpenOptions::new().create(true).append(true).open(ruta).await.map_err(error_interno)?;
        f.write_all(texto.as_bytes()).await.map_err(error_interno)?;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Sin `ronda.json`, no hay ronda a medias.
async fn leer_guardado(ruta: PathBuf) -> Result<Guardado, Response> {
    let texto = leer_si_existe(ruta).await?;
    if texto.is_empty() {
        return Ok(Guardado { guardada: jiff::Timestamp::UNIX_EPOCH, ronda: None, seleccion: None });
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
    escribir(ruta, texto).await?;
    Ok(StatusCode::NO_CONTENT)
}

type Notas = std::collections::BTreeMap<String, String>;

/// Sin `notas.json`, no hay notas. Quien llama tiene el cerrojo.
async fn leer_archivo_de_notas(ruta: PathBuf) -> Result<Notas, Response> {
    let texto = leer_si_existe(ruta).await?;
    if texto.is_empty() {
        return Ok(Notas::new());
    }
    serde_json::from_slice(&texto).map_err(error_interno)
}

async fn leer_notas(State(carpetas): Estado, Path(examen): Path<String>) -> Result<axum::Json<Notas>, Response> {
    leer_examen(&carpetas, &examen).await?;
    let _cerrojo = carpetas.cerrojo.lock().await;
    leer_archivo_de_notas(carpetas.raiz.join(&examen).join("notas.json")).await.map(axum::Json)
}

/// Guarda la nota de una pregunta. Una nota en blanco se borra.
async fn anotar(
    State(carpetas): Estado,
    Path(examen): Path<String>,
    axum::Json(nota): axum::Json<Nota>,
) -> Result<StatusCode, Response> {
    leer_examen(&carpetas, &examen).await?;
    let ruta = carpetas.raiz.join(&examen).join("notas.json");
    let _cerrojo = carpetas.cerrojo.lock().await;
    let mut notas = leer_archivo_de_notas(ruta.clone()).await?;
    if nota.texto.trim().is_empty() {
        notas.remove(&nota.id);
    } else {
        notas.insert(nota.id, nota.texto);
    }
    let texto = serde_json::to_vec_pretty(&notas).map_err(error_interno)?;
    escribir(ruta, texto).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn error_interno(e: impl std::fmt::Display) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
}
