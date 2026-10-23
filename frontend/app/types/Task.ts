// Интерфейсы под ответы бэкенда (backend/src/tasks/model.rs)

export type Mode = 'javascript' | 'python' | 'css'
export type Difficulty = 'easy' | 'medium' | 'hard'

export interface TestCase {
  args: unknown[]
  expected: unknown
  is_hidden: boolean
}

// Элемент списка GET /api/tasks
export interface TaskListItem {
  id: number
  mode: Mode
  difficulty: Difficulty
  title: string
  source: 'manual' | 'llm' // llm — задачу написала нейросеть
}

// Задача целиком GET /api/tasks/{id} — без скрытых тестов, только их число
export interface TaskDetail extends TaskListItem {
  statement_md: string
  function_name: string | null
  params: string[] // имена аргументов — из них строим заготовку кода
  examples: TestCase[]
  hidden_tests_count: number
  target_html: string | null
}
