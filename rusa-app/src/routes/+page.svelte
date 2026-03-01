<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let greetMsg = $state("");
  let name = $state("");

  async function greet() {
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="container">
  <h1>Welcome to RUSA App</h1>

  <form class="row" onsubmit={(e) => { e.preventDefault(); greet(); }}>
    <input id="greet-input" placeholder="Enter a name..." bind:value={name} />
    <button type="submit">Greet</button>
  </form>

  <p>{greetMsg}</p>
</main>

<style>
  .container {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    font-family: sans-serif;
  }

  .row {
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  input {
    padding: 0.5rem;
    border: 1px solid #ccc;
    border-radius: 4px;
  }

  button {
    padding: 0.5rem 1rem;
    background-color: #0366d6;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  button:hover {
    background-color: #0256b9;
  }
</style>
