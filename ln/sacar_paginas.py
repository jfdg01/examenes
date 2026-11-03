import re
import subprocess
import sys
from pathlib import Path


def sacar(carpeta):
    destino = carpeta / "paginas"
    destino.mkdir(exist_ok=True)
    for pdf in sorted(carpeta.glob("*.pdf")):
        info = subprocess.run(["pdfinfo", pdf], capture_output=True, text=True, check=True).stdout
        for pagina in range(1, int(re.search(r"^Pages:\s+(\d+)", info, re.M).group(1)) + 1):
            nombre = destino / f"{pdf.stem}-{pagina}"
            if not Path(f"{nombre}.png").exists():
                subprocess.run(["pdftoppm", "-png", "-r", "80", "-f", str(pagina), "-l", str(pagina),
                                "-singlefile", pdf, nombre], check=True)


if __name__ == "__main__":
    sacar(Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent)
