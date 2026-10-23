<script setup lang="ts">
import type { RunResult } from '~/types/Run'
import type { TaskDetail } from '~/types/Task'

const route = useRoute()
const { apiUrl, apiFetch } = useApi()
const { isLoggedIn } = useAuth()

const { data: task, pending, error } = await useFetch<TaskDetail>(apiUrl(`/tasks/${route.params.id}`))

// ── Редактор и проверка ──
const code = ref('')
const result = ref<RunResult | null>(null)
const runError = ref<string | null>(null)
const isChecking = ref(false)

const isCss = computed(() => task.value?.mode === 'css')

const resetCode = () => {
  if (!task.value) return
  code.value = makeStarterCode(task.value.mode, task.value.function_name, task.value.params)
}

watch(task, resetCode, { immediate: true })

const check = async (kind: 'run' | 'submit') => {
  if (!task.value || isChecking.value || isCss.value) return

  isChecking.value = true
  runError.value = null
  try {
    result.value = await apiFetch<RunResult>(`/tasks/${task.value.id}/${kind}`, {
      method: 'POST',
      body: { code: code.value },
    })
  } catch (err) {
    runError.value = errorMessage(err, 'Сервер недоступен')
  } finally {
    isChecking.value = false
  }
}

// ── Вкладки левой панели ──
const leftTab = ref('description')
const leftPanelRef = ref<HTMLElement | null>(null)

// ── Сдаться: правильный ответ и разбор ──
const solution = ref<string | null>(null)
const giveUpError = ref<string | null>(null)

const leftTabs = computed(() => [
  { id: 'description', label: 'Описание' },
  { id: 'review', label: 'Ответ и разбор', disabled: solution.value === null },
])

const handleGiveUp = async () => {
  if (!task.value) return

  // Ответ закрыт для гостей: иначе его можно подсмотреть посреди матча в соседней вкладке
  if (!isLoggedIn.value) {
    await navigateTo({ path: '/login', query: { redirect: route.fullPath } })
    return
  }
  if (!confirm('Сдаться и посмотреть правильный ответ?')) return

  giveUpError.value = null
  try {
    const response = await apiFetch<{ solution: string }>(`/tasks/${task.value.id}/solution`)
    solution.value = response.solution
    stopClock()
    leftTab.value = 'review'
    // На телефоне панели идут друг под другом — прокручиваем к ответу
    leftPanelRef.value?.scrollIntoView({ behavior: 'smooth' })
  } catch (err) {
    giveUpError.value = errorMessage(err, 'Сервер недоступен')
  }
}

// ── Секундомер тренировки ──
const startedAt = ref(Date.now())
const now = ref(Date.now())
let clockTimer: ReturnType<typeof setInterval> | null = null

const elapsed = computed(() => formatClock(now.value - startedAt.value))

onMounted(() => {
  startedAt.value = Date.now()
  clockTimer = setInterval(() => { now.value = Date.now() }, 1000)
})

const stopClock = () => {
  if (clockTimer) clearInterval(clockTimer)
  clockTimer = null
}

onUnmounted(stopClock)

useHead({
  title: computed(() => (task.value ? `${task.value.title} — CodeClash` : 'CodeClash')),
})
</script>

