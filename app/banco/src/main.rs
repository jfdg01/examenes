use std::path::Path;
use std::process::ExitCode;

use banco::Tipo;

const USO: &str = "Uso: banco comprobar <carpeta> | banco paginas <carpeta> \
                   | banco importar <carpeta> <tipo> <apartado> <borrador> | banco preparar <carpeta> <tipo> <apartado>... \
                   | banco montar <carpeta> <tipo> <apartado>... | banco revision <carpeta> <tipo> <informe>... \
                   (<tipo> es conceptos o test)";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [orden, carpeta, resto @ ..] = args.as_slice() else {
        eprintln!("{USO}");
        return ExitCode::FAILURE;
    };
    let carpeta = Path::new(carpeta);
    let resultado = match (orden.as_str(), resto) {
        ("comprobar", []) => banco::informe(carpeta),
        ("paginas", []) => banco::sacar(carpeta).map(|()| true),
        (orden, [tipo, resto @ ..]) if !resto.is_empty() => tipo.parse::<Tipo>().and_then(|tipo| match (orden, resto) {
            ("revision", informes) => banco::revision(carpeta, tipo, informes).map(|()| true),
            ("preparar", apartados) => banco::preparar(carpeta, tipo, apartados).map(|()| true),
            ("montar", apartados) => banco::montar(carpeta, tipo, apartados).map(|n| {
                println!("{n} añadidos.");
                true
            }),
            ("importar", [apartado, borrador]) => banco::importar(carpeta, tipo, apartado, Path::new(borrador)).map(|ids| {
                println!("{} añadidos: {}", ids.len(), ids.join(", "));
                true
            }),
            _ => Err(USO.to_string()),
        }),
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
