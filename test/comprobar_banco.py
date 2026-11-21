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
        distractores = p.get("distractores", [])
        if len(distractores) != distractores_por_pregunta:
            errores.append(f"{id_}: {len(distractores)} distractores, y no {distractores_por_pregunta}")
        if p.get("correcta") in distractores:
            errores.append(f"{id_}: un distractor es la respuesta correcta: {p.get('correcta')}")
        fuente = p.get("fuente", {})
        pdf = Path(carpeta) / "pdfs" / fuente.get("pdf", "")
        if fuente.get("pdf") not in examen["pdfs"] or not pdf.is_file():
            errores.append(f"{id_}: el pdf no está en examen.json o en pdfs/: {fuente.get('pdf')}")
            continue
        if p.get("tema") not in examen["pdfs"][fuente["pdf"]]:
            errores.append(f"{id_}: el tema {p.get('tema')} no es un tema de {fuente['pdf']} en examen.json")
        if fuente.get("pagina", 0) > paginas(pdf):
            errores.append(f"{id_}: la página {fuente.get('pagina')} pasa de las {paginas(pdf)} de {pdf.name}")
    for concepto, n in collections.Counter(p.get("concepto") for p in preguntas).items():
        validos = (1,) if str(concepto).startswith("sigla-") else (2, 3)
        if n not in validos:
            esperado = " o ".join(map(str, validos))
            errores.append(f"{concepto}: {n} pregunta{'s' if n != 1 else ''}, y no {esperado}")
    return errores


def huerfanas(preguntas, historial):
    if not Path(historial).is_file():
        return []
    ids = {p.get("id") for p in preguntas}
    lineas = Path(historial).read_text(encoding="utf-8").splitlines()
    return sorted({json.loads(l)["id"] for l in lineas if l.strip()} - ids)


ABSOLUTO = re.compile(r"\b(siempre|nunca|s[oó]l[oa]s?)\b", re.I)


def absolutos(texto):
    return set(m.lower() for m in ABSOLUTO.findall(re.sub(r"\buna? s[oó]l[oa]\b", "", texto, flags=re.I)))


def pistas(preguntas):
    """Los avisos de las pistas de la regla 6, que un revisor juzga."""
    avisos = []
    for p in preguntas:
        sobran = set().union(*map(absolutos, p.get("distractores", []))) - absolutos(p.get("correcta", ""))
        if sobran:
            avisos.append(f"{p.get('id')}: un distractor dice {', '.join(sorted(sobran))}, y la correcta no")
    normales = [p for p in preguntas if not str(p.get("concepto")).startswith("sigla-") and p.get("distractores")]
    larga = sum(len(p["correcta"]) > max(map(len, p["distractores"])) for p in normales)
    if normales and larga * 3 > len(normales):
        avisos.append(f"la correcta es la opción más larga en {larga} de {len(normales)} preguntas, más de 1 de cada 3")
    return avisos


@functools.cache
def paginas(pdf):
    info = subprocess.run(["pdfinfo", str(pdf)], capture_output=True, text=True, check=True).stdout
    return int(re.search(r"^Pages:\s+(\d+)", info, re.M).group(1))


if __name__ == "__main__":
    banco = Path(sys.argv[1])
    carpeta = banco.resolve().parent
    preguntas = json.loads(banco.read_text())
    errores = comprobar(preguntas, carpeta)
    print("\n".join(errores) or "El banco no tiene errores.")
    for aviso in pistas(preguntas):
        print(f"Aviso: {aviso}")
    sin_banco = huerfanas(preguntas, carpeta / "respuestas.jsonl")
    if sin_banco:
        print(f"Aviso: {len(sin_banco)} id del historial no están en el banco: {', '.join(sin_banco)}")
    sys.exit(1 if errores else 0)
