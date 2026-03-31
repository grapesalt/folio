<script lang="ts">
  import "../app.css";
  import { browser } from "$app/environment";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { fly, fade } from "svelte/transition";
  import { initIndexingEvents } from "../lib/stores/indexing";
  import { settingsStore } from "../lib/stores/settings.svelte";
  let { children } = $props();

  if (browser) {
    void initIndexingEvents();
    void settingsStore.load();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!e.metaKey && !e.ctrlKey) return;
    if (e.key === ",") {
      e.preventDefault();
      goto("/settings");
    } else if (e.key === "[") {
      e.preventDefault();
      history.back();
    } else if (e.key === "]") {
      e.preventDefault();
      history.forward();
    }
  }

  $effect(() => {
    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });
</script>

{#key page.url.pathname}
  <div in:fly={{ x: -100, duration: 400 }}>
    {@render children()}
  </div>
{/key}
