import collections
import functools
import json
import re
import subprocess
import sys
from pathlib import Path


def comprobar(preguntas, carpeta):
    examen = json.loads((Path(carpeta) / "examen.json").read_text(encoding="utf-8"))
    distractores_por_pregunta = examen["opciones"] - 1
    errores = []
    vistos = set()
    for p in preguntas:
        id_ = p.get("id")
        if id_ in vistos:
            errores.append(f"{id_}: id repetido")
        vistos.add(id_)
        if p.get("tema") not in examen["temas"]:
            errores.append(f"{id_}: tema fuera de la lista: {p.get('tema')}")
        distractores = p.get("distractores", [])
        if len(distractores) != distractores_por_pregunta:
            errores.append(f"{id_}: {len(distractores)} distractores, y no {distractores_por_pregunta}")
        if p.get("correcta") in distractores:
            errores.append(f"{id_}: un distractor es la respuesta correcta: {p.get('correcta')}")
        fuente = p.get("fuente", {})
        pdf = Path(carpeta) / fuente.get("pdf", "")
        if not fuente.get("pdf") or not pdf.is_file():
            errores.append(f"{id_}: el pdf no existe en la carpeta: {fuente.get('pdf')}")
        elif fuente.get("pagina", 0) > paginas(pdf):
            errores.append(f"{id_}: la página {fuente.get('pagina')} pasa de las {paginas(pdf)} de {pdf.name}")
    for concepto, n in collections.Counter(p.get("concepto") for p in preguntas).items():
        if n not in (2, 3):
            errores.append(f"{concepto}: {n} pregunta{'s' if n != 1 else ''}, y no 2 o 3")
    return errores


def huerfanas(preguntas, historial):
    if not Path(historial).is_file():
        return []
    ids = {p.get("id") for p in preguntas}
    lineas = Path(historial).read_text(encoding="utf-8").splitlines()
    return sorted({json.loads(l)["id"] for l in lineas if l.strip()} - ids)


@functools.cache
def paginas(pdf):
    info = subprocess.run(["pdfinfo", str(pdf)], capture_output=True, text=True, check=True).stdout
    return int(re.search(r"^Pages:\s+(\d+)", info, re.M).group(1))


if __name__ == "__main__":
    banco = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).parent / "preguntas.json"
    carpeta = banco.resolve().parent
    preguntas = json.loads(banco.read_text())
    errores = comprobar(preguntas, carpeta)
    print("\n".join(errores) or "El banco no tiene errores.")
    sin_banco = huerfanas(preguntas, carpeta / "respuestas.jsonl")
    if sin_banco:
        print(f"Aviso: {len(sin_banco)} id del historial no están en el banco: {', '.join(sin_banco)}")
    sys.exit(1 if errores else 0)
