# Un lote de apartados

Un lote escribe varios apartados de una vez, o rehace un examen entero. Un agente principal lo dirige, con un agente creador y un agente revisor por apartado, todos en paralelo. Solo el agente principal escribe en el banco del examen. Las reglas del banco, del creador y del revisor están en [`BANCO.md`](BANCO.md).

## 1. La preparación

1. El agente principal pregunta al dueño qué pasa con el historial de los apartados del lote. `banco importar` vuelve a empezar en el primer `id` libre, así que una respuesta vieja de `respuestas.jsonl` señala una pregunta nueva. Las opciones son borrar las respuestas de esos temas, o guardarlas fuera del repo. `ronda.json` se borra, porque una ronda a medias usa los `id` viejos.
2. El agente compila `banco` una vez, para el binario de los creadores, y corre `banco preparar`. El comando quita del banco las preguntas de los apartados y su `sin_pregunta`, y crea `<examen>/trabajo/<apartado>/` para cada tema. Falla sin cambiar nada mientras quedan respuestas de esos temas o `ronda.json`. `trabajo/` está en `.gitignore`.

```bash
cargo build --manifest-path app/Cargo.toml -p banco
app/target/debug/banco preparar examenes/ps introduccion claves
```

## 2. La creación

El agente principal lanza un creador por apartado con esta orden. `<examen>`, `<apartado>`, `<pdf>` y `<N>` cambian en cada orden. El agente principal añade lo que el `README.md` del examen marca para ese PDF.

````
Escribe desde cero las preguntas del apartado `<apartado>` del examen `<examen>`. El PDF es `<pdf>`, de <N> páginas. Escribe en español.

Lee primero `docs/BANCO.md` entero y `<examen>/README.md`. Eres el creador de «El flujo de trabajo», y tu carpeta de trabajo es `<examen>/trabajo/<apartado>/`. Cada punto marcado del README que toca tu PDF tiene su concepto.

```bash
B=app/target/debug/banco
$B importar <examen>/trabajo/<apartado> <apartado> <examen>/trabajo/<apartado>/tanda.md
$B comprobar <examen>/trabajo/<apartado>
```
````

El agente principal guarda las dudas de cada creador para su revisor.

## 3. El montaje

Cuando acaban todos los creadores, el agente principal une sus apartados al banco del examen con `banco montar`. El comando falla sin cambiar nada con una pregunta de otro apartado o con un `id` repetido.

```bash
app/target/debug/banco montar ps introduccion claves
```

Los `id` no chocan, porque cada uno empieza por su apartado. Después, el agente principal hace esto:

1. Corre `banco comprobar <examen>`. Una sigla de dos apartados da un concepto con 2 preguntas. Su pregunta queda en el apartado cuya página explica la sigla, y la otra se borra con su concepto.
2. Comprueba que cada punto marcado del `README.md` tiene su concepto. Un punto que ningún creador cubrió entra con un borrador y `banco importar`.
3. Corre los tests, cambia la cifra de preguntas del `README.md` del examen y hace el commit. El commit separa el banco de los creadores de los arreglos de la revisión.

## 4. La revisión

El agente principal lanza un revisor por apartado con esta orden. Cada revisor recibe las dudas del creador de su apartado.

````
Haz la revisión completa del apartado `<apartado>` del examen `<examen>`. Tú no escribiste estas preguntas. Escribe en español.

Lee primero `docs/BANCO.md`, sobre todo «Las reglas», «Las fuentes» y «La revisión completa», y `<examen>/README.md`. Eres el revisor de «La revisión completa». Tu PDF es `<pdf>`.

Lee solo las preguntas con `"apartado": "<apartado>"` de `<examen>/preguntas.json`, con un filtro de python3: el banco entero satura tu contexto.

Las dudas del creador: <dudas>.
````

## 5. Los arreglos

Cuando acaban todos los revisores, el agente principal aplica los informes con `banco revision` y arregla a mano cada fallo de `A mano:`. El lote acaba cuando cada fallo de `A mano:` tiene su arreglo, `banco comprobar` no da errores ni avisos, y los tests pasan. Entonces el agente hace el commit y borra `<examen>/trabajo/`.
