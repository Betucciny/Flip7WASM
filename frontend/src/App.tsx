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
      <h1>Flip7 Helper</h1>
      <p>Rust says:</p>
      <pre>{message()}</pre>
    </div>
  );
}
