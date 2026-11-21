//! `migrar <respuestas.jsonl>…`: cambia `acierto` por `resultado` en cada historial, con una copia en `respuestas.jsonl.antes`.
//! Un `acierto: true` pasa a `Acierto`, y un `acierto: false` pasa a `Fallo`. Una línea con `resultado` queda igual.

use serde_json::{Value, json};

fn main() {
    for ruta in std::env::args().skip(1) {
        let antes = format!("{ruta}.antes");
        // La copia de una migración anterior es el único original que queda.
        assert!(!std::path::Path::new(&antes).exists(), "{antes} ya existe");
        let texto = std::fs::read_to_string(&ruta).unwrap();
        let mut nuevo = String::new();
        for linea in texto.lines().filter(|l| !l.trim().is_empty()) {
            let mut r: Value = serde_json::from_str(linea).unwrap();
            if let Some(acierto) = r.as_object_mut().unwrap().remove("acierto") {
                r["resultado"] = json!(if acierto.as_bool().unwrap() { "Acierto" } else { "Fallo" });
            }
            let r: nucleo::Respuesta = serde_json::from_value(r).unwrap();
            nuevo += &serde_json::to_string(&r).unwrap();
            nuevo += "\n";
        }
        std::fs::copy(&ruta, &antes).unwrap();
        std::fs::write(&ruta, nuevo).unwrap();
        println!("{ruta}: {} respuestas", texto.lines().filter(|l| !l.trim().is_empty()).count());
    }
}
