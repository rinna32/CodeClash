<script setup lang="ts">
import type { MatchView, SubmitResponse } from '~/types/Match'
import type { RunResult } from '~/types/Run'

// Часы, опрос сервера и редактор живут только в браузере.
// key — чтобы «Реванш» (переход на /match/другой-id) пересоздавал страницу с нуля
definePageMeta({
  middleware: 'auth',
  ssr: false,
  key: route => route.fullPath,
})

const POLL_INTERVAL = 1000 // мс — так часто спрашиваем сервер о прогрессе бота
const TIME_WARNING = 30_000 // мс — последние 30 секунд таймер краснеет
const BOT_CODE_LINES = [72, 48, 85, 60, 30, 78, 55, 40, 66] // ширины размытых строк «кода бота», %

const route = useRoute()
const { apiFetch } = useApi()

const match = ref<MatchView | null>(null)
const loadError = ref<string | null>(null)
const clockOffset = ref(0) // разница часов сервера и браузера, мс
const now = ref(Date.now())

const code = ref('')
const result = ref<RunResult | null>(null)
const runError = ref<string | null>(null)
const isChecking = ref(false)
const isResultOpen = ref(true) // окно итогов можно закрыть, чтобы посмотреть ответ и разбор
const leftTab = ref('description')

// ── Состояние матча ──

const serverNow = computed(() => now.value + clockOffset.value)

const phase = computed<'loading' | 'countdown' | 'active' | 'finished'>(() => {
  if (!match.value) return 'loading'
  if (match.value.status === 'finished') return 'finished'
  if (serverNow.value < match.value.starts_at) return 'countdown'
  return 'active'
})

const countdown = computed(() => (match.value ? Math.ceil((match.value.starts_at - serverNow.value) / 1000) : 0))
// После конца матча таймер замирает на моменте финиша, а не тикает дальше
const timeLeft = computed(() => {
  if (!match.value) return 0
  return match.value.ends_at - (match.value.finished_at ?? serverNow.value)
})
const isTimeRunningOut = computed(() => phase.value === 'active' && timeLeft.value < TIME_WARNING)

const playerShare = computed(() => (match.value ? (match.value.player_passed / match.value.total_tests) * 100 : 0))
const botShare = computed(() => (match.value ? (match.value.bot_passed / match.value.total_tests) * 100 : 0))

const hasSolution = computed(() => phase.value === 'finished' && match.value?.solution !== null)
const leftTabs = computed(() => [
  { id: 'description', label: 'Описание' },
  { id: 'review', label: 'Ответ и разбор', disabled: !hasSolution.value },
])

// ── Запросы ──

const applyMatch = (view: MatchView) => {
  match.value = view
  clockOffset.value = view.server_now - Date.now()

  if (!code.value) {
    code.value = makeStarterCode(view.mode, view.task.function_name, view.task.params)
  }
}

const loadMatch = async () => {
  try {
    applyMatch(await apiFetch<MatchView>(`/matches/${route.params.id}`))
  } catch (err) {
    if (isUnauthorized(err)) {
      await navigateTo({ path: '/login', query: { redirect: route.fullPath } })
      return
    }
    loadError.value = errorMessage(err, 'Сервер недоступен')
  }
}

const check = async (kind: 'run' | 'submit') => {
  if (!match.value || phase.value !== 'active' || isChecking.value) return

  isChecking.value = true
  runError.value = null
  try {
    if (kind === 'run') {
      result.value = await apiFetch<RunResult>(`/matches/${match.value.id}/run`, {
        method: 'POST',
        body: { code: code.value },
      })
    } else {
      const response = await apiFetch<SubmitResponse>(`/matches/${match.value.id}/submit`, {
        method: 'POST',
        body: { code: code.value },
      })
      result.value = response.run
      applyMatch(response.match)
    }
  } catch (err) {
    runError.value = errorMessage(err, 'Сервер недоступен')
    await loadMatch() // например, матч успел закончиться
  } finally {
    isChecking.value = false
  }
}

const handleResign = async () => {
  if (!match.value || !confirm('Сдаться? Это засчитается как поражение, зато сразу увидишь ответ и разбор.')) return

  applyMatch(await apiFetch<MatchView>(`/matches/${match.value.id}/resign`, { method: 'POST' }))
}

const showReview = () => {
  isResultOpen.value = false
  leftTab.value = 'review'
}

const handleRematch = async () => {
  if (!match.value) return

  try {
    const next = await apiFetch<MatchView>('/matches', {
      method: 'POST',
      body: { mode: match.value.mode, difficulty: match.value.difficulty, bot_level: match.value.bot_level },
    })
    await navigateTo(`/match/${next.id}`)
  } catch (err) {
    runError.value = errorMessage(err, 'Не получилось начать реванш')
  }
}

// ── Часы и опрос ──

