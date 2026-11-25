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
        json!({ "titulo": titulo, "preguntas": 20, "opciones": 4, "minutos": 25, "penalizacion": 0.25, "pdfs": {}, "apartados": {} }).to_string(),
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

#[tokio::test]
async fn un_campo_de_mas_en_el_examen_o_en_el_banco_da_un_error_que_lo_nombra() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;
    let error = || async { reqwest::get(format!("{url}/ln/api/inicio")).await.unwrap().text().await.unwrap() };

    let examen = raiz.path().join("ln/examen.json");
    let mut e: Value = serde_json::from_str(&fs::read_to_string(&examen).unwrap()).unwrap();
    e["puerto"] = json!(8601);
    fs::write(&examen, e.to_string()).unwrap();
    assert!(error().await.contains("unknown field `puerto`"));

    crear_examen(raiz.path(), "ps", "PS", None);
    let mut banco: Value = serde_json::from_str(BANCO_LN).unwrap();
    banco[0]["dificultad"] = json!(3);
    fs::write(raiz.path().join("ps/preguntas.json"), banco.to_string()).unwrap();
    let texto = reqwest::get(format!("{url}/ps/api/inicio")).await.unwrap().text().await.unwrap();
    assert!(texto.contains("unknown field `dificultad`"), "{texto}");
}

async fn ronda(url: &str) -> Value {
    reqwest::get(format!("{url}/ln/api/ronda")).await.unwrap().error_for_status().unwrap().json().await.unwrap()
}

async fn guardar_ronda(url: &str, cuerpo: &Value) -> reqwest::StatusCode {
    reqwest::Client::new().put(format!("{url}/ln/api/ronda")).json(cuerpo).send().await.unwrap().status()
}

#[tokio::test]
async fn la_ronda_guardada_vuelve_igual() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;
    assert_eq!(ronda(&url).await, json!({ "guardada": "1970-01-01T00:00:00Z", "ronda": null }));

    let g = json!({ "guardada": "2026-11-21T10:51:16.012Z", "ronda": {
        "ids": ["rgpd-001", "rgpd-002"], "resultados": ["Blanco"], "respuestas": [{ "id": "rgpd-001", "ts": "2026-11-21T10:50:00Z", "resultado": "Blanco", "seguro": false }], "simulacro": true, "limite": 1_795_185_376_012_u64 },
        "seleccion": [{ "pdf": "1_Legislacion.pdf", "apartado": "rgpd" }] });
    assert_eq!(guardar_ronda(&url, &g).await, 204);
    assert_eq!(ronda(&url).await, g);
    assert_eq!(serde_json::from_str::<Value>(&fs::read_to_string(raiz.path().join("ln/ronda.json")).unwrap()).unwrap(), g);
}

#[tokio::test]
async fn una_ronda_mal_formada_no_se_guarda() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;

    for mala in [
        json!({ "guardada": 1_795_183_876_012_u64, "ronda": null }),
        json!({ "guardada": "ayer", "ronda": null }),
        json!({ "guardada": "2026-11-21T10:51:16Z", "ronda": null, "extra": 1 }),
        json!({ "guardada": "2026-11-21T10:51:16Z", "ronda": { "ids": [], "resultados": ["acierto"], "respuestas": [], "simulacro": false, "limite": null } }),
    ] {
        assert!(guardar_ronda(&url, &mala).await.is_client_error(), "{mala}");
    }
    assert!(!raiz.path().join("ln/ronda.json").exists());
}

#[tokio::test]
async fn un_guardado_mas_viejo_no_pisa_la_ronda() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;
    let nuevo = json!({ "guardada": "2026-11-21T10:51:17Z", "ronda": null });

    assert_eq!(guardar_ronda(&url, &nuevo).await, 204);
    let viejo = json!({ "guardada": "2026-11-21T10:51:16Z", "ronda": { "ids": ["rgpd-001"], "resultados": [], "respuestas": [], "simulacro": false, "limite": null } });
    assert_eq!(guardar_ronda(&url, &viejo).await, 204);
    assert_eq!(ronda(&url).await, nuevo);
}