<template>
  <div v-if="pending" class="text-center py-20">
    <p class="text-muted">Загружаем задачу...</p>
  </div>

  <div v-else-if="error || !task" class="text-center py-20">
    <p class="text-lg mb-2">Задача не найдена</p>
    <NuxtLink to="/" class="text-sm text-muted hover:text-ink transition-colors duration-200">
      Вернуться к списку →
    </NuxtLink>
  </div>

  <!-- Рабочее место, как в LeetCode: на широком экране — во всю высоту под шапкой -->
  <div v-else class="p-2 flex flex-col lg:flex-row gap-2 lg:h-[calc(100vh-48px)]">
    <!-- Слева: условие / ответ и разбор -->
    <section ref="leftPanelRef" class="lg:w-1/2 flex flex-col bg-panel border border-line rounded-card overflow-hidden min-h-[360px]">
      <PanelTabs v-model="leftTab" :tabs="leftTabs">
        <NuxtLink to="/" class="text-xs text-muted hover:text-ink transition-colors duration-200">← Все задачи</NuxtLink>
      </PanelTabs>

      <div class="flex-1 overflow-auto p-5">
        <!-- v-show, а не v-if: иначе при переключении вкладок разбор запрашивался бы заново -->
        <div v-show="leftTab === 'description'">
          <h1 class="text-2xl font-semibold mb-3">{{ task.id }}. {{ task.title }}</h1>
          <div class="flex flex-wrap items-center gap-2 mb-5">
            <span :class="['px-2.5 py-0.5 rounded-full text-xs font-medium', DIFFICULTY_COLORS[task.difficulty]]">
              {{ DIFFICULTY_LABELS[task.difficulty] }}
            </span>
            <span class="px-2.5 py-0.5 rounded-full bg-raised text-xs">{{ MODE_LABELS[task.mode] }}</span>
            <span v-if="task.source === 'llm'" class="px-2.5 py-0.5 rounded-full bg-accent/15 text-accent text-xs">
              Написала нейросеть
            </span>
          </div>

          <TaskStatement :task="task" />

          <NuxtLink
            v-if="!isCss"
            to="/play"
            class="inline-block mt-8 text-sm text-accent hover:underline"
          >
            Сразиться с ботом →
          </NuxtLink>
        </div>

        <div v-if="solution !== null" v-show="leftTab === 'review'">
          <SolutionReview :solution="solution" :code="code" :review-path="`/tasks/${task.id}/review`" />
        </div>
      </div>
    </section>

    <!-- Справа: редактор и результат -->
    <section class="lg:w-1/2 flex flex-col gap-2 min-h-0">
      <div class="flex flex-col bg-panel border border-line rounded-card overflow-hidden h-[480px] lg:h-auto lg:flex-1 min-h-0">
        <!-- Панель редактора: язык, секундомер, действия -->
        <div class="flex flex-wrap items-center gap-2 px-3 py-1.5 min-h-10 border-b border-line bg-raised/40 shrink-0">
          <span class="text-sm font-medium">{{ MODE_LABELS[task.mode] }}</span>
          <span class="font-mono text-sm text-muted tabular-nums" title="Время тренировки">{{ elapsed }}</span>

          <div class="ml-auto flex flex-wrap justify-end items-center gap-2">
            <button class="px-2 py-1 text-xs text-muted hover:text-ink transition-colors duration-200" @click="resetCode">
              Сбросить
            </button>
            <button
              v-if="solution === null"
              class="px-3 py-1.5 rounded-md border border-bad/50 text-bad text-sm font-medium hover:bg-bad/10 transition-colors duration-200"
              @click="handleGiveUp"
            >
              Сдаться
            </button>
            <template v-if="!isCss">
              <button
                :disabled="isChecking"
                class="px-3 py-1.5 rounded-md bg-raised text-sm font-medium hover:bg-line transition-colors duration-200 disabled:opacity-50"
                title="⌘/Ctrl + Enter"
                @click="check('run')"
              >
                Запустить
              </button>
              <button
                :disabled="isChecking"
                class="px-3 py-1.5 rounded-md bg-ok text-white text-sm font-medium hover:bg-ok/90 transition-colors duration-200 disabled:opacity-50"
                title="⌘/Ctrl + Shift + Enter"
                @click="check('submit')"
              >
                Отправить
              </button>
            </template>
          </div>
        </div>

        <p v-if="giveUpError" class="px-3 py-2 text-sm text-bad border-b border-line">{{ giveUpError }}</p>

        <div class="flex-1 min-h-0">
          <CodeEditor v-model="code" :mode="task.mode" @run="check('run')" @submit="check('submit')" />
        </div>
      </div>

      <!-- Алгоритмы: результат проверки -->
      <div v-if="!isCss" class="bg-panel border border-line rounded-card overflow-hidden h-[320px] lg:h-[38%] shrink-0">
        <RunResultPanel :result="result" :pending="isChecking" :error="runError" />
      </div>

      <!-- CSS: твоё превью и эталон рядом. Оба ровно 400×300 — как холст в условии, иначе их не сравнить -->
      <div v-else class="bg-panel border border-line rounded-card overflow-auto shrink-0 p-4">
        <div class="flex flex-wrap gap-4">
          <div>
            <h2 class="text-xs text-muted mb-2">Твой результат</h2>
            <!-- sandbox без allow-scripts: вёрстка видна, а скрипты из кода не выполняются -->
            <ClientOnly>
              <iframe :srcdoc="code" sandbox="" class="w-[402px] h-[302px] rounded-lg border border-line bg-white" title="Твой результат" />
            </ClientOnly>
          </div>
          <div>
            <h2 class="text-xs text-muted mb-2">Нужно получить</h2>
            <!-- ClientOnly — iframe из серверного HTML после гидратации оставался пустым -->
            <ClientOnly>
              <iframe :srcdoc="task.target_html ?? ''" sandbox="" class="w-[402px] h-[302px] rounded-lg border border-line bg-white" title="Эталонный макет" />
            </ClientOnly>
          </div>
        </div>
        <p class="text-sm text-muted mt-3">Автопроверка CSS появится позже — пока сравни на глаз.</p>
      </div>
    </section>
  </div>
</template>
