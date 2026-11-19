import json
import tempfile
import threading
import unittest
import urllib.error
import urllib.request
from pathlib import Path

from serve import Manejador, crear_servidor

Manejador.log_message = lambda *args: None


class TestServe(unittest.TestCase):
    def setUp(self):
        self.carpeta = Path(tempfile.mkdtemp())
        (self.carpeta / "preguntas.json").write_text('[{"id": "real"}]')
        (self.carpeta / "preguntas-ejemplo.json").write_text('[{"id": "ejemplo"}]')
        (self.carpeta / "2A _Teoría.pdf").write_bytes(b"%PDF-falso")
        (self.carpeta / "paginas").mkdir()
        (self.carpeta / "paginas" / "2A _Teoría-5.png").write_bytes(b"png")
        (self.carpeta / "index.html").write_text("<title>LN</title>")
        self.historial = self.carpeta / "respuestas.jsonl"
        self.arrancar("preguntas.json")

    def arrancar(self, banco):
        self.servidor = crear_servidor(0, banco, self.historial, self.carpeta)
        threading.Thread(target=self.servidor.serve_forever, daemon=True).start()
        self.url = f"http://127.0.0.1:{self.servidor.server_address[1]}"

    def tearDown(self):
        self.servidor.shutdown()
        self.servidor.server_close()

    def get(self, ruta):
        with urllib.request.urlopen(self.url + ruta) as r:
            return r.read()

    def post(self, cuerpo):
        datos = cuerpo if isinstance(cuerpo, bytes) else json.dumps(cuerpo).encode()
        peticion = urllib.request.Request(self.url + "/api/respuestas", data=datos, method="POST")
        with urllib.request.urlopen(peticion) as r:
            return json.loads(r.read())

    def test_escucha_solo_en_localhost(self):
        self.assertEqual(self.servidor.server_address[0], "127.0.0.1")

    def test_sirve_la_interfaz_el_banco_los_pdfs_y_las_paginas(self):
        self.assertIn(b"<title>LN</title>", self.get("/"))
        self.assertEqual(json.loads(self.get("/preguntas.json")), [{"id": "real"}])
        self.assertEqual(self.get("/2A%20_Teor%C3%ADa.pdf"), b"%PDF-falso")
        self.assertEqual(self.get("/paginas/2A%20_Teor%C3%ADa-5.png"), b"png")

    def test_sirve_otro_banco_en_la_ruta_del_banco(self):
        self.tearDown()
        self.arrancar("preguntas-ejemplo.json")
        self.assertEqual(json.loads(self.get("/preguntas.json")), [{"id": "ejemplo"}])

    def test_el_historial_empieza_vacio(self):
        self.assertEqual(json.loads(self.get("/api/respuestas")), [])

    def test_post_anade_respuestas_y_get_devuelve_el_historial_entero(self):
        a = {"id": "rgpd-001", "ts": "2026-11-06T18:02:11Z", "acierto": True, "seguro": False}
        b = {"id": "rgpd-002", "ts": "2026-11-06T18:03:00Z", "acierto": False, "seguro": True}
        self.post([a])
        self.post([b])
        self.assertEqual(json.loads(self.get("/api/respuestas")), [a, b])
        lineas = self.historial.read_text().splitlines()
        self.assertEqual([json.loads(l) for l in lineas], [a, b])

    def test_una_respuesta_enviada_2_veces_entra_una_vez(self):
        a = {"id": "rgpd-001", "ts": "2026-11-06T18:02:11Z", "acierto": True, "seguro": False}
        self.post([a])
        self.assertEqual(self.post([a]), {"anadidas": 0})
        self.assertEqual(json.loads(self.get("/api/respuestas")), [a])

    def test_un_blanco_lleva_la_clave_blanco(self):
        b = {"id": "rgpd-001", "ts": "2026-11-06T18:02:11Z", "acierto": False, "seguro": False, "blanco": True}
        self.assertEqual(self.post([b]), {"anadidas": 1})
        self.assertEqual(json.loads(self.get("/api/respuestas")), [b])

    def test_rechaza_una_respuesta_sin_el_formato(self):
        blanco = {"id": "x", "ts": "t", "acierto": False, "seguro": False}
        malas = [b"no es json", {"id": "x"}, [{"id": "x", "ts": "t", "acierto": "si", "seguro": True}], [{"id": "x"}],
                 [{**blanco, "blanco": False}], [{**blanco, "acierto": True, "blanco": True}], [{**blanco, "otra": 1}]]
        for mala in malas:
            with self.assertRaises(urllib.error.HTTPError) as error:
                self.post(mala)
            self.assertEqual(error.exception.code, 400)
        self.assertFalse(self.historial.exists())

    def test_una_nota_se_guarda_se_sustituye_y_se_borra_con_el_texto_vacio(self):
        def poner(cuerpo):
            peticion = urllib.request.Request(self.url + "/api/notas", data=json.dumps(cuerpo).encode(), method="POST")
            urllib.request.urlopen(peticion).close()

        self.assertEqual(json.loads(self.get("/api/notas")), {})
        poner({"id": "a", "texto": "uno"})
        poner({"id": "b", "texto": "dos"})
        poner({"id": "a", "texto": "tres"})
        self.assertEqual(json.loads(self.get("/api/notas")), {"a": "tres", "b": "dos"})
        poner({"id": "a", "texto": "  "})
        self.assertEqual(json.loads(self.get("/api/notas")), {"b": "dos"})
        with self.assertRaises(urllib.error.HTTPError) as e:
            poner({"id": "a", "texto": 1})
        self.assertEqual(e.exception.code, 400)

    def test_la_ronda_empieza_sin_guardado_y_se_guarda_y_se_cierra(self):
        def poner(cuerpo):
            peticion = urllib.request.Request(self.url + "/api/ronda", data=json.dumps(cuerpo).encode(), method="POST")
            urllib.request.urlopen(peticion).close()

        self.assertEqual(json.loads(self.get("/api/ronda")), {"guardada": 0, "ronda": None})
        media = {"guardada": 5, "ronda": {"ids": ["a", "b"], "resultados": ["acierto"], "simulacro": True, "limite": 99}}
        poner(media)
        self.assertEqual(json.loads(self.get("/api/ronda")), media)
        poner({"guardada": 6, "ronda": None})
        self.assertEqual(json.loads(self.get("/api/ronda")), {"guardada": 6, "ronda": None})

    def test_rechaza_un_guardado_de_la_ronda_sin_el_formato(self):
        ronda = {"ids": ["a"], "resultados": [], "simulacro": False, "limite": None}
        malos = [
            {"guardada": 1},
            {"guardada": "1", "ronda": None},
            {"guardada": True, "ronda": None},
            {"guardada": 1, "ronda": None, "otro": 1},
            {"guardada": 1, "ronda": {**ronda, "ids": [1]}},
            {"guardada": 1, "ronda": {**ronda, "resultados": ["quizá"]}},
            {"guardada": 1, "ronda": {**ronda, "resultados": [[1]]}},
            {"guardada": 1, "ronda": {**ronda, "simulacro": 1}},
            {"guardada": 1, "ronda": {**ronda, "limite": "x"}},
            {"guardada": 1, "ronda": {"ids": ["a"]}},
            [],
        ]
        for malo in malos:
            peticion = urllib.request.Request(self.url + "/api/ronda", data=json.dumps(malo).encode(), method="POST")
            with self.assertRaises(urllib.error.HTTPError) as e:
                urllib.request.urlopen(peticion)
            self.assertEqual(e.exception.code, 400, malo)
        self.assertFalse((self.carpeta / "ronda.json").exists())


if __name__ == "__main__":
    unittest.main()
