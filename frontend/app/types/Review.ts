// Разбор решения от нейросети (backend/src/llm/review.rs)

export interface Review {
  summary: string // решена ли задача и главное впечатление
  approach: string // идея правильного решения
  mistakes: string[]
  tips: string[]
}
