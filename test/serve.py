import argparse
import json
import threading
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit

AQUI = Path(__file__).resolve().parent
CAMPOS = {"id": str, "ts": str, "acierto": bool, "seguro": bool}
RESULTADOS = {"acierto", "fallo", "blanco"}
SIN_RONDA = {"guardada": 0, "ronda": None}
cerrojo = threading.Lock()


def es_int(x):
    return type(x) is int


def valida_ronda(g):
    if not (isinstance(g, dict) and g.keys() == {"guardada", "ronda"} and es_int(g["guardada"])):
        return False
    r = g["ronda"]
    if r is None:
        return True
    return (
        isinstance(r, dict)
        and r.keys() == {"ids", "resultados", "simulacro", "limite"}
        and isinstance(r["ids"], list) and all(type(i) is str for i in r["ids"])
        and isinstance(r["resultados"], list) and all(type(x) is str and x in RESULTADOS for x in r["resultados"])
        and type(r["simulacro"]) is bool
        and (r["limite"] is None or es_int(r["limite"]))
    )


def valida(respuestas):
    return isinstance(respuestas, list) and all(
        isinstance(r, dict) and r.keys() == CAMPOS.keys() and all(type(r[k]) is t for k, t in CAMPOS.items())
        for r in respuestas
    )


def leer(historial):
    if not historial.exists():
        return []
    return [json.loads(l) for l in historial.read_text(encoding="utf-8").splitlines() if l.strip()]


class Manejador(SimpleHTTPRequestHandler):
    def __init__(self, *args, banco, historial, notas, ronda, **kwargs):
        self.banco = banco
        self.historial = historial
        self.notas = notas
        self.ronda = ronda
        super().__init__(*args, **kwargs)

    def end_headers(self):
        self.send_header("Cache-Control", "no-cache")
        super().end_headers()

    def translate_path(self, path):
        if urlsplit(path).path == "/preguntas.json":
            return str(Path(self.directory) / self.banco)
        # La carpeta del examen tiene sus datos, y la interfaz sale de la carpeta del código.
        propia = Path(super().translate_path(path))
        return str(propia if propia.is_file() else AQUI / propia.relative_to(self.directory))

    def responder_json(self, codigo, datos):
        cuerpo = json.dumps(datos, ensure_ascii=False).encode()
        self.send_response(codigo)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(cuerpo)))
        self.end_headers()
        self.wfile.write(cuerpo)

    def do_GET(self):
        ruta = urlsplit(self.path).path
        if ruta == "/api/respuestas":
            with cerrojo:
                self.responder_json(200, leer(self.historial))
        elif ruta == "/api/notas":
            with cerrojo:
                self.responder_json(200, self.leer_notas())
        elif ruta == "/api/ronda":
            with cerrojo:
                self.responder_json(200, json.loads(self.ronda.read_text(encoding="utf-8")) if self.ronda.exists() else SIN_RONDA)
        else:
            super().do_GET()

    def leer_notas(self):
        return json.loads(self.notas.read_text(encoding="utf-8")) if self.notas.exists() else {}

    def guardar_nota(self, nota):
        if not (isinstance(nota, dict) and nota.keys() == {"id", "texto"} and all(type(v) is str for v in nota.values())):
            self.send_error(400, "Una nota lleva solo id y texto")
            return
        with cerrojo:
            notas = self.leer_notas()
            if nota["texto"].strip():
                notas[nota["id"]] = nota["texto"]
            else:
                notas.pop(nota["id"], None)
            self.notas.write_text(json.dumps(notas, ensure_ascii=False, indent=1), encoding="utf-8")
        self.responder_json(200, {})

    def do_POST(self):
        ruta = urlsplit(self.path).path
        if ruta not in ("/api/respuestas", "/api/notas", "/api/ronda"):
            self.send_error(404)
            return
        try:
            respuestas = json.loads(self.rfile.read(int(self.headers.get("Content-Length", 0))))
        except ValueError:
            respuestas = None
        if ruta == "/api/notas":
            self.guardar_nota(respuestas)
            return
        if ruta == "/api/ronda":
            if not valida_ronda(respuestas):
                self.send_error(400, "Un guardado de la ronda lleva solo guardada y ronda")
                return
            with cerrojo:
                self.ronda.write_text(json.dumps(respuestas, ensure_ascii=False), encoding="utf-8")
            self.responder_json(200, {})
            return
        if not valida(respuestas):
            self.send_error(400, "Una respuesta lleva solo id, ts, acierto y seguro")
            return
        with cerrojo:
            vistas = {(r["id"], r["ts"]) for r in leer(self.historial)}
            nuevas = []
            for r in respuestas:
                if (r["id"], r["ts"]) not in vistas:
                    vistas.add((r["id"], r["ts"]))
                    nuevas.append(r)
            if nuevas:
                with self.historial.open("a", encoding="utf-8") as f:
                    f.writelines(json.dumps(r, ensure_ascii=False) + "\n" for r in nuevas)
        self.responder_json(200, {"anadidas": len(nuevas)})


def crear_servidor(puerto, banco, historial, carpeta):
    manejador = partial(Manejador, banco=banco, historial=Path(carpeta) / historial, notas=Path(carpeta) / "notas.json", ronda=Path(carpeta) / "ronda.json", directory=str(carpeta))
    return ThreadingHTTPServer(("127.0.0.1", puerto), manejador)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="El servidor del test de un examen")
    parser.add_argument("carpeta", type=Path, help="la carpeta del examen, con examen.json")
    parser.add_argument("--banco", default="preguntas.json", help="el archivo que sirve en /preguntas.json")
    parser.add_argument("--historial", default="respuestas.jsonl")
    argumentos = parser.parse_args()
    carpeta = argumentos.carpeta.resolve()
    puerto = json.loads((carpeta / "examen.json").read_text(encoding="utf-8"))["puerto"]
    servidor = crear_servidor(puerto, argumentos.banco, argumentos.historial, carpeta)
    print(f"{carpeta.name} en http://127.0.0.1:{puerto}")
    servidor.serve_forever()
