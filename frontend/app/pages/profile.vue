<script setup lang="ts">
import type { Me } from '~/types/Auth'
import type { MatchHistoryItem } from '~/types/Match'

definePageMeta({ middleware: 'auth' })

const { apiUrl, authHeaders } = useApi()
const { logout } = useAuth()

const { data: me, error: meError } = await useFetch<Me>(apiUrl('/me'), { headers: authHeaders })
const { data: history, pending } = await useFetch<MatchHistoryItem[]>(apiUrl('/me/matches'), {
  headers: authHeaders,
  default: () => [],
})

// Токен протух — выходим, middleware отправит на вход
if (meError.value?.statusCode === 401) logout()

// Доля побед для полоски в карточке рейтинга
const winRate = (games: number, wins: number) => (games ? Math.round((wins / games) * 100) : 0)

const formatDate = (ms: number | null) =>
  ms ? new Date(ms).toLocaleString('ru-RU', { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' }) : ''

useHead({ title: 'Профиль — CodeClash' })
</script>

<template>
  <div class="max-w-5xl mx-auto px-4 lg:px-6 py-8">
    <!-- Шапка профиля -->
    <div class="flex items-center gap-4 mb-8">
      <span class="w-16 h-16 rounded-full bg-raised flex items-center justify-center text-2xl font-bold uppercase">
        {{ me?.username.slice(0, 1) }}
      </span>
      <div>
        <h1 class="text-2xl font-semibold">{{ me?.username ?? 'Профиль' }}</h1>
        <p class="text-muted text-sm">Рейтинг и история матчей с ботом</p>
      </div>
    </div>

    <!-- Рейтинг по режимам -->
    <div v-if="me" class="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-10">
      <div v-for="rating in me.ratings" :key="rating.mode" class="bg-panel border border-line rounded-card p-5">
        <div class="flex items-center justify-between mb-3">
          <span class="text-sm text-muted">{{ MODE_LABELS[rating.mode] }}</span>
          <span :class="['px-2.5 py-0.5 rounded-full text-xs font-semibold', LEAGUE_COLORS[rating.league]]">{{ rating.league }}</span>
        </div>
        <p class="font-mono text-4xl tabular-nums mb-3">{{ rating.rating }}</p>
        <div class="h-1.5 rounded-full bg-raised overflow-hidden mb-2">
          <div class="h-full bg-ok" :style="{ width: `${winRate(rating.games, rating.wins)}%` }" />
        </div>
        <p class="text-sm text-muted">
          Матчей: {{ rating.games }} · побед: {{ rating.wins }} · {{ winRate(rating.games, rating.wins) }}%
        </p>
      </div>
    </div>

    <!-- История матчей -->
    <h2 class="text-lg font-semibold mb-3">История матчей</h2>

    <div v-if="pending" class="bg-panel border border-line rounded-card text-center py-10">
      <p class="text-muted">Загружаем историю...</p>
    </div>

    <div v-else-if="!history.length" class="bg-panel border border-line rounded-card text-center py-12">
      <p class="text-muted mb-4">Ты ещё не играл</p>
      <NuxtLink to="/play" class="px-6 py-2.5 bg-accent text-white rounded-lg hover:brightness-110 transition text-sm font-medium">
        Первый матч →
      </NuxtLink>
    </div>

    <div v-else class="bg-panel border border-line rounded-card overflow-hidden">
      <div
        v-for="item in history"
        :key="item.id"
        class="flex flex-wrap items-center gap-x-4 gap-y-1 px-4 py-3 text-sm even:bg-raised/30"
      >
        <span v-if="item.result" :class="['w-24 font-medium', RESULT_COLORS[item.result]]">{{ RESULT_LABELS[item.result] }}</span>
        <span class="font-medium">{{ item.task_title }}</span>
        <span class="text-muted">
          {{ MODE_LABELS[item.mode] }} ·
          <span :class="DIFFICULTY_TEXT[item.difficulty]">{{ DIFFICULTY_LABELS[item.difficulty] }}</span>
          · бот {{ BOT_LEVEL_LABELS[item.bot_level] }}
        </span>
        <span class="ml-auto font-mono text-muted">{{ item.player_passed }}/{{ item.total_tests }}</span>
        <span
          v-if="item.rating_delta !== null"
          :class="['w-12 text-right font-mono', item.rating_delta >= 0 ? 'text-ok' : 'text-bad']"
        >
          {{ item.rating_delta >= 0 ? '+' : '' }}{{ item.rating_delta }}
        </span>
        <span class="w-32 text-right text-muted">{{ formatDate(item.finished_at) }}</span>
      </div>
    </div>
  </div>
</template>
