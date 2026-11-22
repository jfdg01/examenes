use std::path::Path;
use std::process::ExitCode;

const USO: &str = "Uso: banco comprobar <carpeta> | banco paginas <carpeta>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [orden, carpeta] = args.as_slice() else {
        eprintln!("{USO}");
        return ExitCode::FAILURE;
    };
    let carpeta = Path::new(carpeta);
    let resultado = match orden.as_str() {
        "comprobar" => banco::informe(carpeta),
        "paginas" => banco::sacar(carpeta).map(|()| true),
        _ => Err(USO.to_string()),
    };
    match resultado {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
