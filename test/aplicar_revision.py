import json
import sys
from pathlib import Path

CAMPOS = ("pregunta", "correcta", "distractores", "explicacion", "fuente")


def aplicar(banco, fallos):
    """Aplica cada arreglo al banco, y da los fallos que un agente arregla a mano."""
    por_id = {p["id"]: p for p in banco}
    pendientes = []
    for f in fallos:
        arreglo = f.get("arreglo") or {}
        raros = set(arreglo) - set(CAMPOS)
        if f.get("id") not in por_id or not arreglo or raros:
            pendientes.append(f)
            continue
        por_id[f["id"]].update(arreglo)
    return pendientes


if __name__ == "__main__":
    banco_json = Path(sys.argv[1])
    banco = json.loads(banco_json.read_text(encoding="utf-8"))
    fallos = [f for informe in sys.argv[2:] for f in json.loads(Path(informe).read_text(encoding="utf-8"))]
    pendientes = aplicar(banco, fallos)
    banco_json.write_text(json.dumps(banco, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"{len(fallos) - len(pendientes)} arreglos aplicados.")
    for f in pendientes:
        print(f"A mano: {f.get('id')} ({f.get('gravedad')}): {f.get('fallo')}")
