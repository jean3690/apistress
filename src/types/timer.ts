import type { TestElement } from './common'

export interface ConstantTimer extends TestElement {
  type: 'ConstantTimer'
  delay: number // ms
}

export interface UniformRandomTimer extends TestElement {
  type: 'UniformRandomTimer'
  minDelay: number // ms
  maxDelay: number // ms
}

export interface GaussianRandomTimer extends TestElement {
  type: 'GaussianRandomTimer'
  delay: number // ms (mean)
  deviation: number // ms
}

export type TimerUnion = ConstantTimer | UniformRandomTimer | GaussianRandomTimer

export function createDefaultConstantTimer(id: string): ConstantTimer {
  return { id, type: 'ConstantTimer', name: 'Constant Timer', enabled: true, delay: 300 }
}