#[tokio::test]
async fn el_examen_sirve_las_imagenes_de_las_paginas_y_los_pdfs() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let ln = raiz.path().join("ln");
    fs::create_dir(ln.join("paginas")).unwrap();
    fs::create_dir(ln.join("pdfs")).unwrap();
    fs::write(ln.join("paginas/2A _Teoría-7.webp"), "webp").unwrap();
    fs::write(ln.join("pdfs/2A _Teoría.pdf"), "pdf").unwrap();
    let url = arrancar(raiz.path()).await;

    for (ruta, tipo, cuerpo) in
        [("paginas/2A _Teoría-7.webp", "image/webp", "webp"), ("pdfs/2A _Teoría.pdf#page=7", "application/pdf", "pdf")]
    {
        let r = reqwest::get(format!("{url}/ln/{ruta}")).await.unwrap().error_for_status().unwrap();
        assert_eq!(r.headers()["content-type"], tipo);
        assert_eq!(r.text().await.unwrap(), cuerpo);
    }
    for ruta in ["paginas/no-esta.webp", "paginas/..%2Fexamen.json", "otra/examen.json"] {
        assert_eq!(reqwest::get(format!("{url}/ln/{ruta}")).await.unwrap().status(), 404, "{ruta}");
    }
}

async fn notas(url: &str) -> Value {
    reqwest::get(format!("{url}/ln/api/notas")).await.unwrap().error_for_status().unwrap().json().await.unwrap()
}

async fn anotar(url: &str, cuerpo: &Value) -> reqwest::StatusCode {
    reqwest::Client::new().post(format!("{url}/ln/api/notas")).json(cuerpo).send().await.unwrap().status()
}

#[tokio::test]
async fn una_nota_se_guarda_en_notas_json_y_vuelve() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;
    assert_eq!(notas(&url).await, json!({}));

    assert_eq!(anotar(&url, &json!({ "id": "rgpd-001", "texto": "El art. 6" })).await, 204);
    assert_eq!(anotar(&url, &json!({ "id": "rgpd-002", "texto": "Otra" })).await, 204);
    assert_eq!(anotar(&url, &json!({ "id": "rgpd-001", "texto": "El art. 6.1" })).await, 204);
    let esperadas = json!({ "rgpd-001": "El art. 6.1", "rgpd-002": "Otra" });
    assert_eq!(notas(&url).await, esperadas);
    let archivo: Value = serde_json::from_str(&fs::read_to_string(raiz.path().join("ln/notas.json")).unwrap()).unwrap();
    assert_eq!(archivo, esperadas);
}

#[tokio::test]
async fn una_nota_en_blanco_se_borra() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;

    assert_eq!(anotar(&url, &json!({ "id": "rgpd-001", "texto": "El art. 6" })).await, 204);
    assert_eq!(anotar(&url, &json!({ "id": "rgpd-001", "texto": " \n" })).await, 204);
    assert_eq!(notas(&url).await, json!({}));
}

#[tokio::test]
async fn una_nota_mal_formada_no_se_guarda() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;

    for mala in [
        json!({ "id": "rgpd-001" }),
        json!({ "id": "rgpd-001", "texto": 1 }),
        json!({ "id": "rgpd-001", "texto": "x", "extra": 1 }),
        json!([{ "id": "rgpd-001", "texto": "x" }]),
    ] {
        assert!(anotar(&url, &mala).await.is_client_error(), "{mala}");
    }
    assert!(!raiz.path().join("ln/notas.json").exists());
}

#[tokio::test]
async fn la_lista_da_cada_examen_por_su_titulo() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ps", "PS", None);
    crear_examen(raiz.path(), "ln", "LN", None);
    fs::create_dir(raiz.path().join("docs")).unwrap();
    let url = arrancar(raiz.path()).await;
    let lista = || async { reqwest::get(format!("{url}/api/examenes")).await.unwrap().json::<Value>().await.unwrap() };

    assert_eq!(lista().await, json!([{ "carpeta": "ln", "titulo": "LN" }, { "carpeta": "ps", "titulo": "PS" }]));
    crear_examen(raiz.path(), "asn", "ASN", None);
    assert_eq!(lista().await[0], json!({ "carpeta": "asn", "titulo": "ASN" }));
}

