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
  class="pomodoro fixed top-0 left-1/2 z-50 flex -translate-x-1/2 rounded-b-lg text-[13px] font-semibold text-sobre-acento [.en-simulacro_&]:hidden"
  class:bg-estudio={e.fase === "estudio"}
  class:bg-pausa={e.fase !== "estudio"}
>
  <button class="py-0.5 pl-3 tabular-nums" class:opacity-60={!e.fin} title={e.fin ? "Pausar" : "Empezar"} onclick={alternar}>
    {Math.floor(segundos / 60)}:{String(segundos % 60).padStart(2, "0")}
  </button>
  <button class="peer px-2" title="Los controles">▾</button>
  <div
    class="absolute top-full right-0 hidden flex-col rounded-lg bg-tarjeta p-1 text-texto peer-hover:flex peer-focus:flex hover:flex focus-within:flex [&>button]:rounded [&>button]:px-2 [&>button]:py-0.5 [&>button]:text-left [&>button]:whitespace-nowrap [&>button:hover]:bg-linea"
  >
    <button onclick={alternar}>{e.fin ? "Pausar" : "Empezar"}</button>
    <button onclick={mas5}>+5 min</button>
    <button onclick={saltar}>Saltar {e.fase === "estudio" ? "el estudio" : "la pausa"}</button>
  </div>
</div>
