//! `migrar-ronda [raíz]`: `guardada` de cada `ronda.json` de la raíz pasa de milisegundos a ISO 8601 con `Z`.
//! Corre una sola vez, con el servidor parado.

fn main() {
    let raiz = std::env::args().nth(1).unwrap_or_else(|| "..".into());
    for entrada in std::fs::read_dir(&raiz).expect("la raíz no se lee") {
        let ruta = entrada.unwrap().path().join("ronda.json");
        let Ok(texto) = std::fs::read_to_string(&ruta) else { continue };
        let mut g: serde_json::Value = serde_json::from_str(&texto).expect("ronda.json no es JSON");
        let Some(ms) = g["guardada"].as_i64() else {
            println!("{} ya está migrado", ruta.display());
            continue;
        };
        g["guardada"] = jiff::Timestamp::from_millisecond(ms).unwrap().to_string().into();
        let g: nucleo::Guardado = serde_json::from_value(g).expect("ronda.json no encaja con Guardado");
        std::fs::write(&ruta, serde_json::to_vec(&g).unwrap()).unwrap();
        println!("{}: {}", ruta.display(), g.guardada);
    }
}
