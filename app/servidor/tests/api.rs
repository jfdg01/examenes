//! Cada test crea una raíz temporal de exámenes, arranca el servidor y le habla por HTTP.

use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use tempfile::TempDir;

const BANCO_LN: &str = include_str!("datos/ln-preguntas.json");

fn crear_examen(raiz: &Path, carpeta: &str, titulo: &str, banco: Option<&str>) {
    let dir = raiz.join(carpeta);
    fs::create_dir(&dir).unwrap();
    fs::write(
        dir.join("examen.json"),
        json!({ "titulo": titulo, "preguntas": 20, "opciones": 4, "minutos": 25, "penalizacion": 0.25 }).to_string(),
    )
    .unwrap();
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
    reqwest::get(format!("{url}/{carpeta}/api/inicio")).await.unwrap().error_for_status().unwrap().json().await.unwrap()
}

#[tokio::test]
async fn el_inicio_da_las_cifras_el_banco_y_el_historial() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;

    let inicio = inicio(&url, "ln").await;
    assert_eq!((&inicio["examen"]["titulo"], &inicio["examen"]["penalizacion"]), (&json!("LN"), &json!(0.25)));
    assert_eq!(inicio["banco"].as_array().unwrap().len(), 6);
    assert_eq!(inicio["historial"], json!([]));
}

#[tokio::test]
async fn el_inicio_va_con_gzip() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;

    let respuesta =
        reqwest::Client::new().get(format!("{url}/ln/api/inicio")).header("accept-encoding", "gzip").send().await.unwrap();
    assert_eq!(respuesta.headers()["content-encoding"], "gzip");
}

#[tokio::test]
async fn un_examen_sin_preguntas_tiene_el_banco_vacio() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "asn", "ASN", None);
    crear_examen(raiz.path(), "ps", "PS", Some("[]"));
    let url = arrancar(raiz.path()).await;

    assert_eq!(inicio(&url, "asn").await["banco"], json!([]));
    assert_eq!(inicio(&url, "ps").await["banco"], json!([]));
}

#[tokio::test]
async fn un_examen_nuevo_aparece_al_crear_su_carpeta() {
    let raiz = TempDir::new().unwrap();
    let url = arrancar(raiz.path()).await;
    let ruta = format!("{url}/nuevo/api/inicio");
    assert_eq!(reqwest::get(&ruta).await.unwrap().status(), 404);

    crear_examen(raiz.path(), "nuevo", "Nuevo", None);

    assert_eq!(inicio(&url, "nuevo").await["examen"]["titulo"], "Nuevo");
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

async fn responder(url: &str, carpeta: &str, cuerpo: &Value) -> reqwest::StatusCode {
    reqwest::Client::new().post(format!("{url}/{carpeta}/api/respuestas")).json(cuerpo).send().await.unwrap().status()
}

#[tokio::test]
async fn una_respuesta_entra_en_el_historial_una_sola_vez() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;
    let respuesta = json!({ "id": "rgpd-001", "ts": "2026-11-04T12:41:22Z", "resultado": "Blanco", "seguro": false });

    assert_eq!(responder(&url, "ln", &json!([respuesta])).await, 204);
    assert_eq!(responder(&url, "ln", &json!([respuesta])).await, 204);

    let historial = fs::read_to_string(raiz.path().join("ln/respuestas.jsonl")).unwrap();
    assert_eq!(historial, "{\"id\":\"rgpd-001\",\"ts\":\"2026-11-04T12:41:22Z\",\"resultado\":\"Blanco\",\"seguro\":false}\n");
    assert_eq!(inicio(&url, "ln").await["historial"], json!([respuesta]));
}

#[tokio::test]
async fn una_respuesta_mal_formada_no_entra_en_el_historial() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;

    for mala in [
        json!([{ "id": "rgpd-001", "ts": "ayer", "resultado": "Acierto", "seguro": true }]),
        json!([{ "id": "rgpd-001", "ts": "2026-11-04T12:41:22Z", "resultado": "Quizá", "seguro": true }]),
        json!([{ "id": "rgpd-001", "ts": "2026-11-04T12:41:22Z", "acierto": true, "seguro": true }]),
        json!([{ "id": "rgpd-001", "ts": "2026-11-04T12:41:22Z", "resultado": "Acierto", "seguro": true, "extra": 1 }]),
        json!({ "id": "rgpd-001" }),
    ] {
        assert!(responder(&url, "ln", &mala).await.is_client_error(), "{mala}");
    }
    assert!(!raiz.path().join("ln/respuestas.jsonl").exists());
}
