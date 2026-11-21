import { mount } from "svelte";
import App from "./App.svelte";
import cargarNucleo from "./nucleo/nucleo.js";

cargarNucleo().then(() => mount(App, { target: document.getElementById("app")! }));
