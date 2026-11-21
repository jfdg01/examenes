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
async fn el_inicio_da_el_titulo_y_las_preguntas_de_la_sesion() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;

    let inicio = inicio(&url, "ln").await;
    assert_eq!(
        (&inicio["estado"], &inicio["titulo"], &inicio["sesion"], &inicio["simulacro"]),
        (&json!("Listo"), &json!("LN"), &json!(6), &json!(6))
    );
}

#[tokio::test]
async fn la_tarjeta_del_inicio_cuenta_el_historial_con_el_objetivo_de_la_peticion() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;
    let respuesta = json!({ "id": "rgpd-001", "ts": "2026-11-04T12:41:22Z", "acierto": true, "seguro": true });
    assert_eq!(responder(&url, "ln", &json!([respuesta])).await, 204);

    let tarjeta = |objetivo: &'static str| {
        let url = url.clone();
        async move {
            let inicio: Value = reqwest::get(format!("{url}/ln/api/inicio{objetivo}")).await.unwrap().json().await.unwrap();
            inicio["tarjeta"].clone()
        }
    };
    let (cinco, diez) = (tarjeta("").await, tarjeta("?objetivo=10").await);
    assert_eq!((&cinco["sabes"], &cinco["sin_ver"]), (&json!(1), &json!(5)));
    assert!(diez["hoy"].as_u64() < cinco["hoy"].as_u64(), "{diez} {cinco}");
    for mal in ["mucho", "NaN", "11", "-1"] {
        let respuesta = reqwest::get(format!("{url}/ln/api/inicio?objetivo={mal}")).await.unwrap();
        assert_eq!(respuesta.status(), 400, "{mal}");
    }
}

#[tokio::test]
async fn un_examen_sin_preguntas_tiene_el_banco_vacio() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "asn", "ASN", None);
    crear_examen(raiz.path(), "ps", "PS", Some("[]"));
    let url = arrancar(raiz.path()).await;

    assert_eq!(inicio(&url, "asn").await, json!({ "estado": "BancoVacio", "titulo": "ASN" }));
    assert_eq!(inicio(&url, "ps").await["estado"], "BancoVacio");
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

async fn sesion(url: &str, carpeta: &str) -> Vec<Value> {
    reqwest::get(format!("{url}/{carpeta}/api/sesion")).await.unwrap().error_for_status().unwrap().json().await.unwrap()
}

async fn responder(url: &str, carpeta: &str, cuerpo: &Value) -> reqwest::StatusCode {
    reqwest::Client::new().post(format!("{url}/{carpeta}/api/respuestas")).json(cuerpo).send().await.unwrap().status()
}

#[tokio::test]
async fn la_sesion_envia_solo_lo_que_la_pantalla_muestra() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;

    let sesion = sesion(&url, "ln").await;
    assert_eq!(sesion.len(), 6);
    let mut campos: Vec<&String> = sesion[0].as_object().unwrap().keys().collect();
    campos.sort();
    let esperados = [
        "concepto",
        "correcta",
        "explicacion",
        "id",
        "opciones",
        "paso",
        "paso_tras_otra",
        "paso_tras_seguro",
        "pregunta",
        "tema",
    ];
    assert_eq!(campos, esperados);
    assert_eq!(sesion[0]["opciones"].as_array().unwrap().len(), 4);
    assert_eq!(sesion[0]["paso"], "sin ver");
}

#[tokio::test]
async fn una_respuesta_entra_en_el_historial_una_sola_vez_y_mueve_el_paso() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;
    let respuesta = json!({ "id": "rgpd-001", "ts": "2026-11-04T12:41:22Z", "acierto": true, "seguro": true });

    assert_eq!(responder(&url, "ln", &json!([respuesta])).await, 204);
    assert_eq!(responder(&url, "ln", &json!([respuesta])).await, 204);

    let historial = fs::read_to_string(raiz.path().join("ln/respuestas.jsonl")).unwrap();
    assert_eq!(historial, "{\"id\":\"rgpd-001\",\"ts\":\"2026-11-04T12:41:22Z\",\"acierto\":true,\"seguro\":true}\n");
    let pregunta = sesion(&url, "ln").await.into_iter().find(|q| q["id"] == "rgpd-001").unwrap();
    assert_eq!(pregunta["paso"], "paso 1");
}

#[tokio::test]
async fn una_respuesta_mal_formada_no_entra_en_el_historial() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;

    for mala in [
        json!([{ "id": "rgpd-001", "ts": "ayer", "acierto": true, "seguro": true }]),
        json!([{ "id": "rgpd-001", "ts": "2026-11-04T12:41:22Z", "acierto": "sí", "seguro": true }]),
        json!([{ "id": "rgpd-001", "ts": "2026-11-04T12:41:22Z", "acierto": true, "seguro": true, "extra": 1 }]),
        json!({ "id": "rgpd-001" }),
    ] {
        assert!(responder(&url, "ln", &mala).await.is_client_error(), "{mala}");
    }
    assert!(!raiz.path().join("ln/respuestas.jsonl").exists());
}

#[tokio::test]
async fn el_simulacro_da_su_reloj_y_las_preguntas_del_banco() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;

    let simulacro: Value = reqwest::get(format!("{url}/ln/api/simulacro")).await.unwrap().json().await.unwrap();
    assert_eq!(simulacro["segundos"], 25 * 60);
    assert_eq!(simulacro["preguntas"].as_array().unwrap().len(), 6);
}

async fn cerrar(url: &str, cuerpo: &Value) -> reqwest::Response {
    reqwest::Client::new().post(format!("{url}/ln/api/cierre")).json(cuerpo).send().await.unwrap()
}

#[tokio::test]
async fn la_nota_del_simulacro_lleva_la_penalizacion_del_examen() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;

    let cierre = json!({ "preguntas": 4, "resultados": ["Acierto", "Acierto", "Fallo"] });
    let nota: Value = cerrar(&url, &cierre).await.json().await.unwrap();
    assert_eq!(nota, json!({ "aciertos": 2, "fallos": 1, "en_blanco": 1, "nota": "4,38" }));
    let de_mas = json!({ "preguntas": 1, "resultados": ["Acierto", "Fallo"] });
    assert_eq!(cerrar(&url, &de_mas).await.status(), 400);
}
