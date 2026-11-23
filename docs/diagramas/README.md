# Los diagramas de la app

Los diagramas de `app/` siguen el modelo C4: el contexto, los contenedores y los componentes, de menos a más zoom. Cada diagrama responde a una pregunta. Las clases y los campos viven en el código, y no van aquí.

| Archivo | La pregunta |
|---|---|
| `1-contexto.puml` | ¿Quién usa la app? |
| `2-contenedores.puml` | ¿Qué corre dónde? |
| `3-interfaz.puml` | ¿Qué pantalla hace qué? |
| `3-nucleo.puml` | ¿Dónde vive cada regla? |
| `3-servidor-y-banco.puml` | ¿Quién toca cada archivo de un examen? |
| `ronda.puml` | ¿Cómo pasa la ronda de una pantalla a otra? |
| `abrir-un-examen.puml` | ¿Cómo acaban el dispositivo y el servidor con el mismo historial y la misma ronda? |
| `contestar-una-pregunta.puml` | ¿Qué pasa con una respuesta sin red? |

Un cambio de un contenedor, un componente o una flecha actualiza su diagrama en el mismo cambio.

En VS Code, la extensión `jebbs.plantuml` abre la vista previa con Alt+D. Cada diagrama lleva `!pragma layout smetana`, que dibuja sin Graphviz. Las macros de C4 vienen dentro de PlantUML, sin red.
