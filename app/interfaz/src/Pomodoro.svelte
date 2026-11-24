<script lang="ts">
  // El cronómetro de pausas: 25 minutos de estudio y 5 de pausa, y 20 de pausa tras cada 3 de estudio.
  // Guarda la hora de fin en `localStorage`, así que sigue al recargar y al cambiar de pantalla.
  // Al acabar una fase, suena y espera un clic para la siguiente.
  type Fase = "estudio" | "pausa" | "larga";
  type Estado = { fase: Fase; hechos: number; fin: number | null; resta: number };
  const MIN = 60_000;
  const duracion: Record<Fase, number> = { estudio: 25 * MIN, pausa: 5 * MIN, larga: 20 * MIN };
  const clave = "pomodoro";

  let e = $state<Estado>(JSON.parse(localStorage.getItem(clave) ?? "null") ?? { fase: "estudio", hechos: 0, fin: null, resta: duracion.estudio });
  let ahora = $state(Date.now());
  const resta = $derived(e.fin ? Math.max(0, e.fin - ahora) : e.resta);
  const segundos = $derived(Math.ceil(resta / 1000));
  // "+5 min" alarga la fase, así que la barra cuenta sobre la fase alargada.
  const hecho = $derived(1 - resta / Math.max(duracion[e.fase], resta));
  const empezada = $derived(!!e.fin || e.resta !== duracion[e.fase]);
  const nombre = $derived(e.fase === "estudio" ? `Estudio · ${(e.hechos % 3) + 1} de 3` : e.fase === "pausa" ? "Pausa" : "Pausa larga");

  const guardar = (n: Estado) => localStorage.setItem(clave, JSON.stringify((e = n)));
  // Otra pestaña cambia el cronómetro, y esta lo sigue.
  addEventListener("storage", (ev) => ev.key === clave && ev.newValue && (e = JSON.parse(ev.newValue)));

  setInterval(() => {
    ahora = Date.now();
    if (e.fin && e.fin <= ahora) {
      sonar();
      saltar();
    }
  }, 1000);

  function saltar() {
    const hechos = e.fase === "estudio" ? e.hechos + 1 : e.hechos;
    const fase: Fase = e.fase !== "estudio" ? "estudio" : hechos % 3 ? "pausa" : "larga";
    guardar({ fase, hechos, fin: null, resta: duracion[fase] });
  }

  // El navegador solo deja sonar tras un clic, así que el audio nace al arrancar.
  let audio: AudioContext | undefined;
  function alternar() {
    ahora = Date.now();
    if (e.fin) guardar({ ...e, fin: null, resta: Math.max(0, e.fin - ahora) });
    else {
      audio ??= new AudioContext();
      guardar({ ...e, fin: ahora + e.resta });
    }
  }

  const mas5 = () => guardar(e.fin ? { ...e, fin: e.fin + 5 * MIN } : { ...e, resta: e.resta + 5 * MIN });

  // 3 notas suaves de onda senoidal. En un simulacro, el cronómetro calla.
  function sonar() {
    if (!audio || document.documentElement.classList.contains("en-simulacro")) return;
    [660, 880, 660].forEach((f, k) => {
      const t = audio!.currentTime + k * 0.4;
      const o = audio!.createOscillator();
      const g = audio!.createGain();
      o.frequency.value = f;
      g.gain.setValueAtTime(0, t);
      g.gain.linearRampToValueAtTime(0.15, t + 0.05);
      g.gain.exponentialRampToValueAtTime(0.001, t + 0.38);
      o.connect(g).connect(audio!.destination);
      o.start(t);
      o.stop(t + 0.4);
    });
  }
</script>

<div
  class="pomodoro fixed top-0 left-1/2 z-50 flex -translate-x-1/2 items-center rounded-b-lg text-[13px] font-semibold text-sobre-acento shadow-md [.en-simulacro_&]:hidden"
  class:bg-estudio={e.fase === "estudio"}
  class:bg-pausa={e.fase !== "estudio"}
>
  <button class="flex items-center gap-1.5 py-1 pl-3 tabular-nums" title={e.fin ? "Pausar" : empezada ? "Seguir" : "Empezar"} onclick={alternar}>
    {#if e.fin}{@render icono(11, "M8 5v14M16 5v14")}{:else}{@render icono(11, "M6 4l14 8-14 8z", true)}{/if}
    {Math.floor(segundos / 60)}:{String(segundos % 60).padStart(2, "0")}
  </button>
  <button class="peer self-stretch px-2 opacity-70 hover:opacity-100 focus:opacity-100" title="Los controles">{@render icono(12, "M6 9l6 6 6-6")}</button>
  <!-- La barra del avance de la fase. -->
  <span class="absolute inset-x-2 bottom-0.5 h-0.5 rounded-full bg-sobre-acento/15">
    <span class="block h-full rounded-full bg-sobre-acento/50" style:width="{hecho * 100}%"></span>
  </span>
  <!-- El relleno de arriba, y no un margen, deja pasar el ratón de la flecha a los controles. -->
  <div class="absolute top-full left-1/2 hidden min-w-full -translate-x-1/2 pt-1 peer-hover:block peer-focus:block hover:block focus-within:block">
    <div class="flex flex-col gap-1 rounded-lg border border-linea bg-tarjeta p-1.5 text-texto shadow-lg">
      <span class="px-1 text-[11px] font-normal whitespace-nowrap text-tenue">{nombre}</span>
      <div class="flex gap-1 [&>button]:flex [&>button]:h-7 [&>button]:flex-1 [&>button]:items-center [&>button]:justify-center [&>button]:rounded-md [&>button]:px-2 [&>button]:hover:bg-linea">
        <button title={e.fin ? "Pausar" : empezada ? "Seguir" : "Empezar"} onclick={alternar}>
          {#if e.fin}{@render icono(14, "M8 5v14M16 5v14")}{:else}{@render icono(14, "M6 4l14 8-14 8z", true)}{/if}
        </button>
        <button title="+5 min" onclick={mas5}>+5</button>
        <button title={e.fase === "estudio" ? "Saltar el estudio" : "Saltar la pausa"} onclick={saltar}>{@render icono(14, "M5 4l10 8-10 8zM19 5v14")}</button>
      </div>
    </div>
  </div>
</div>

{#snippet icono(lado: number, d: string, relleno = false)}
  <svg width={lado} height={lado} viewBox="0 0 24 24" fill={relleno ? "currentColor" : "none"} stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
    <path {d} />
  </svg>
{/snippet}
