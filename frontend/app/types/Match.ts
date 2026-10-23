// Интерфейсы под ответы бэкенда (backend/src/matches/model.rs)
import type { League } from '~/types/Auth'
import type { RunResult } from '~/types/Run'
import type { Difficulty, Mode, TaskDetail } from '~/types/Task'

export type BotLevel = 'novice' | 'middle' | 'senior'
export type MatchResult = 'win' | 'loss' | 'draw'
export type MatchStatus = 'countdown' | 'active' | 'finished'

export interface MatchView {
  id: number
  status: MatchStatus
  mode: Mode
  difficulty: Difficulty
  bot_level: BotLevel
  bot_rating: number
  task: TaskDetail
  server_now: number // по нему выравниваем часы браузера
  starts_at: number
  ends_at: number
  total_tests: number
  player_passed: number
  bot_passed: number
  result: MatchResult | null
  finished_at: number | null // после конца таймер замирает на этом моменте
  rating_delta: number | null
  rating_after: number | null
  league: League | null
  solution: string | null // эталон — только после конца матча
}

export interface SubmitResponse {
  run: RunResult
  match: MatchView
}

export interface MatchHistoryItem {
  id: number
  task_title: string
  mode: Mode
  difficulty: Difficulty
  bot_level: BotLevel
  player_passed: number
  total_tests: number
  result: MatchResult | null
  rating_delta: number | null
  finished_at: number | null
}
