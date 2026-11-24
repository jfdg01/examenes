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

## 3. El montaje

Cuando acaban todos los creadores, el agente principal une sus temas al banco del examen:

```bash
python3 - ps introduccion claves <<'E2'
import json, sys
ex, temas = sys.argv[1], sys.argv[2:]
leer = lambda r: json.load(open(r))
banco, examen = leer(f"{ex}/preguntas.json"), leer(f"{ex}/examen.json")
for t in temas:
    banco += leer(f"{ex}/trabajo/{t}/preguntas.json")
    copia = leer(f"{ex}/trabajo/{t}/examen.json").get("sin_pregunta", {})
    for pdf, ts in examen["pdfs"].items():
        if t in ts and pdf in copia:
            examen.setdefault("sin_pregunta", {})[pdf] = copia[pdf]
for nombre, datos in [("preguntas", banco), ("examen", examen)]:
    open(f"{ex}/{nombre}.json", "w").write(json.dumps(datos, ensure_ascii=False, indent=2) + "\n")
E2
```

Los `id` no chocan, porque cada uno empieza por su tema. Después, el agente principal hace esto:

1. Corre `banco comprobar <examen>`. Una sigla de dos temas da un concepto con 2 preguntas. Su pregunta queda en el tema cuya `fuente` explica la sigla, y la otra se borra.
2. Comprueba que cada punto marcado del `README.md` tiene su concepto. Un punto que ningún creador cubrió entra con un borrador y `banco importar`.
3. Corre los tests, cambia la cifra de preguntas del `README.md` del examen y hace el commit. El commit separa el banco de los creadores de los arreglos de la revisión.

## 4. La revisión

El agente principal lanza un revisor por tema con esta orden. Cada revisor recibe las dudas del creador de su tema.

````
Haz la revisión completa del tema `<tema>` del examen `<examen>`. Tú no escribiste estas preguntas. Escribe en español.

Lee primero `docs/BANCO.md`, sobre todo «Las reglas», «Las fuentes» y «La revisión completa», y `<examen>/README.md`. Eres el revisor de «La revisión completa». Tu PDF es `<pdf>`.

Lee solo las preguntas con `"tema": "<tema>"` de `<examen>/preguntas.json`, con un filtro de python3: el banco entero satura tu contexto.

Las dudas del creador: <dudas>.
````

## 5. Los arreglos

Cuando acaban todos los revisores, el agente principal aplica los informes con `banco revision` y arregla a mano cada fallo de `A mano:`. La tanda acaba cuando cada fallo de `A mano:` tiene su arreglo, `banco comprobar` no da errores ni avisos, y los tests pasan. Entonces el agente hace el commit y borra `<examen>/trabajo/`.
