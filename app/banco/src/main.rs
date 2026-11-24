use std::path::Path;
use std::process::ExitCode;

const USO: &str = "Uso: banco comprobar <carpeta> | banco paginas <carpeta> | banco revision <carpeta> <informe>... \
                   | banco importar <carpeta> <tema> <borrador> | banco preparar <carpeta> <tema>... \
                   | banco montar <carpeta> <tema>...";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [orden, carpeta, resto @ ..] = args.as_slice() else {
        eprintln!("{USO}");
        return ExitCode::FAILURE;
    };
    let carpeta = Path::new(carpeta);
    let resultado = match orden.as_str() {
        "comprobar" if resto.is_empty() => banco::informe(carpeta),
        "paginas" if resto.is_empty() => banco::sacar(carpeta).map(|()| true),
        "revision" if !resto.is_empty() => banco::revision(carpeta, resto).map(|()| true),
        "preparar" if !resto.is_empty() => banco::preparar(carpeta, resto).map(|()| true),
        "montar" if !resto.is_empty() => banco::montar(carpeta, resto).map(|n| {
            println!("{n} preguntas añadidas.");
            true
        }),
        "importar" => match resto {
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
