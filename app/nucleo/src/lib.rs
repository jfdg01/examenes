//! La lógica de la app. `nucleo` no lee archivos ni la red.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Las cifras de un examen, de `examen.json`.
#[derive(Deserialize)]
pub struct Examen {
    pub titulo: String,
}

/// Una pregunta del banco, de `preguntas.json`.
#[derive(Deserialize)]
pub struct Pregunta {
    pub id: String,
}

/// Lo que muestra la pantalla del inicio.
#[derive(Serialize, TS)]
#[ts(export)]
pub struct Inicio {
    pub titulo: String,
    pub banco_vacio: bool,
}

pub fn inicio(examen: &Examen, banco: &[Pregunta]) -> Inicio {
    Inicio {
        titulo: examen.titulo.clone(),
        banco_vacio: banco.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn examen() -> Examen {
        Examen { titulo: "LN".into() }
    }

    #[test]
    fn el_inicio_da_el_titulo_del_examen() {
        let banco = [Pregunta { id: "rgpd-001".into() }];
        let inicio = inicio(&examen(), &banco);
        assert_eq!(inicio.titulo, "LN");
        assert!(!inicio.banco_vacio);
    }

    #[test]
    fn un_banco_sin_preguntas_es_un_banco_vacio() {
        assert!(inicio(&examen(), &[]).banco_vacio);
    }
}
