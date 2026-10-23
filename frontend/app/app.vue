<script setup lang="ts">
const { theme } = useTheme()

// Тема — атрибутом на <html>: от него зависят CSS-переменные в main.css
useHead({
  htmlAttrs: { 'data-theme': theme },
})
</script>

<template>
  <div class="min-h-screen flex flex-col">
    <AppHeader />

    <main class="flex-1">
      <NuxtPage />
    </main>
  </div>
</template>
