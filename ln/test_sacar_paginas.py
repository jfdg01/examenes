import tempfile
import unittest
from pathlib import Path

from sacar_paginas import sacar


def pdf_de_paginas(n):
    objetos = ["<< /Type /Catalog /Pages 2 0 R >>",
               f"<< /Type /Pages /Kids [{' '.join(f'{3 + k} 0 R' for k in range(n))}] /Count {n} >>"]
    objetos += ["<< /Type /Page /Parent 2 0 R /MediaBox [0 0 72 72] >>"] * n
    datos, posiciones = b"%PDF-1.4\n", []
    for k, objeto in enumerate(objetos, 1):
        posiciones.append(len(datos))
        datos += f"{k} 0 obj\n{objeto}\nendobj\n".encode()
    xref = len(datos)
    datos += f"xref\n0 {len(objetos) + 1}\n0000000000 65535 f \n".encode()
    datos += "".join(f"{p:010d} 00000 n \n" for p in posiciones).encode()
    datos += f"trailer\n<< /Size {len(objetos) + 1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode()
    return datos


class TestSacarPaginas(unittest.TestCase):
    def setUp(self):
        self.carpeta = Path(tempfile.mkdtemp())
        (self.carpeta / "2A _Teoría.pdf").write_bytes(pdf_de_paginas(2))
        (self.carpeta / "3 _Teoría.pdf").write_bytes(pdf_de_paginas(1))

    def test_saca_una_imagen_por_pagina_de_cada_pdf(self):
        sacar(self.carpeta)
        imagenes = sorted(p.name for p in (self.carpeta / "paginas").iterdir())
        self.assertEqual(imagenes, ["2A _Teoría-1.png", "2A _Teoría-2.png", "3 _Teoría-1.png"])
        self.assertTrue((self.carpeta / "paginas" / "3 _Teoría-1.png").read_bytes().startswith(b"\x89PNG"))

    def test_no_repite_una_imagen_que_ya_existe(self):
        (self.carpeta / "paginas").mkdir()
        (self.carpeta / "paginas" / "2A _Teoría-1.png").write_bytes(b"ya existe")
        sacar(self.carpeta)
        self.assertEqual((self.carpeta / "paginas" / "2A _Teoría-1.png").read_bytes(), b"ya existe")
        self.assertTrue((self.carpeta / "paginas" / "2A _Teoría-2.png").exists())


if __name__ == "__main__":
    unittest.main()
