// Подписи и цвета — нужны в карточках, фильтрах, матче и профиле
import type { League } from '~/types/Auth'
import type { BotLevel, MatchResult } from '~/types/Match'
import type { Verdict } from '~/types/Run'
import type { Difficulty, Mode } from '~/types/Task'

export const MODE_LABELS: Record<Mode, string> = {
  javascript: 'JavaScript',
  python: 'Python',
  css: 'CSS',
}

export const DIFFICULTY_LABELS: Record<Difficulty, string> = {
  easy: 'Лёгкая',
  medium: 'Средняя',
  hard: 'Сложная',
}

// Плашка сложности: цветной текст на прозрачной подложке того же цвета
export const DIFFICULTY_COLORS: Record<Difficulty, string> = {
  easy: 'bg-easy/15 text-easy',
  medium: 'bg-medium/15 text-medium',
  hard: 'bg-hard/15 text-hard',
}

// Только цвет текста — для таблицы задач, как в LeetCode
export const DIFFICULTY_TEXT: Record<Difficulty, string> = {
  easy: 'text-easy',
  medium: 'text-medium',
  hard: 'text-hard',
}

export const BOT_LEVEL_LABELS: Record<BotLevel, string> = {
  novice: 'Новичок',
  middle: 'Мидл',
  senior: 'Сеньор',
}

export const RESULT_LABELS: Record<MatchResult, string> = {
  win: 'Победа',
  loss: 'Поражение',
  draw: 'Ничья',
}

export const RESULT_COLORS: Record<MatchResult, string> = {
  win: 'text-ok',
  loss: 'text-bad',
  draw: 'text-muted',
}

export const VERDICT_LABELS: Record<Verdict, string> = {
  accepted: 'Все тесты пройдены',
  wrong_answer: 'Неверный ответ',
  runtime_error: 'Ошибка выполнения',
  time_limit: 'Превышено время',
}

export const LEAGUE_COLORS: Record<League, string> = {
  Bronze: 'bg-[#cd7f32]/15 text-[#8a5a3b] dark:text-[#e0a370]',
  Silver: 'bg-gray-400/20 text-gray-600 dark:text-gray-300',
  Gold: 'bg-yellow-400/25 text-yellow-800 dark:text-yellow-300',
  Platinum: 'bg-cyan-400/20 text-cyan-800 dark:text-cyan-300',
  Diamond: 'bg-indigo-400/20 text-indigo-700 dark:text-indigo-300',
}
