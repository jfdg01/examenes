import collections
import functools
import json
import re
import subprocess
import sys
from pathlib import Path

TEMAS = {"rgpd", "iso27001", "ens", "riesgos", "auditoria", "lssi", "software",
         "propiedad", "delitos", "etica", "certificaciones"}


def comprobar(preguntas, carpeta):
    errores = []
    vistos = set()
    for p in preguntas:
        id_ = p.get("id")
        if id_ in vistos:
            errores.append(f"{id_}: id repetido")
        vistos.add(id_)
        if p.get("tema") not in TEMAS:
            errores.append(f"{id_}: tema fuera de la lista: {p.get('tema')}")
        distractores = p.get("distractores", [])
        if len(distractores) != 3:
            errores.append(f"{id_}: {len(distractores)} distractores, y no 3")
        if p.get("correcta") in distractores:
            errores.append(f"{id_}: un distractor es la respuesta correcta: {p.get('correcta')}")
        fuente = p.get("fuente", {})
        pdf = Path(carpeta) / fuente.get("pdf", "")
        if not fuente.get("pdf") or not pdf.is_file():
            errores.append(f"{id_}: el pdf no existe en ln/: {fuente.get('pdf')}")
        elif fuente.get("pagina", 0) > paginas(pdf):
            errores.append(f"{id_}: la página {fuente.get('pagina')} pasa de las {paginas(pdf)} de {pdf.name}")
    for concepto, n in collections.Counter(p.get("concepto") for p in preguntas).items():
        if n not in (2, 3):
            errores.append(f"{concepto}: {n} pregunta{'s' if n != 1 else ''}, y no 2 o 3")
    return errores


@functools.cache
def paginas(pdf):
    info = subprocess.run(["pdfinfo", str(pdf)], capture_output=True, text=True, check=True).stdout
    return int(re.search(r"^Pages:\s+(\d+)", info, re.M).group(1))


if __name__ == "__main__":
    ln = Path(__file__).parent
    banco = Path(sys.argv[1]) if len(sys.argv) > 1 else ln / "preguntas.json"
    errores = comprobar(json.loads(banco.read_text()), ln)
    print("\n".join(errores) or "El banco no tiene errores.")
    sys.exit(1 if errores else 0)
