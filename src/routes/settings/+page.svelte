<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog'
  import { goto } from '$app/navigation'
  import {
    ArrowLeft,
    Check,
    Folder,
    Minus,
    Plus,
    RefreshCw,
    Search,
    X,
  } from '@lucide/svelte'

  import { indexing, runIndex } from '../../lib/stores/indexing'
  import { settingsStore } from '../../lib/stores/settings.svelte'

  settingsStore.load()

  let dirs = $state<string[]>([])
  let exts = $state<string[]>([])
  let transcription = $state(false)

  let saving = $state(false)
  let addingExt = $state(false)
  let extInput = $state('')

  let initialDirs = $state<string[]>([])
  let initialExts = $state<string[]>([])
  let initialTranscription = $state(false)

  let seeded = $state(false)

  $effect(() => {
    const s = settingsStore.value
    if (s && !seeded) {
      dirs = [...s.dirs]
      exts = [...s.exts]
      transcription = s.transcription

      initialDirs = [...s.dirs]
      initialExts = [...s.exts]
      initialTranscription = s.transcription

      seeded = true
    }
  })

  const checkSameArray = (a: string[], b: string[]) =>
    a.length === b.length && a.every((value, index) => value === b[index])

  const canApply = $derived(
    !checkSameArray(dirs, initialDirs) ||
      !checkSameArray(exts, initialExts) ||
      transcription !== initialTranscription
  )

  const save = async () => {
    if (!canApply) return

    saving = true
    const s = { dirs, exts, transcription }
    await settingsStore.save(s)

    initialDirs = [...dirs]
    initialExts = [...exts]
    initialTranscription = transcription

    saving = false
  }

  const confirmAddExt = () => {
    const normalized = extInput.trim().replace(/^\./, '').toLowerCase()
    if (!normalized || exts.includes(normalized)) {
      addingExt = false
      extInput = ''
      return
    }

    exts = [...exts, normalized]
    addingExt = false
    extInput = ''
  }

  const cancelAddExt = () => {
    addingExt = false
    extInput = ''
  }
</script>

<div class="min-h-screen flex flex-col">
  <header>
    <nav class="m-3 flex items-center justify-between select-none">
      <div class="flex">
        <button
          onclick={() => history.back()}
          class="flex items-center gap-2 ml-3 hover:scale-105 active:scale-90 transition-all duration-100"
        >
          <ArrowLeft class="" size={24} />
        </button>
        <h1 class="text-[19px] font-bold ml-2">Settings</h1>
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
            title="Index files ⌘I"
          />
        </button>
        <button
          onclick={() => goto('/')}
          class="bg-[#1E1E1E] h-10 w-10 flex items-center justify-center rounded-md transition-all duration-100 active:scale-90 hover:opacity-90"
          title="Home"
        >
          <Search size={20} color="white" />
        </button>
      </div>
    </nav>
  </header>

  <main class="flex-1 px-6 py-4 flex flex-col gap-6 select-none w-200 mx-auto">
    <!-- Watched Directories -->
    <section>
      <div class="flex items-center justify-between mb-2">
        <span class="text-[15px] font-medium">Watched directories</span>
        <button
          onclick={async () => {
            const selected = await open({ directory: true, multiple: true })
            if (!selected) return
            const picked = Array.isArray(selected) ? selected : [selected]
            dirs = [...new Set([...dirs, ...picked])]
          }}
          class="h-7 w-7 flex items-center justify-center rounded-sm hover:bg-gray-200 transition-colors active:scale-90"
        >
          <Plus size={18} />
        </button>
      </div>

      {#if dirs.length > 0}
        <div class="bg-white rounded-xl overflow-hidden border border-gray-200">
          {#each dirs as dir, i}
            <button
              class="w-full flex items-center gap-3 px-4 py-3 hover:bg-gray-50 transition-colors text-left {i <
              dirs.length - 1
                ? 'border-b border-gray-100'
                : ''}"
              onclick={() => {
                dirs = dirs.filter(d => d !== dir)
              }}
              title="Click to remove {dir}"
            >
              <Folder size={18} class="text-gray-500 shrink-0" />
              <span class="text-[14px] truncate">{dir}</span>
              <Minus class="shrink-0 ml-auto" />
            </button>
          {/each}
        </div>
      {/if}
    </section>

    <!-- File Extensions -->
    <section>
      <div class="flex items-center justify-between mb-2">
        <span class="text-[15px] font-medium">File extensions</span>
        {#if addingExt}
          <button
            onclick={() => {
              addingExt = false
            }}
            class="h-7 w-7 flex items-center justify-center rounded-md hover:bg-gray-200 transition-colors"
          >
            <Minus size={18} />
          </button>
        {:else}
          <button
            onclick={() => {
              addingExt = true
            }}
            class="h-7 w-7 flex items-center justify-center rounded-md hover:bg-gray-200 transition-colors"
          >
            <Plus size={18} />
          </button>
        {/if}
      </div>

      {#if addingExt}
        <div class="mb-3 flex items-center gap-2">
          <input
            bind:value={extInput}
            placeholder="e.g. webm"
            class="h-8 rounded-sm border border-gray-300 bg-white px-2 text-[14px] focus:outline-none focus:border-gray-500"
            onkeydown={e => {
              if (e.key === 'Enter') {
                e.preventDefault()
                confirmAddExt()
              }

              if (e.key === 'Escape') {
                e.preventDefault()
                cancelAddExt()
              }
            }}
          />
        </div>
      {/if}

      <div class="flex flex-wrap gap-2">
        {#each exts as ext}
          <button
            onclick={() => {
              exts = exts.filter(e => e !== ext)
            }}
            title="Click to remove .{ext}"
            class="flex items-center justify-center gap-1 bg-white border border-gray-200 rounded-md px-3 py-1 text-[14px] hover:bg-[#1E1E1E] hover:border-[#1E1E1E] hover:text-white transition-colors duration-100 active:scale-90"
          >
            <span>.{ext}</span>
            <X size={14} />
          </button>
        {/each}
      </div>
    </section>

    <!-- Toggle Transcriptions -->
    <section>
      <span class="text-[15px] font-medium block mb-2">Transcription</span>
      <label class="flex items-center justify-between cursor-pointer mt-4">
        <span class="text-[14px]">
          Enable automatic transcription (whisper)
        </span>

        <input type="checkbox" bind:checked={transcription} class="sr-only" />

        <span
          class="h-5.5 w-5.5 rounded-xs border-[#1E1E1E] border flex items-center justify-center transition-all {transcription
            ? 'bg-[#1E1E1E]'
            : 'bg-transparent'}"
        >
          {#if transcription}
            <Check size={14} color="white" />
          {/if}
        </span>
      </label>
    </section>

    <!-- Apply Button -->
    <button
      onclick={save}
      disabled={!canApply && saving}
      class="w-fit text-[14px] font-medium px-5 py-2.5 rounded-md transition-all duration-100 {canApply
        ? 'bg-[#1E1E1E] text-white active:scale-95 hover:opacity-90'
        : 'bg-transparent border border-gray-400 text-gray-500 cursor-not-allowed'} {saving
        ? 'cursor-wait'
        : ''}"
    >
      {saving ? 'Applying...' : 'Apply'}
    </button>
  </main>
</div>
