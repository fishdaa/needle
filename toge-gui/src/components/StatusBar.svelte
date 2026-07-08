<script lang="ts">
  import { onMount } from 'svelte'
  import { statusText, isLoading, indexStatusText, fetchStatus, results, selectedIndex, totalCount, sizeIndexed, formatSize, formatTimestamp } from '$lib/searchStore'

  let refreshTimer: ReturnType<typeof setInterval> | null = null

  onMount(() => {
    fetchStatus()
    refreshTimer = setInterval(() => {
      fetchStatus()
    }, 3000)

    return () => {
      if (refreshTimer) clearInterval(refreshTimer)
    }
  })

  const selectedRow = $derived(($selectedIndex >= 0 && $selectedIndex < $results.length) ? $results[$selectedIndex] : null)
  const selectedDetails = $derived.by(() => {
    if (!selectedRow) return ''

    const sizeLabel = $sizeIndexed ? formatSize(selectedRow.size_bytes) : 'size unavailable'
    const modifiedLabel = formatTimestamp(selectedRow.modified_unix) || 'unknown'
    return `Size: ${sizeLabel}, Date Modified: ${modifiedLabel}, Path: ${selectedRow.parent}`
  })
</script>

<div class="status-bar">
  {#if selectedRow}
    <div class="status-group">
      <span class="status-text">{selectedDetails}</span>
    </div>
    <div class="status-group status-group-right">
      <span class="status-text">{Math.max(0, $selectedIndex + 1)} of {$totalCount}</span>
    </div>
  {:else}
    <div class="status-group">
      <span class="status-label">Search</span>
      <span class="status-text">{$statusText}</span>
      <span class="loading-indicator" class:visible={$isLoading} aria-hidden={!$isLoading}>⟳</span>
    </div>
    <div class="status-group status-group-right">
      <span class="status-label">Index</span>
      <span class="status-text">{$indexStatusText}</span>
    </div>
  {/if}
</div>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 3px 8px;
    border-top: 1px solid var(--border);
    background: color-mix(in srgb, var(--bg-surface) 96%, var(--bg));
    font-size: 12px;
    color: var(--text-secondary);
    min-height: 24px;
  }

  .status-group {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .status-group-right {
    justify-content: flex-end;
    flex: 1;
  }

  .status-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-placeholder);
    flex-shrink: 0;
  }

  .status-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-secondary);
  }

  .loading-indicator {
    animation: spin 1s linear infinite;
    flex-shrink: 0;
    width: 1em;
    text-align: center;
    visibility: hidden;
    opacity: 0;
    transition: opacity 120ms ease;
  }

  .loading-indicator.visible {
    visibility: visible;
    opacity: 1;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
