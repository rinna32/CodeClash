<script setup lang="ts">
import type { BotLevel, MatchView } from '~/types/Match'
import type { Difficulty, Mode } from '~/types/Task'

definePageMeta({ middleware: 'auth' })

const { apiFetch } = useApi()

const modes = [
  { id: 'javascript', label: 'JavaScript', hint: 'Алгоритмы' },
  { id: 'python', label: 'Python', hint: 'Алгоритмы' },
  { id: 'css', label: 'CSS', hint: 'Скоро' },
] as const

const difficulties = [
  { id: 'easy', label: 'Лёгкая', hint: '5 минут' },
  { id: 'medium', label: 'Средняя', hint: '8 минут' },
  { id: 'hard', label: 'Сложная', hint: '12 минут' },
] as const

const botLevels = [
  { id: 'novice', label: 'Новичок', hint: 'Рейтинг 900 · не торопится, иногда застревает' },
  { id: 'middle', label: 'Мидл', hint: 'Рейтинг 1200 · решает к середине матча' },
  { id: 'senior', label: 'Сеньор', hint: 'Рейтинг 1500 · решает в первые минуты' },
] as const

const selectedMode = ref<Mode>('javascript')
const selectedDifficulty = ref<Difficulty>('easy')
const selectedBot = ref<BotLevel>('middle')
const isStarting = ref(false)
const startError = ref<string | null>(null)

const handleStart = async () => {
  isStarting.value = true
  startError.value = null

  try {
    const match = await apiFetch<MatchView>('/matches', {
      method: 'POST',
      body: { mode: selectedMode.value, difficulty: selectedDifficulty.value, bot_level: selectedBot.value },
    })
    await navigateTo(`/match/${match.id}`)
  } catch (err) {
    if (isUnauthorized(err)) {
      await navigateTo({ path: '/login', query: { redirect: '/play' } })
      return
    }
    startError.value = errorMessage(err, 'Сервер недоступен — запущен ли бэкенд?')
  } finally {
    isStarting.value = false
  }
}
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 lg:px-6 py-8">
    <h1 class="text-2xl font-semibold mb-1">Матч с ботом</h1>
    <p class="text-muted mb-8">Нейросеть уже решила задачу. Успеешь быстрее?</p>

    <!-- Шаг 1: режим -->
    <h2 class="text-xs font-medium tracking-[0.08em] uppercase text-muted mb-3">Режим</h2>
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 mb-8">
      <button
        v-for="mode in modes"
        :key="mode.id"
        :disabled="mode.id === 'css'"
        :class="[
          'text-left rounded-card p-4 border transition-colors duration-200 disabled:opacity-40 disabled:cursor-not-allowed',
          selectedMode === mode.id ? 'bg-accent/10 border-accent' : 'bg-panel border-line hover:border-muted'
        ]"
        @click="selectedMode = mode.id"
      >
        <p class="font-medium">{{ mode.label }}</p>
        <p class="text-sm text-muted">{{ mode.hint }}</p>
      </button>
    </div>

    <!-- Шаг 2: сложность -->
    <h2 class="text-xs font-medium tracking-[0.08em] uppercase text-muted mb-3">Сложность</h2>
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 mb-8">
      <button
        v-for="difficulty in difficulties"
        :key="difficulty.id"
        :class="[
          'text-left rounded-card p-4 border transition-colors duration-200',
          selectedDifficulty === difficulty.id ? 'bg-accent/10 border-accent' : 'bg-panel border-line hover:border-muted'
        ]"
        @click="selectedDifficulty = difficulty.id"
      >
        <p :class="['font-medium', DIFFICULTY_TEXT[difficulty.id]]">{{ difficulty.label }}</p>
        <p class="text-sm text-muted">{{ difficulty.hint }}</p>
      </button>
    </div>

    <!-- Шаг 3: соперник -->
    <h2 class="text-xs font-medium tracking-[0.08em] uppercase text-muted mb-3">Соперник</h2>
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 mb-10">
      <button
        v-for="bot in botLevels"
        :key="bot.id"
        :class="[
          'text-left rounded-card p-4 border transition-colors duration-200',
          selectedBot === bot.id ? 'bg-accent/10 border-accent' : 'bg-panel border-line hover:border-muted'
        ]"
        @click="selectedBot = bot.id"
      >
        <p class="font-medium">{{ bot.label }}</p>
        <p class="text-sm text-muted">{{ bot.hint }}</p>
      </button>
    </div>

    <p v-if="startError" class="text-bad mb-4">
      {{ startError }}
      <NuxtLink to="/" class="underline ml-1">К задачам →</NuxtLink>
    </p>

    <button
      :disabled="isStarting"
      class="w-full sm:w-auto px-10 py-3 bg-accent text-white rounded-lg hover:brightness-110 transition text-base font-semibold disabled:opacity-50"
      @click="handleStart"
    >
      {{ isStarting ? 'Подбираем задачу...' : 'Поехали →' }}
    </button>
  </div>
</template>
