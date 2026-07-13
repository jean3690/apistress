import dayjs from 'dayjs'
import duration from 'dayjs/plugin/duration'

dayjs.extend(duration)

export function now(): number {
  return dayjs().valueOf()
}

export function formatElapsed(seconds: number): string {
  const d = dayjs.duration(seconds, 'seconds')
  const m = String(Math.floor(d.asMinutes())).padStart(2, '0')
  const s = String(d.seconds()).padStart(2, '0')
  return `${m}:${s}`
}

export function formatTimestamp(ms: number): string {
  return dayjs(ms).format('HH:mm:ss')
}
