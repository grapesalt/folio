<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import { page } from '$app/stores'
  import { onMount } from 'svelte'
  import { ArrowLeft, RefreshCw, Settings2 } from '@lucide/svelte'
  import { goto } from '$app/navigation'
  import { runIndex, indexing } from '../../lib/stores/indexing'

  interface Segment {
    start: number
    end: number
    text: string
  }

  interface SearchResult {
    file: string
    segment: Segment
    score: number
  }

  interface ResultWithThumb extends SearchResult {
    thumbnail: string | null
    loading: boolean
  }

  let query = $derived($page.url.searchParams.get('q') ?? '')
  let results = $state<ResultWithThumb[]>([])
  let searching = $state(false)
  let error = $state<string | null>(null)

  const msToTimestamp = (ms: number) => {
    const s = Math.floor(ms / 1000)
    const m = Math.floor(s / 60)
    const h = Math.floor(m / 60)
    if (h > 0) {
      return `${h}:${(m % 60).toString().padStart(2, '0')}:${(s % 60).toString().padStart(2, '0')}`
    }
    return `${m}:${(s % 60).toString().padStart(2, '0')}`
  }

  const segmentDurationMs = (segment: Segment) =>
    Math.max(0, segment.end - segment.start)

  const basename = (path: string) => path.split(/[\\/]/).pop() ?? path

  const loadThumbnail = async (idx: number, result: SearchResult) => {
    results[idx].loading = true
    try {
      const thumb = await invoke<string>('get_thumbnail', { res: result })
      results[idx].thumbnail = thumb
    } catch {
      results[idx].thumbnail = null
    } finally {
      results[idx].loading = false
    }
  }

  const doSearch = async (q: string) => {
    if (!q.trim()) return
    searching = true
    error = null
    results = []
    try {
      const raw = await invoke<SearchResult[]>('search', { query: q })
      results = raw.map(r => ({ ...r, thumbnail: null, loading: false }))
      for (let i = 0; i < results.length; i++) {
        loadThumbnail(i, raw[i])
      }
    } catch (e) {
      error = String(e)
    } finally {
      searching = false
    }
  }

  let prev = ''

  $effect(() => {
    if (query !== prev) {
      prev = query
      doSearch(query)
    }
  })

  onMount(() => doSearch(query))
</script>

<div class="min-h-screen flex flex-col">
  <header>
    <nav class="m-3 flex items-center justify-between select-none">
      <div class="flex items-center">
        <button
          onclick={() => history.back()}
          class="flex items-center gap-2 ml-3 hover:scale-105 active:scale-90 transition-all duration-100"
        >
          <ArrowLeft size={24} />
        </button>
        <div class="ml-2">
          <h1 class="text-[19px] font-bold">Results</h1>
          <p class="text-[14px] text-gray-500">
            Showing results for <span class="text-[#1E1E1E]">"{query}"</span>
          </p>
        </div>
      </div>
      <div class="flex gap-3">
        <button
          onclick={runIndex}
          class="bg-[#1E1E1E] h-10 w-10 flex items-center justify-center rounded-md transition-all duration-100 active:scale-90 hover:opacity-90"
        >
          <RefreshCw
            class={$indexing ? 'animate-spin' : ''}
            size={20}
            color="white"
          />
        </button>
        <button
          onclick={() => goto('/settings')}
          class="bg-[#1E1E1E] h-10 w-10 flex items-center justify-center rounded-md transition-all duration-100 active:scale-90 hover:opacity-90"
        >
          <Settings2 size={20} color="white" />
        </button>
      </div>
    </nav>
  </header>

  <main class="flex-1 flex flex-col items-center px-4 py-4">
    <div class="w-full max-w-200">
      {#if searching}
        <div class="grid grid-cols-3 gap-3">
          {#each { length: 9 } as _}
            <div
              class="bg-white border border-gray-200 rounded-xl overflow-hidden animate-pulse"
            >
              <div class="aspect-video bg-gray-200"></div>
              <div class="p-3 flex flex-col gap-1.5">
                <div class="h-3 w-2/3 bg-gray-200 rounded"></div>
                <div class="h-3 w-full bg-gray-200 rounded"></div>
                <div class="h-3 w-4/5 bg-gray-200 rounded"></div>
              </div>
            </div>
          {/each}
        </div>
      {:else if error}
        <p class="text-[14px] text-red-500 mt-8 text-center">Error: {error}</p>
      {:else if results.length === 0}
        <p class="text-[14px] text-gray-500 mt-8 text-center">
          No results for "{query}".
        </p>
      {:else}
        <p class="text-[14px] text-gray-500 mb-3 select-none">
          {results.length} result{results.length === 1 ? '' : 's'}
        </p>
        <div class="grid grid-cols-3 gap-3">
          {#each results as result}
            <div
              class="bg-white border border-gray-200 rounded-xl overflow-hidden flex flex-col"
            >
              <!-- Thumbnail -->
              <div class="relative aspect-video bg-gray-200 shrink-0">
                {#if result.loading}
                  <div class="absolute inset-0 bg-gray-200 animate-pulse"></div>
                {:else if result.thumbnail}
                  <img
                    src={result.thumbnail}
                    alt="thumbnail"
                    class="w-full h-full object-cover"
                  />
                {/if}
                <!-- Duration badge -->
                <span
                  class="absolute bottom-1.5 right-1.5 bg-black/70 text-white text-[11px] font-medium px-1.5 py-0.5 rounded"
                >
                  {msToTimestamp(segmentDurationMs(result.segment))}
                </span>
              </div>
              <!-- Info -->
              <div class="px-3 py-2.5 flex flex-col gap-1">
                <p class="text-[14px] font-medium truncate text-[#1E1E1E]">
                  {basename(result.file)}
                </p>
                <p class="text-[14px] text-gray-600 leading-snug line-clamp-3">
                  {@html result.segment.text}
                </p>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </main>
</div>
