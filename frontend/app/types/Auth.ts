// Интерфейсы под ответы бэкенда (backend/src/auth, backend/src/profile)
import type { Mode } from '~/types/Task'

export type League = 'Bronze' | 'Silver' | 'Gold' | 'Platinum' | 'Diamond'

export interface User {
  id: number
  username: string
}

export interface AuthResponse {
  token: string
  user: User
}

export interface ModeRating {
  mode: Mode
  rating: number
  games: number
  wins: number
  league: League
}

export interface Me extends User {
  ratings: ModeRating[]
}

export interface LeaderboardItem {
  place: number
  username: string
  rating: number
  games: number
  wins: number
  league: League
}
