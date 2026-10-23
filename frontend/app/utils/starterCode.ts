// Заготовка кода под задачу: функция с нужными аргументами, чтобы не набирать её руками
import type { Mode } from '~/types/Task'

export function makeStarterCode(mode: Mode, functionName: string | null, params: string[]) {
  const name = functionName ?? 'solve'
  const args = params.join(', ')

  if (mode === 'python') {
    return `def ${name}(${args}):\n    pass\n`
  }

  if (mode === 'css') {
    return '<div></div>\n<style>\n  body {\n    margin: 0;\n  }\n</style>\n'
  }

  return `function ${name}(${args}) {\n  \n}\n`
}

// 125 000 мс → «02:05»
export function formatClock(ms: number) {
  const totalSeconds = Math.max(0, Math.ceil(ms / 1000))
  const minutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds % 60

  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`
}
