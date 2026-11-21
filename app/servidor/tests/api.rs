//! Cada test crea una raíz temporal de exámenes, arranca el servidor y le habla por HTTP.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use tempfile::TempDir;

const BANCO_LN: &str = include_str!("datos/ln-preguntas.json");

fn crear_examen(raiz: &Path, carpeta: &str, titulo: &str, banco: Option<&str>) {
    let dir = raiz.join(carpeta);
    fs::create_dir(&dir).unwrap();
    fs::write(dir.join("examen.json"), json!({ "titulo": titulo }).to_string()).unwrap();
    if let Some(banco) = banco {
        fs::write(dir.join("preguntas.json"), banco).unwrap();
    }
}

async fn arrancar(raiz: &Path) -> String {
    let interfaz = raiz.join(".interfaz");
    fs::create_dir(&interfaz).unwrap();
    fs::write(interfaz.join("index.html"), "<div id=\"app\"></div>").unwrap();
    let oyente = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", oyente.local_addr().unwrap());
    tokio::spawn(axum::serve(oyente, servidor::app(raiz.to_path_buf(), interfaz)).into_future());
    url
}

async fn inicio(url: &str, carpeta: &str) -> Value {
    reqwest::get(format!("{url}/{carpeta}/api/inicio"))
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test]
async fn el_inicio_da_el_titulo_del_examen() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;

    assert_eq!(inicio(&url, "ln").await, json!({ "titulo": "LN", "banco_vacio": false }));
}

#[tokio::test]
async fn un_examen_sin_preguntas_tiene_el_banco_vacio() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "asn", "ASN", None);
    crear_examen(raiz.path(), "ps", "PS", Some("[]"));
    let url = arrancar(raiz.path()).await;

    assert_eq!(inicio(&url, "asn").await["banco_vacio"], true);
    assert_eq!(inicio(&url, "ps").await["banco_vacio"], true);
}

#[tokio::test]
async fn un_examen_nuevo_aparece_al_crear_su_carpeta() {
    let raiz = TempDir::new().unwrap();
    let url = arrancar(raiz.path()).await;
    let ruta = format!("{url}/nuevo/api/inicio");
    assert_eq!(reqwest::get(&ruta).await.unwrap().status(), 404);

    crear_examen(raiz.path(), "nuevo", "Nuevo", None);

    assert_eq!(inicio(&url, "nuevo").await["titulo"], "Nuevo");
}

#[tokio::test]
async fn una_carpeta_sin_examen_json_no_es_un_examen() {
    let raiz = TempDir::new().unwrap();
    fs::create_dir(raiz.path().join("docs")).unwrap();
    let url = arrancar(raiz.path()).await;

    crear_examen(raiz.path(), "ln", "LN", None);
    for ruta in ["docs/api/inicio", "docs/", "../api/inicio", "docs%2F..%2Fln/api/inicio"] {
        assert_eq!(reqwest::get(format!("{url}/{ruta}")).await.unwrap().status(), 404, "{ruta}");
    }
}

#[tokio::test]
async fn la_ruta_del_examen_sirve_la_pagina() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", None);
    let url = arrancar(raiz.path()).await;

    let respuesta = reqwest::get(format!("{url}/ln")).await.unwrap();
    assert_eq!(respuesta.url().path(), "/ln/");
    assert_eq!(respuesta.status(), 200);
    assert!(respuesta.text().await.unwrap().contains("<div id=\"app\">"));
}
