<script setup lang="ts">
import type { Review } from '~/types/Review'

// Правильный ответ и разбор решения нейросетью — содержимое вкладки «Ответ и разбор».
// Разбор просим сразу при появлении: к этому моменту матч закончен или игрок сдался
const props = defineProps<{
  solution: string
  code: string // код игрока на момент конца — его и разбираем
  reviewPath: string // /matches/{id}/review или /tasks/{id}/review
}>()

const { apiFetch } = useApi()

const review = ref<Review | null>(null)
const reviewError = ref<string | null>(null)
const isLoading = ref(false)

// Код игрока запоминаем один раз: правки после конца не должны менять разбор
const reviewedCode = props.code

const loadReview = async () => {
  isLoading.value = true
  reviewError.value = null

  try {
    review.value = await apiFetch<Review>(props.reviewPath, {
      method: 'POST',
      body: { code: reviewedCode },
    })
  } catch (err) {
    reviewError.value = errorMessage(err, 'Сервер недоступен')
  } finally {
    isLoading.value = false
  }
}

onMounted(loadReview)
</script>

<template>
  <div class="space-y-8">
    <!-- Правильный ответ -->
    <section>
      <h2 class="text-lg font-semibold mb-3">Правильный ответ</h2>
      <pre
        v-if="props.solution"
        class="bg-bg border border-line rounded-lg p-4 overflow-x-auto font-mono text-sm leading-relaxed"
      ><code>{{ props.solution }}</code></pre>
      <p v-else class="text-muted">У этой задачи нет эталона.</p>
    </section>

    <!-- Разбор от нейросети -->
    <section>
      <h2 class="text-lg font-semibold mb-3">Разбор твоего решения</h2>

      <div v-if="isLoading" class="flex items-center gap-3 text-muted">
        <span class="w-2 h-2 rounded-full bg-accent animate-pulse" />
        Нейросеть разбирает твой код… (до минуты)
      </div>

      <div v-else-if="reviewError">
        <p class="text-bad font-medium mb-1">Разбор не получился</p>
        <p class="text-sm text-muted mb-4">{{ reviewError }}</p>
        <button
          class="px-4 py-2 bg-raised rounded-lg text-sm font-medium hover:bg-line transition-colors duration-200"
          @click="loadReview"
        >
          Попробовать ещё раз
        </button>
      </div>

      <!-- v-html безопасен: formatStatement экранирует HTML и подсвечивает только `код` -->
      <div v-else-if="review" class="space-y-5 leading-relaxed">
        <p class="text-ink/90" v-html="formatStatement(review.summary)" />

        <div class="rounded-lg bg-accent/10 border border-accent/30 p-4">
          <h3 class="font-medium text-accent mb-1">Идея решения</h3>
          <p class="text-ink/90" v-html="formatStatement(review.approach)" />
        </div>

        <div v-if="review.mistakes.length">
          <h3 class="font-medium text-bad mb-2">Ошибки</h3>
          <ul class="list-disc pl-5 space-y-1.5 text-ink/90 marker:text-bad">
            <li v-for="(mistake, index) in review.mistakes" :key="index" v-html="formatStatement(mistake)" />
          </ul>
        </div>

        <div v-if="review.tips.length">
          <h3 class="font-medium text-ok mb-2">Как улучшить</h3>
          <ul class="list-disc pl-5 space-y-1.5 text-ink/90 marker:text-ok">
            <li v-for="(tip, index) in review.tips" :key="index" v-html="formatStatement(tip)" />
          </ul>
        </div>
      </div>
    </section>
  </div>
</template>
