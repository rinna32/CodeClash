<script setup lang="ts">
import type { RunResult } from '~/types/Run'

// Нижняя панель рабочего места: вердикт, тесты и вывод игрока — как «Test Result» в LeetCode
const props = defineProps<{
  result: RunResult | null
  pending: boolean
  error: string | null
}>()

const isAccepted = computed(() => props.result?.verdict === 'accepted')

const formatValue = (value: unknown) => JSON.stringify(value)
const formatArgs = (args: unknown[] | null) => (args ?? []).map(formatValue).join(', ')
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="flex items-center px-4 h-10 border-b border-line bg-raised/40 shrink-0">
      <span class="text-sm font-medium">Результат</span>
    </div>

    <div class="flex-1 overflow-auto p-4">
      <div v-if="props.pending" class="py-8 text-center">
        <p class="text-muted">Проверяем решение...</p>
      </div>

      <div v-else-if="props.error" class="py-2">
        <p class="text-bad font-medium mb-1">Не получилось проверить</p>
        <p class="text-sm text-muted">{{ props.error }}</p>
      </div>

      <div v-else-if="!props.result" class="py-8 text-center">
        <p class="text-muted mb-1">Здесь появится результат</p>
        <p class="text-sm text-muted/70">«Запустить» — открытые тесты, «Отправить» — все</p>
      </div>

      <template v-else>
        <!-- Вердикт -->
        <div class="flex items-baseline gap-3 mb-4">
          <span :class="['text-xl font-semibold', isAccepted ? 'text-ok' : 'text-bad']">
            {{ VERDICT_LABELS[props.result.verdict] }}
          </span>
          <span class="text-sm text-muted">тестов пройдено: {{ props.result.passed }} / {{ props.result.total }}</span>
        </div>

        <!-- Ошибка всего запуска: синтаксис, таймаут -->
        <pre
          v-if="props.result.error"
          class="text-sm bg-bad/10 rounded-lg p-3 mb-4 whitespace-pre-wrap font-mono text-bad"
        >{{ props.result.error }}</pre>

        <!-- Тесты -->
        <div class="space-y-2 mb-4">
          <div
            v-for="test in props.result.results"
            :key="test.index"
            class="rounded-lg bg-raised/50 px-3 py-2.5 text-sm"
          >
            <div class="flex items-center gap-2">
              <span :class="test.passed ? 'text-ok' : 'text-bad'">{{ test.passed ? '✓' : '✕' }}</span>
              <span class="font-medium">Тест {{ test.index + 1 }}</span>
              <span v-if="test.is_hidden" class="text-muted">· скрытый</span>
              <span class="ml-auto text-muted font-mono text-xs">{{ test.time_ms.toFixed(2) }} мс</span>
            </div>

            <div v-if="!test.is_hidden" class="mt-2 font-mono text-xs space-y-1">
              <p><span class="text-muted">Ввод:</span> {{ formatArgs(test.args) }}</p>
              <p><span class="text-muted">Ожидали:</span> {{ formatValue(test.expected) }}</p>
              <p v-if="!test.passed"><span class="text-muted">Получили:</span> {{ formatValue(test.got) }}</p>
              <p v-if="test.error" class="text-bad">{{ test.error }}</p>
            </div>
          </div>
        </div>

        <!-- Вывод игрока: console.log / print -->
        <div v-if="props.result.stdout">
          <p class="text-xs text-muted mb-1">Вывод</p>
          <pre class="text-sm bg-bg rounded-lg p-3 whitespace-pre-wrap font-mono">{{ props.result.stdout }}</pre>
        </div>
      </template>
    </div>
  </div>
</template>
