<script lang="ts">
  import Pomodoro from "./Pomodoro.svelte";
  import type { Datos } from "./nucleo/nucleo.js";

  // ponytail: el plan de la semana de ASN y PS va en el código. Tras el examen del lunes 30, la pantalla se borra.
  type Bloque = { examen: "asn" | "ps"; titulo: string; temas: string; sesiones: number; simulacro?: boolean };
  type Dia = { fecha: string; nombre: string; examen?: string; bloques: Bloque[]; nota?: string };
  const plan: Dia[] = [
    { fecha: "2026-11-24", nombre: "Martes 24", bloques: [{ examen: "asn", titulo: "ASN · tarde", temas: "Introducción · + Ciclo de vida", sesiones: 16 }] },
    { fecha: "2026-11-25", nombre: "Miércoles 25", bloques: [{ examen: "asn", titulo: "ASN", temas: "Acaba Ciclo de vida · + Alta disponibilidad · + Despliegue", sesiones: 30 }] },
    {
      fecha: "2026-11-26",
      nombre: "Jueves 26",
      bloques: [{ examen: "asn", titulo: "ASN", temas: "+ Monitorización · + Organización · + DFIR", sesiones: 26, simulacro: true }],
      nota: "Para antes de cenar. Duerme bien.",
    },
    {
      fecha: "2026-11-27",
      nombre: "Viernes 27",
      examen: "16:30 · Examen de ASN",
      bloques: [
        { examen: "asn", titulo: "ASN · hasta 13:30", temas: "+ Contenedores, y repaso con todo marcado", sesiones: 12 },
        { examen: "ps", titulo: "PS · desde 18:00", temas: "Introducción", sesiones: 12 },
      ],
    },
    { fecha: "2026-11-28", nombre: "Sábado 28", bloques: [{ examen: "ps", titulo: "PS", temas: "Acaba Introducción · + Claves · + Transporte", sesiones: 30 }] },
    {
      fecha: "2026-11-29",
      nombre: "Domingo 29",
      bloques: [{ examen: "ps", titulo: "PS", temas: "Acaba Transporte · + Aplicación · + SSL/TLS, por la tarde", sesiones: 26, simulacro: true }],
      nota: "Para antes de cenar. Duerme bien.",
    },
    {
      fecha: "2026-11-30",
      nombre: "Lunes 30",
      examen: "16:30 · Examen de PS",
      bloques: [{ examen: "ps", titulo: "PS · hasta 13:30", temas: "Acaba SSL/TLS, y repaso con todo marcado", sesiones: 15 }],
    },
  ];

  const dia = (d: Date) => `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  const hoy = dia(new Date());

  // Las respuestas de cada examen por día local. El viernes, ASN y PS comparten día: cada bloque cuenta las de su examen.
  let respuestas = $state<Record<string, Record<string, number>>>({});
  let error = $state("");
  for (const examen of ["asn", "ps"])
    fetch(`/${examen}/api/inicio`)
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(`El servidor responde ${r.status}`))))
      .then((d: Datos) => {
        const cuenta: Record<string, number> = {};
        for (const r of d.historial) cuenta[dia(new Date(r.ts))] = (cuenta[dia(new Date(r.ts))] ?? 0) + 1;
        respuestas[examen] = cuenta;
      })
      .catch((e) => (error = e.message));

  // Una sesión son 12 respuestas. El viernes, las de ASN van a su bloque, y las de PS al suyo.
  const hechas = (b: Bloque, fecha: string) => Math.floor((respuestas[b.examen]?.[fecha] ?? 0) / 12);
</script>

<main class="mx-auto max-w-[1100px] p-4">
  <p class="flex items-center justify-between text-[13px] text-tenue">
    <button class="cursor-pointer" onclick={() => history.back()}>← Volver</button>
    <Pomodoro />
  </p>
  <h1 class="mt-2 mb-1 text-[24px] font-bold">Plan de estudio</h1>
  <p class="mb-4 text-[13px] text-tenue">
    Una casilla = una sesión de 12 respuestas, y se marca sola. Cada fila de 4 = un bloque de 30 min, y después 5 a 10 min de pausa. + = marca ese tema en el inicio,
    sin desmarcar los anteriores.
  </p>
  <div class="grid gap-3 pc:grid-cols-4">
    {#each plan as d (d.fecha)}
      <section
        class={[
          "rounded-xl border-2 p-3",
          d.fecha === hoy ? "border-acento" : d.examen ? "border-texto" : "border-linea",
          d.fecha < hoy && "opacity-60",
        ]}
      >
        <h2 class="text-[16px] font-bold">{d.nombre}{#if d.fecha === hoy}<span class="ml-2 text-[12px] text-acento">hoy</span>{/if}</h2>
        {#each d.bloques as b, k (k)}
          {@const n = hechas(b, d.fecha)}
          {#if d.examen && k === 1}<p class="my-2 rounded-md bg-texto px-2 py-1 text-[12px] font-bold text-fondo">{d.examen}</p>{/if}
          <div class={["mt-2 rounded-lg border-l-4 bg-tarjeta p-2.5", b.examen === "asn" ? "border-acento" : "border-aviso"]}>
            <b class="text-[14px]">{b.titulo}</b>
            <p class="my-1.5 text-[12px] text-tenue">{b.temas}</p>
            <div class="grid w-max grid-cols-4 gap-1">
              {#each { length: b.sesiones }, j (j)}
                <i class={["size-4 rounded-sm border-[1.5px]", j < n ? (b.examen === "asn" ? "border-acento bg-acento" : "border-aviso bg-aviso") : "border-tenue"]}></i>
              {/each}
            </div>
            <p class="mt-1.5 text-[12px] text-tenue">{Math.min(n, b.sesiones)} de {b.sesiones} sesiones{#if n > b.sesiones} · +{n - b.sesiones} de más{/if}</p>
            {#if b.simulacro}<p class="mt-1 text-[12px]">+ simulacro a las 16:30</p>{/if}
          </div>
        {/each}
        {#if d.examen && d.bloques.length === 1}<p class="my-2 rounded-md bg-texto px-2 py-1 text-[12px] font-bold text-fondo">{d.examen}</p>{/if}
        {#if d.nota}<p class="mt-2 text-[12px] font-bold text-tenue">{d.nota}</p>{/if}
      </section>
    {/each}
  </div>
  <div class="mt-4 grid gap-3 text-[13px] text-tenue pc:grid-cols-4">
    <p><b class="text-texto">Cuándo pasar de tema.</b> Cuando ya no salen preguntas nuevas, aunque falles algunas. Los fallos vuelven solos en los repasos.</p>
    <p><b class="text-texto">Los fallos.</b> Lee la explicación, y abre la página del PDF si no entiendes por qué fallaste.</p>
    <p><b class="text-texto">El día del examen.</b> Estudia hasta las 13:30. Después come, descansa y no abras nada nuevo.</p>
    <p><b class="text-texto">Si un día te quedas corto.</b> Pasa las casillas al día siguiente. No las juntes todas en la víspera.</p>
  </div>
  {#if error}<p class="my-3 text-aviso">{error}</p>{/if}
</main>
