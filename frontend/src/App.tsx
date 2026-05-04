import { createSignal, onMount } from "solid-js";
import init, { greet } from "@engine/engine";

export default function App() {
  const [message, setMessage] = createSignal("Loading WASM...");

  onMount(async () => {
    try {
      await init(); // initialize wasm
      const result = greet(); // call Rust function
      setMessage(result);
    } catch (err) {
      console.error(err);
      setMessage("Failed to load WASM");
    }
  });

  return (
    <div style={{ padding: "2rem", "font-family": "sans-serif" }}>
      <h1 class="text-3xl font-bold underline">Hello world!</h1>
      <p class="text-2xl">Rust says:</p>
      <pre class="underline">{message()}</pre>
    </div>
  );
}
