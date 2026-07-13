import { shallowRef, ref, computed, onBeforeUnmount, type Ref, type ShallowRef } from 'vue'

export interface PanelResizeOptions {
  initial: [number, number, number]
  minWidth?: number
}

export function usePanelResize(containerRef: Readonly<ShallowRef<HTMLElement | null>>, options: PanelResizeOptions) {
  const MIN_W = options.minWidth ?? 28

  const pxSizes = ref<[number, number, number]>(options.initial) as Ref<[number, number, number]>
  const dragging = shallowRef(-1)
  const dragStartPx = ref<[number, number, number]>(options.initial)
  const dragStartX = shallowRef(0)

  const sizes = computed(() => pxSizes.value)

  function initFromContainer() {
    const w = containerRef.value?.getBoundingClientRect().width ?? 1200
    pxSizes.value = [Math.round(w * 0.2), Math.round(w * 0.42), Math.round(w * 0.38)]
  }

  function panelStyle(idx: number): Record<string, string> {
    if (idx === 2 && sizes.value[idx] > MIN_W) {
      return { flex: '1 1 0px', minWidth: MIN_W + 'px' }
    }
    return { width: sizes.value[idx] + 'px', flex: 'none', minWidth: MIN_W + 'px' }
  }

  function togglePanel(idx: number) {
    const cur = sizes.value[idx]
    const newPx: [number, number, number] = [...pxSizes.value] as [number, number, number]
    if (cur <= MIN_W) {
      const total = containerRef.value?.getBoundingClientRect().width ?? 1200
      newPx[idx] = Math.round(total * 0.25)
      const sibs = [0, 1, 2].filter(i => i !== idx)
      for (const s of sibs) newPx[s] = Math.max(MIN_W, newPx[s] - (newPx[idx] - cur) / 2)
    } else {
      const excess = cur - MIN_W
      newPx[idx] = MIN_W
      const sibs = [0, 1, 2].filter(i => i !== idx && sizes.value[i] > MIN_W)
      if (sibs.length > 0) {
        const each = excess / sibs.length
        for (const s of sibs) newPx[s] += each
      }
    }
    pxSizes.value = newPx
  }

  function onHandleMousedown(idx: number, e: MouseEvent) {
    e.preventDefault()
    const newPx = [...pxSizes.value] as [number, number, number]
    if (newPx[idx] <= MIN_W) newPx[idx] = 100
    if (newPx[idx + 1] <= MIN_W) newPx[idx + 1] = 100
    pxSizes.value = newPx

    dragging.value = idx
    dragStartX.value = e.clientX
    dragStartPx.value = [...newPx]
    document.addEventListener('mousemove', onMousemove)
    document.addEventListener('mouseup', onMouseup)
  }

  function onMousemove(e: MouseEvent) {
    if (dragging.value < 0) return
    const dx = e.clientX - dragStartX.value
    const newPx: [number, number, number] = [...dragStartPx.value]
    const i = dragging.value
    newPx[i] = Math.max(MIN_W, dragStartPx.value[i] + dx)
    newPx[i + 1] = Math.max(MIN_W, dragStartPx.value[i + 1] - dx)
    pxSizes.value = newPx
  }

  function onMouseup() {
    dragging.value = -1
    document.removeEventListener('mousemove', onMousemove)
    document.removeEventListener('mouseup', onMouseup)
  }

  onBeforeUnmount(() => {
    document.removeEventListener('mousemove', onMousemove)
    document.removeEventListener('mouseup', onMouseup)
  })

  return {
    MIN_W,
    pxSizes,
    sizes,
    dragging,
    initFromContainer,
    panelStyle,
    togglePanel,
    onHandleMousedown,
  }
}
