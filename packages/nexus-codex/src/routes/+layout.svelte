<script lang="ts">
  import { activeView, type ActiveView } from "$lib/stores.svelte";

  let { children } = $props();

  const tabs: { id: ActiveView; label: string }[] = [
    { id: "config", label: "Config" },
    { id: "scan", label: "Scan" },
    { id: "report", label: "Report" },
  ];

  function select(view: ActiveView) {
    activeView.value = view;
  }
</script>

<div class="app">
  <header class="navbar">
    <div class="brand">
      <span class="brand-mark">◆</span>
      <span class="brand-name">Nexus Codex</span>
    </div>
    <nav class="tabs">
      {#each tabs as tab (tab.id)}
        <button
          class="tab"
          class:active={activeView.value === tab.id}
          onclick={() => select(tab.id)}
        >
          {tab.label}
        </button>
      {/each}
    </nav>
  </header>

  <main class="content">
    {@render children()}
  </main>
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    height: 100%;
  }

  :global(body) {
    background: #1a1a2e;
    color: #ffffff;
    font-family: "Inter", "Segoe UI", system-ui, -apple-system, sans-serif;
    font-size: 14px;
    line-height: 1.5;
  }

  :global(*) {
    box-sizing: border-box;
  }

  :global(button) {
    font-family: inherit;
  }

  .app {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  .navbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1.25rem;
    height: 56px;
    background: #16213e;
    border-bottom: 1px solid #0f3460;
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 700;
    font-size: 1.05rem;
  }

  .brand-mark {
    color: #e94560;
    font-size: 1.1rem;
  }

  .brand-name {
    letter-spacing: 0.02em;
  }

  .tabs {
    display: flex;
    gap: 0.25rem;
  }

  .tab {
    background: transparent;
    border: none;
    color: #b8c0d8;
    padding: 0.5rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 600;
    transition: background 0.15s, color 0.15s;
  }

  .tab:hover {
    background: #0f3460;
    color: #ffffff;
  }

  .tab.active {
    background: #e94560;
    color: #ffffff;
  }

  .content {
    flex: 1;
    padding: 1.5rem;
    width: 100%;
    max-width: 1200px;
    margin: 0 auto;
  }
</style>
