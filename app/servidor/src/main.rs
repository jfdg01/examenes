//! `servidor [raíz] [dirección]`: sirve cada carpeta de la raíz con `examen.json` en `/<carpeta>/`.

use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let raiz = PathBuf::from(args.next().unwrap_or_else(|| "../examenes".into()));
    let direccion = args.next().unwrap_or_else(|| "127.0.0.1:8602".into());
    let interfaz = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../interfaz/dist"));

    let oyente = tokio::net::TcpListener::bind(&direccion).await.expect("la dirección está ocupada");
    println!("Sirvo {} en http://{direccion}/", raiz.display());
    axum::serve(oyente, servidor::app(raiz, interfaz)).await.unwrap();
}
