# El diagrama de la app

El diagrama de clases de `app/`: los tipos de `nucleo`, el servidor y la interfaz, y cómo se llaman. Un cambio de un tipo, una ruta o una dependencia entre las 3 partes actualiza este diagrama en el mismo commit.

```mermaid
classDiagram
  direction LR

  namespace nucleo {
    class Examen {
      +titulo: String
    }
    class Pregunta {
      +id: String
    }
    class Inicio {
      <<ts-rs export>>
      +titulo: String
      +banco_vacio: bool
    }
    class funciones_nucleo["nucleo"] {
      +inicio(examen: &Examen, banco: &[Pregunta]) Inicio
    }
  }

  namespace servidor {
    class Carpetas {
      -raiz: PathBuf
      -interfaz: PathBuf
    }
    class rutas["servidor (axum)"] {
      +app(raiz, interfaz) Router
      GET /:examen → 308 /:examen/
      GET /:examen/ → index.html
      GET /:examen/api/inicio → Inicio
      GET /assets/* → dist/assets
      -leer_examen(carpetas, examen) Examen
    }
  }

  namespace interfaz {
    class App["App.svelte"] {
      fetch("api/inicio")
      muestra titulo
      muestra "El banco no tiene preguntas"
    }
    class InicioTs["tipos/Inicio.ts"] {
      <<generado>>
      titulo: string
      banco_vacio: boolean
    }
  }

  funciones_nucleo ..> Examen : lee
  funciones_nucleo ..> Pregunta : lee
  funciones_nucleo --> Inicio : devuelve
  rutas *-- Carpetas : estado
  rutas ..> funciones_nucleo : llama
  Inicio ..> InicioTs : ts-rs genera
  App ..> InicioTs : importa el tipo
  App ..> rutas : HTTP
```
