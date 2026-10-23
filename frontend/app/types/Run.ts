// Интерфейсы под ответ POST /api/tasks/{id}/run и /submit (backend/src/runner/model.rs)

export type Verdict = 'accepted' | 'wrong_answer' | 'runtime_error' | 'time_limit'

export interface TestResult {
  index: number
  is_hidden: boolean
  passed: boolean
  args: unknown[] | null // у скрытых тестов всегда null
  expected: unknown | null
  got: unknown | null
  error: string | null
  time_ms: number
}

export interface RunResult {
  verdict: Verdict
  passed: number
  total: number
  results: TestResult[]
  stdout: string
  error: string | null
}
