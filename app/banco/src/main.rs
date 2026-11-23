use std::path::Path;
use std::process::ExitCode;

const USO: &str = "Uso: banco comprobar <carpeta> | banco paginas <carpeta> | banco revision <carpeta> <informe>... \
                   | banco importar <carpeta> <tema> <borrador>";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [orden, carpeta, informes @ ..] = args.as_slice() else {
        eprintln!("{USO}");
        return ExitCode::FAILURE;
    };
    let carpeta = Path::new(carpeta);
    let resultado = match orden.as_str() {
        "comprobar" if informes.is_empty() => banco::informe(carpeta),
        "paginas" if informes.is_empty() => banco::sacar(carpeta).map(|()| true),
        "revision" if !informes.is_empty() => banco::revision(carpeta, informes).map(|()| true),
        "importar" => match informes {
            [tema, borrador] => banco::importar(carpeta, tema, Path::new(borrador)).map(|ids| {
                println!("{} preguntas añadidas: {}", ids.len(), ids.join(", "));
                true
            }),
            _ => Err(USO.to_string()),
        },
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
