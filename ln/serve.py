import argparse
import json
import threading
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit

AQUI = Path(__file__).resolve().parent
PUERTO = 8601
CAMPOS = {"id": str, "ts": str, "acierto": bool, "seguro": bool}
cerrojo = threading.Lock()


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
    def __init__(self, *args, banco, historial, **kwargs):
        self.banco = banco
        self.historial = historial
        super().__init__(*args, **kwargs)

    def end_headers(self):
        self.send_header("Cache-Control", "no-cache")
        super().end_headers()

    def translate_path(self, path):
        if urlsplit(path).path == "/preguntas.json":
            return str(Path(self.directory) / self.banco)
        return super().translate_path(path)

    def responder_json(self, codigo, datos):
        cuerpo = json.dumps(datos, ensure_ascii=False).encode()
        self.send_response(codigo)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(cuerpo)))
        self.end_headers()
        self.wfile.write(cuerpo)

    def do_GET(self):
        if urlsplit(self.path).path == "/api/respuestas":
            with cerrojo:
                self.responder_json(200, leer(self.historial))
        else:
            super().do_GET()

    def do_POST(self):
        if urlsplit(self.path).path != "/api/respuestas":
            self.send_error(404)
            return
        try:
            respuestas = json.loads(self.rfile.read(int(self.headers.get("Content-Length", 0))))
        except ValueError:
            respuestas = None
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


def crear_servidor(puerto, banco, historial, carpeta=AQUI):
    manejador = partial(Manejador, banco=banco, historial=Path(carpeta) / historial, directory=str(carpeta))
    return ThreadingHTTPServer(("127.0.0.1", puerto), manejador)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="El servidor del test de LN")
    parser.add_argument("--banco", default="preguntas.json", help="el archivo que sirve en /preguntas.json")
    parser.add_argument("--historial", default="respuestas.jsonl")
    argumentos = parser.parse_args()
    servidor = crear_servidor(PUERTO, argumentos.banco, argumentos.historial)
    print(f"LN en http://127.0.0.1:{PUERTO}")
    servidor.serve_forever()
