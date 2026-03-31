<script lang="ts">
  import { goto } from "$app/navigation";
  import { indexing, progress, runIndex } from "../lib/stores/indexing";
  import { ArrowRight, RefreshCw, Search, Settings2 } from "@lucide/svelte";

  let query = $state("");
  let searchInput: HTMLInputElement | null = null;

  const onWindowKeydown = (event: KeyboardEvent) => {
    if (event.metaKey && event.key.toLowerCase() === "k") {
      event.preventDefault();
      searchInput?.focus();
      searchInput?.select();
    }

    if (event.metaKey && event.key.toLowerCase() === "i") {
      event.preventDefault();
      runIndex();
    }
  };

  const search = async () => {
    if (!query.trim()) return;
    goto(`/results?q=${encodeURIComponent(query)}`);
  };

  const year = new Date().getFullYear();
</script>

<svelte:window onkeydown={onWindowKeydown} />

<div class="min-h-screen flex flex-col">
  <header>
    <nav class="m-3 flex items-center justify-end">
      <div class="flex gap-3">
        <button
          onclick={runIndex}
          title="Index files ⌘I"
          class="bg-[#1E1E1E] h-10 w-10 flex items-center justify-center rounded-md transition-all duration-100 active:scale-90 hover:opacity-90"
        >
          <RefreshCw
            class={$indexing ? "animate-spin" : ""}
            size={20}
            color="white"
          />
        </button>
        <button
          onclick={() => goto("/settings")}
          class="bg-[#1E1E1E] h-10 w-10 flex items-center justify-center rounded-md transition-all duration-100 active:scale-90 hover:opacity-90"
          title="Settings ⌘,"
        >
          <Settings2 size={20} color="white" />
        </button>
      </div>
    </nav>
  </header>

  <main class="flex-1">
    <div class="flex flex-col justify-center items-center select-none">
      <h1 class="text-[75px] font-bold">FOLIO</h1>
      <h3 class="text-[18px]">Every frame fully searchable.</h3>
    </div>

    <form
      onsubmit={(e) => {
        e.preventDefault();
        search();
      }}
      class="mt-10 flex justify-center"
    >
      <div class="relative">
        <Search
          size={18}
          class="absolute left-4 top-1/2 -translate-y-1/2 text-gray-500"
        />
        <input
          type="text"
          placeholder="search by description or dialogue..."
          bind:value={query}
          bind:this={searchInput}
          class="w-118.75 h-13 border-2 border-gray-300 bg-white rounded-md p-5 pl-11 pr-28 focus:border-gray-500 focus:outline-none"
        />
        <kbd
          class="pointer-events-none absolute right-4 top-1/2 -translate-y-1/2 text-gray-500 tracking-widest"
        >
          ⌘K
        </kbd>
      </div>
    </form>
  </main>

  <footer
    class="flex m-5 justify-between select-none text-[14px] text-gray-900"
  >
    <p>Indexed: {$progress?.current ?? 0} files</p>
    <p>&copy; {year} Aditya Chaturvedi</p>
  </footer>
</div>

<style>
</style>
