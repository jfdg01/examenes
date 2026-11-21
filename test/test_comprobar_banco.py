import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from aplicar_revision import aplicar
from comprobar_banco import comprobar, huerfanas, pistas

AQUI = Path(__file__).parent
LN = AQUI.parent / "ln"


def pregunta(**cambios):
    p = {
        "id": "rgpd-001",
        "concepto": "rgpd-brecha-plazo",
        "tema": "rgpd",
        "fuente": {"pdf": "1_Legislacion.pdf", "pagina": 34},
        "pregunta": "¿En qué plazo notifica el responsable una brecha?",
        "correcta": "72 horas",
        "distractores": ["24 horas", "Un mes", "48 horas"],
        "explicacion": "Art. 33 del RGPD.",
    }
    p.update(cambios)
    return p


def con_pareja(*preguntas):
    """Las preguntas, con otra del mismo concepto para que el concepto tenga 2."""
    return [*preguntas, pregunta(id="rgpd-002", pregunta="¿Cuántas horas tiene el responsable para notificar una brecha?")]


class ComprobarBanco(unittest.TestCase):
    def test_una_pregunta_valida_no_da_errores(self):
        self.assertEqual(comprobar(con_pareja(pregunta()), LN), [])

    def test_un_id_repetido_falla(self):
        errores = comprobar(con_pareja(pregunta(), pregunta()), LN)
        self.assertEqual(len(errores), 1)
        self.assertIn("rgpd-001", errores[0])

    def test_un_tema_fuera_de_la_lista_falla(self):
        errores = comprobar(con_pareja(pregunta(tema="gdpr")), LN)
        self.assertEqual(len(errores), 1)
        self.assertIn("gdpr", errores[0])

    def test_un_numero_de_distractores_distinto_de_3_falla(self):
        for distractores in (["24 horas", "Un mes"], ["24 horas", "Un mes", "48 horas", "1 semana"]):
            with self.subTest(n=len(distractores)):
                errores = comprobar(con_pareja(pregunta(distractores=distractores)), LN)
                self.assertEqual(len(errores), 1)
                self.assertIn("distractores", errores[0])

    def test_un_tema_que_no_es_de_su_pdf_falla(self):
        errores = comprobar(con_pareja(pregunta(tema="ens")), LN)
        self.assertEqual(len(errores), 1)
        self.assertIn("ens", errores[0])

    def test_un_distractor_igual_a_la_correcta_falla(self):
        errores = comprobar(con_pareja(pregunta(distractores=["24 horas", "72 horas", "Un mes"])), LN)
        self.assertEqual(len(errores), 1)
        self.assertIn("72 horas", errores[0])

    def test_un_pdf_que_no_existe_en_ln_falla(self):
        errores = comprobar(con_pareja(pregunta(fuente={"pdf": "4_Teoria.pdf", "pagina": 1})), LN)
        self.assertEqual(len(errores), 1)
        self.assertIn("4_Teoria.pdf", errores[0])

    def test_una_pagina_mayor_que_las_del_pdf_falla(self):
        # 1_Legislacion.pdf tiene 82 páginas.
        self.assertEqual(comprobar(con_pareja(pregunta(fuente={"pdf": "1_Legislacion.pdf", "pagina": 82})), LN), [])
        errores = comprobar(con_pareja(pregunta(fuente={"pdf": "1_Legislacion.pdf", "pagina": 83})), LN)
        self.assertEqual(len(errores), 1)
        self.assertIn("83", errores[0])

    def test_un_concepto_con_1_o_con_4_preguntas_falla(self):
        tres = con_pareja(pregunta(), pregunta(id="rgpd-003"))
        self.assertEqual(comprobar(tres, LN), [])
        for preguntas in ([pregunta()], [*tres, pregunta(id="rgpd-004")]):
            with self.subTest(n=len(preguntas)):
                errores = comprobar(preguntas, LN)
                self.assertEqual(len(errores), 1)
                self.assertIn("rgpd-brecha-plazo", errores[0])

    def test_un_concepto_de_sigla_tiene_1_pregunta(self):
        sigla = pregunta(concepto="sigla-rgpd")
        self.assertEqual(comprobar([sigla], LN), [])
        errores = comprobar([sigla, pregunta(id="rgpd-002", concepto="sigla-rgpd")], LN)
        self.assertEqual(len(errores), 1)
        self.assertIn("sigla-rgpd", errores[0])

    def test_el_script_falla_con_un_banco_con_errores(self):
        with tempfile.TemporaryDirectory() as tmp:
            banco = Path(tmp) / "preguntas.json"
            (Path(tmp) / "examen.json").write_bytes((LN / "examen.json").read_bytes())
            (Path(tmp) / "pdfs").symlink_to(LN / "pdfs")
            for preguntas, codigo in ((con_pareja(pregunta()), 0), (con_pareja(pregunta(), pregunta()), 1)):
                banco.write_text(json.dumps(preguntas))
                r = subprocess.run([sys.executable, AQUI / "comprobar_banco.py", banco], capture_output=True)
                self.assertEqual(r.returncode, codigo)

    def test_un_absoluto_solo_en_un_distractor_es_una_pista(self):
        avisos = pistas([pregunta(distractores=["Siempre 24 h", "Un mes", "48 horas"])])
        self.assertEqual(len(avisos), 1)
        self.assertIn("siempre", avisos[0])
        for correcta, distractores in (("72 horas", ["Un solo día", "Un mes", "48 horas"]), ("Nunca 72 h", ["Nunca 24 horas", "Un mes", "48 horas"])):
            with self.subTest(distractores=distractores):
                self.assertEqual(pistas([pregunta(correcta=correcta, distractores=distractores)]), [])

    def test_la_correcta_más_larga_en_más_de_1_de_cada_3_es_una_pista(self):
        larga = pregunta(correcta="72 horas desde que lo sabe")
        sigla = pregunta(concepto="sigla-rgpd", correcta="72 horas desde que lo sabe")
        self.assertEqual(pistas([larga, pregunta(), pregunta(), sigla]), [])
        self.assertEqual(len(pistas([larga, larga, pregunta()])), 1)

    def test_la_revisión_aplica_los_arreglos_y_da_el_resto(self):
        banco = [pregunta()]
        fallos = [
            {"id": "rgpd-001", "regla": 3, "gravedad": "grave", "fallo": "x", "arreglo": {"correcta": "3 días"}},
            {"id": "rgpd-001", "regla": 5, "gravedad": "leve", "fallo": "y", "arreglo": None},
            {"id": "rgpd-001", "regla": 0, "gravedad": "leve", "fallo": "z", "arreglo": {"id": "rgpd-009"}},
            {"id": "viejo-1", "regla": 3, "gravedad": "grave", "fallo": "w", "arreglo": {"correcta": "1"}},
        ]
        self.assertEqual([f["fallo"] for f in aplicar(banco, fallos)], ["y", "z", "w"])
        self.assertEqual(banco[0]["correcta"], "3 días")
        self.assertEqual(banco[0]["id"], "rgpd-001")

    def test_un_id_del_historial_fuera_del_banco_es_huerfano(self):
        with tempfile.TemporaryDirectory() as tmp:
            historial = Path(tmp) / "respuestas.jsonl"
            lineas = [{"id": i, "ts": "2026-11-18T10:00:00Z", "acierto": True, "seguro": True} for i in ("rgpd-001", "viejo-1", "viejo-1")]
            historial.write_text("".join(json.dumps(r) + "\n" for r in lineas))
            self.assertEqual(huerfanas([pregunta()], historial), ["viejo-1"])
            self.assertEqual(huerfanas([pregunta()], Path(tmp) / "no-existe.jsonl"), [])


if __name__ == "__main__":
    unittest.main()
