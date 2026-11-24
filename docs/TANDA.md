# Una tanda de temas

Una tanda escribe varios temas de una vez, o rehace un examen entero. Un agente principal la dirige, con un agente creador y un agente revisor por tema, todos en paralelo. Solo el agente principal escribe en el banco del examen. Las reglas del banco, del creador y del revisor están en [`BANCO.md`](BANCO.md).

## 1. La preparación

1. El agente principal pregunta al dueño qué pasa con el historial de los temas de la tanda. `banco importar` vuelve a empezar en el primer `id` libre, así que una respuesta vieja de `respuestas.jsonl` señala una pregunta nueva. Las opciones son borrar las respuestas de esos temas, o guardarlas fuera del repo. `ronda.json` se borra, porque una ronda a medias usa los `id` viejos.
2. El agente quita del banco las preguntas de los temas de la tanda, y quita de `sin_pregunta` sus PDFs.
3. El agente compila `banco` una vez, para el binario de los creadores.
4. El agente crea una carpeta de trabajo por tema. `trabajo/` está en `.gitignore`.

```bash
cargo build --manifest-path app/Cargo.toml -p banco
for t in introduccion claves; do
  d=ps/trabajo/$t; mkdir -p $d; cp ps/examen.json $d/; echo '[]' > $d/preguntas.json; ln -s ../../pdfs $d/pdfs
done
```

## 2. La creación

El agente principal lanza un creador por tema con esta orden. `<examen>`, `<tema>`, `<pdf>` y `<N>` cambian en cada orden. El agente principal añade lo que el `README.md` del examen marca para ese PDF.

````
Escribe desde cero las preguntas del tema `<tema>` del examen `<examen>`. El PDF es `<pdf>`, de <N> páginas. Escribe en español.

Lee primero `docs/BANCO.md` entero y `<examen>/README.md`. Eres el creador de «El flujo de trabajo», y tu carpeta de trabajo es `<examen>/trabajo/<tema>/`. Cada punto marcado del README que toca tu PDF tiene su concepto.

```bash
B=app/target/debug/banco
$B importar <examen>/trabajo/<tema> <tema> <examen>/trabajo/<tema>/tanda.md
$B comprobar <examen>/trabajo/<tema>
```
````

El agente principal guarda las dudas de cada creador para su revisor.