let clockTimer: ReturnType<typeof setInterval> | null = null
let pollTimer: ReturnType<typeof setInterval> | null = null

onMounted(async () => {
  await loadMatch()

  clockTimer = setInterval(() => { now.value = Date.now() }, 250)
  pollTimer = setInterval(() => {
    if (phase.value !== 'finished') loadMatch()
  }, POLL_INTERVAL)
})

onUnmounted(() => {
  if (clockTimer) clearInterval(clockTimer)
  if (pollTimer) clearInterval(pollTimer)
})

useHead({ title: 'Матч — CodeClash' })
</script>

<template>
  <div v-if="loadError" class="text-center py-20">
    <p class="text-lg mb-2">{{ loadError }}</p>
    <NuxtLink to="/play" class="text-sm text-muted hover:text-ink transition-colors duration-200">Новый матч →</NuxtLink>
  </div>

  <div v-else-if="!match" class="text-center py-20">
    <p class="text-muted">Загружаем матч...</p>
  </div>

  <!-- Рабочее место матча: сверху таймер и счёт, ниже — как на странице задачи -->
  <div v-else class="p-2 flex flex-col gap-2 lg:h-[calc(100vh-48px)]">
    <!-- Табло: задача, таймер, прогресс -->
    <div class="bg-panel border border-line rounded-card px-4 py-3 grid grid-cols-1 md:grid-cols-3 gap-3 items-center shrink-0">
      <div class="flex items-center gap-2 min-w-0">
        <span :class="['shrink-0 px-2.5 py-0.5 rounded-full text-xs font-medium', DIFFICULTY_COLORS[match.difficulty]]">
          {{ DIFFICULTY_LABELS[match.difficulty] }}
        </span>
        <span class="shrink-0 px-2.5 py-0.5 rounded-full bg-raised text-xs">{{ MODE_LABELS[match.mode] }}</span>
        <h1 class="font-semibold truncate">{{ match.task.title }}</h1>
      </div>

      <p
        :class="[
          'text-center font-mono text-4xl tabular-nums transition-colors duration-300',
          isTimeRunningOut ? 'text-bad' : 'text-ink'
        ]"
      >
        {{ phase === 'countdown' ? formatClock(match.ends_at - match.starts_at) : formatClock(timeLeft) }}
      </p>

      <div class="space-y-1.5">
        <div class="flex items-center gap-3 text-sm">
          <span class="w-24 shrink-0 font-medium">Ты</span>
          <div class="flex-1 h-2 rounded-full bg-raised overflow-hidden">
            <div class="h-full bg-ok transition-all duration-500" :style="{ width: `${playerShare}%` }" />
          </div>
          <span class="w-10 text-right font-mono text-xs">{{ match.player_passed }}/{{ match.total_tests }}</span>
        </div>
        <div class="flex items-center gap-3 text-sm">
          <span class="w-24 shrink-0 text-muted truncate">Бот · {{ BOT_LEVEL_LABELS[match.bot_level] }}</span>
          <div class="flex-1 h-2 rounded-full bg-raised overflow-hidden">
            <div class="h-full bg-bad transition-all duration-500" :style="{ width: `${botShare}%` }" />
          </div>
          <span class="w-10 text-right font-mono text-xs text-muted">{{ match.bot_passed }}/{{ match.total_tests }}</span>
        </div>
      </div>
    </div>

    <div class="flex-1 min-h-0 flex flex-col lg:flex-row gap-2">
      <!-- Слева: условие и бот / ответ и разбор -->
      <section class="lg:w-1/2 flex flex-col bg-panel border border-line rounded-card overflow-hidden min-h-[360px]">
        <PanelTabs v-model="leftTab" :tabs="leftTabs" />

        <div class="flex-1 overflow-auto p-5">
          <!-- v-show, а не v-if: иначе при переключении вкладок разбор запрашивался бы заново -->
          <div v-show="leftTab === 'description'">
            <TaskStatement :task="match.task" />

            <!-- Код бота: виден только прогресс, сам код размыт -->
            <div class="mt-8 bg-bg border border-line rounded-lg p-4 relative overflow-hidden">
              <div class="space-y-2 blur-[3px] select-none" aria-hidden="true">
                <div
                  v-for="(width, index) in BOT_CODE_LINES"
                  :key="index"
                  :class="['h-2.5 rounded bg-raised', phase === 'active' ? 'animate-pulse' : '']"
                  :style="{ width: `${width}%` }"
                />
              </div>
              <p class="absolute bottom-2 right-3 text-xs text-muted">
                {{ phase === 'active' ? 'Бот пишет код…' : 'Код бота скрыт' }}
              </p>
            </div>
          </div>

          <!-- После конца — правильный ответ и разбор. Разбор запрашивается сразу, даже пока вкладка закрыта -->
          <div v-if="hasSolution && match.solution !== null" v-show="leftTab === 'review'">
            <SolutionReview :solution="match.solution" :code="code" :review-path="`/matches/${match.id}/review`" />
          </div>
        </div>
      </section>

      <!-- Справа: редактор и результат -->
      <section class="lg:w-1/2 flex flex-col gap-2 min-h-0">
        <div class="flex flex-col bg-panel border border-line rounded-card overflow-hidden h-[480px] lg:h-auto lg:flex-1 min-h-0">
          <div class="flex flex-wrap items-center gap-2 px-3 py-1.5 min-h-10 border-b border-line bg-raised/40 shrink-0">
            <span class="text-sm font-medium">{{ MODE_LABELS[match.mode] }}</span>

            <div class="ml-auto flex flex-wrap justify-end items-center gap-2">
              <button
                v-if="phase === 'active'"
                class="px-3 py-1.5 rounded-md border border-bad/50 text-bad text-sm font-medium hover:bg-bad/10 transition-colors duration-200"
                @click="handleResign"
              >
                Сдаться
              </button>
              <button
                v-else-if="phase === 'finished'"
                class="px-3 py-1.5 rounded-md bg-accent text-white text-sm font-medium hover:brightness-110 transition"
                @click="handleRematch"
              >
                Реванш
              </button>
              <button
                :disabled="phase !== 'active' || isChecking"
                class="px-3 py-1.5 rounded-md bg-raised text-sm font-medium hover:bg-line transition-colors duration-200 disabled:opacity-40"
                title="⌘/Ctrl + Enter"
                @click="check('run')"
              >
                Запустить
              </button>
              <button
                :disabled="phase !== 'active' || isChecking"
                class="px-3 py-1.5 rounded-md bg-ok text-white text-sm font-medium hover:bg-ok/90 transition-colors duration-200 disabled:opacity-40"
                title="⌘/Ctrl + Shift + Enter"
                @click="check('submit')"
              >
                Отправить
              </button>
            </div>
          </div>

          <div class="flex-1 min-h-0">
            <CodeEditor v-model="code" :mode="match.mode" @run="check('run')" @submit="check('submit')" />
          </div>
        </div>

        <div class="bg-panel border border-line rounded-card overflow-hidden h-[320px] lg:h-[38%] shrink-0">
          <RunResultPanel :result="result" :pending="isChecking" :error="runError" />
        </div>
      </section>
    </div>

    <!-- Обратный отсчёт -->
    <div v-if="phase === 'countdown'" class="fixed inset-0 z-40 flex items-center justify-center bg-black/60">
      <div class="bg-panel border border-line rounded-card px-16 py-12 text-center shadow-[0_20px_60px_rgba(0,0,0,0.5)]">
        <p class="text-muted mb-2">Матч начнётся через</p>
        <p class="text-[96px] font-bold leading-none tabular-nums text-accent">{{ Math.max(countdown, 1) }}</p>
      </div>
    </div>

    <!-- Итоги -->
    <div v-if="phase === 'finished' && match.result && isResultOpen" class="fixed inset-0 z-40 flex items-center justify-center bg-black/60 px-6">
      <div class="bg-panel border border-line w-full max-w-[560px] rounded-card p-8 shadow-[0_20px_60px_rgba(0,0,0,0.5)] text-center">
        <p :class="['text-[clamp(40px,8vw,64px)] font-bold leading-[1.05] tracking-[-0.02em] mb-2', RESULT_COLORS[match.result]]">
          {{ RESULT_LABELS[match.result] }}
        </p>
        <p class="text-muted mb-6">
          Ты {{ match.player_passed }}/{{ match.total_tests }} · бот {{ match.bot_passed }}/{{ match.total_tests }}
        </p>

        <div class="flex items-center justify-center gap-3 mb-8">
          <span class="font-mono text-2xl">{{ match.rating_after }}</span>
          <span
            v-if="match.rating_delta !== null"
            :class="['font-mono text-lg', match.rating_delta >= 0 ? 'text-ok' : 'text-bad']"
          >
            {{ match.rating_delta >= 0 ? '+' : '' }}{{ match.rating_delta }}
          </span>
          <span v-if="match.league" :class="['px-3 py-1 rounded-full text-xs font-medium', LEAGUE_COLORS[match.league]]">
            {{ match.league }}
          </span>
        </div>

        <div class="flex flex-col sm:flex-row gap-3 justify-center">
          <button class="px-6 py-3 bg-accent text-white rounded-lg hover:brightness-110 transition text-sm font-medium" @click="showReview">
            Ответ и разбор
          </button>
          <button class="px-6 py-3 bg-raised rounded-lg text-sm font-medium hover:bg-line transition-colors duration-200" @click="handleRematch">
            Реванш
          </button>
          <NuxtLink to="/play" class="px-6 py-3 text-sm text-muted hover:text-ink transition-colors duration-200">
            Другой матч →
          </NuxtLink>
        </div>
      </div>
    </div>
  </div>
</template>