#[tokio::test]
async fn una_raiz_sin_examenes_sirve_la_interfaz_y_la_lista_vacia() {
    let raiz = TempDir::new().unwrap();
    let url = arrancar(raiz.path()).await;

    assert_eq!(reqwest::get(format!("{url}/api/examenes")).await.unwrap().json::<Value>().await.unwrap(), json!([]));
    assert_eq!(reqwest::get(format!("{url}/")).await.unwrap().text().await.unwrap(), "<div id=\"app\"></div>");
}

#[tokio::test]
async fn la_lista_va_en_orden_alfabetico_sin_tildes_ni_mayusculas() {
    let raiz = TempDir::new().unwrap();
    for (carpeta, titulo) in [("z", "Zeta"), ("b", "bases"), ("o", "Óptica"), ("a", "Álgebra"), ("n", "Nubes")] {
        crear_examen(raiz.path(), carpeta, titulo, None);
    }
    let url = arrancar(raiz.path()).await;

    let lista: Value = reqwest::get(format!("{url}/api/examenes")).await.unwrap().json().await.unwrap();
    let titulos: Vec<_> = lista.as_array().unwrap().iter().map(|e| e["titulo"].as_str().unwrap()).collect();
    assert_eq!(titulos, ["Álgebra", "bases", "Nubes", "Óptica", "Zeta"]);
}

async fn editar(url: &str, cuerpo: &Value) -> reqwest::StatusCode {
    reqwest::Client::new().put(format!("{url}/ln/api/preguntas")).json(cuerpo).send().await.unwrap().status()
}

#[tokio::test]
async fn una_pregunta_editada_entra_en_preguntas_json_con_su_id() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;

    let mut q = inicio(&url, "ln").await["banco"][2].clone();
    let (pregunta, distractor) = (q["pregunta"].clone(), q["distractores"][1].clone());
    q["pregunta"] = json!("¿Un enunciado editado?");
    q["distractores"][1] = json!("Un distractor editado");
    assert_eq!(editar(&url, &q).await, 204);
    let banco = inicio(&url, "ln").await["banco"].clone();
    assert_eq!(banco[2], q);
    assert_eq!(banco.as_array().unwrap().len(), 6);
    // El archivo cambia solo en los textos editados, con el formato de `banco revision`.
    let esperado = BANCO_LN.replacen(&pregunta.to_string(), "\"¿Un enunciado editado?\"", 1).replacen(
        &distractor.to_string(),
        "\"Un distractor editado\"",
        1,
    );
    assert_eq!(fs::read_to_string(raiz.path().join("ln/preguntas.json")).unwrap(), esperado);
}

#[tokio::test]
async fn una_edicion_mal_formada_no_toca_preguntas_json() {
    let raiz = TempDir::new().unwrap();
    crear_examen(raiz.path(), "ln", "LN", Some(BANCO_LN));
    let url = arrancar(raiz.path()).await;

    let q = inicio(&url, "ln").await["banco"][0].clone();
    let con = |campo: &str, valor: Value| {
        let mut m = q.clone();
        m[campo] = valor;
        m
    };
    for (mala, estado) in [
        (con("id", json!("no-esta")), 404),
        (con("pregunta", json!("  ")), 422),
        (con("explicacion", json!("")), 422),
        (con("distractores", json!(["uno"])), 422),
        (con("concepto", json!("otro")), 422),
        (con("extra", json!(1)), 422),
    ] {
        assert_eq!(editar(&url, &mala).await, estado, "{mala}");
    }
    assert_eq!(fs::read_to_string(raiz.path().join("ln/preguntas.json")).unwrap(), BANCO_LN);
}
