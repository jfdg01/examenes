#!/bin/sh
# Corre banco comprobar tras cada edición de un banco. Un fallo devuelve el informe a Claude, que arregla el banco en el turno siguiente.
# El aviso no deshace la edición: un estado intermedio de un lote puede avisar, y el pre-commit sigue como el control final.
ruta=$(jq -r '.tool_input.file_path // empty')
ruta=${ruta#"$CLAUDE_PROJECT_DIR"/}
examen=$(printf '%s\n' "$ruta" | sed -nE 's#^(examenes/[^/]+)/(preguntas|conceptos|desarrollo|examen)\.json$#\1#p')
[ -n "$examen" ] || exit 0
cd "$CLAUDE_PROJECT_DIR" || exit 0
informe=$(cargo run -q --manifest-path app/Cargo.toml -p banco -- comprobar "$examen" 2>&1) && exit 0
printf 'banco comprobar %s falla:\n%s\n' "$examen" "$informe" >&2
exit 2
