<script lang="ts">
  import { onMount } from "svelte";
  import { getConfig } from "$lib/api";
  import { activeView, configStore } from "$lib/stores.svelte";
  import ConfigView from "$lib/components/ConfigView.svelte";
  import ScanView from "$lib/components/ScanView.svelte";
  import ReportView from "$lib/components/ReportView.svelte";

  let loadError = $state<string | null>(null);
  let loading = $state(true);

  onMount(async () => {
    // Default to the Config view on first load.
    activeView.value = "config";
    try {
      configStore.value = await getConfig();
    } catch (e) {
      loadError = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });
</script>

{#if loading}
  <div class="boot">Loading configuration…</div>
{:else if loadError}
  <div class="boot error">Failed to load configuration: {loadError}</div>
{:else if activeView.value === "config"}
  <ConfigView />
{:else if activeView.value === "scan"}
  <ScanView />
{:else}
  <ReportView />
{/if}

<style>
  .boot {
    padding: 2rem;
    text-align: center;
    color: #b8c0d8;
  }

  .boot.error {
    color: #e94560;
  }
</style>
