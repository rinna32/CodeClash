export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  modules: ['@nuxtjs/tailwindcss'],
  css: ['~/assets/css/main.css'],

  // Свой main.css вместо стандартного файла модуля — в нём и директивы Tailwind, и базовые стили
  tailwindcss: {
    cssPath: '~/assets/css/main.css',
  },

  // Модуль Tailwind по умолчанию включает tailwindcss/nesting, а Nuxt 4 не может загрузить его как ESM
  // и пишет предупреждение при каждом старте. Вложенные CSS-правила нам не нужны — выключаем
  postcss: {
    plugins: {
      'tailwindcss/nesting': false,
    },
  },

  // Адрес бэкенда в одном месте. 127.0.0.1, а не localhost: Node может уйти на IPv6 (::1),
  // а сервер на Rust слушает только IPv4
  runtimeConfig: {
    public: {
      apiBase: 'http://127.0.0.1:8000/api',
    },
  },

  // Редактор грузится только на страницах задачи и матча. Без этого Vite в dev-режиме находит
  // CodeMirror при первом заходе, пересобирает зависимости и роняет переход ошибкой 504
  vite: {
    optimizeDeps: {
      include: [
        'codemirror',
        '@codemirror/state',
        '@codemirror/view',
        '@codemirror/commands',
        '@codemirror/lang-javascript',
        '@codemirror/lang-python',
        '@codemirror/lang-html',
        '@codemirror/theme-one-dark',
      ],
    },
  },

  app: {
    head: {
      title: 'CodeClash',
      htmlAttrs: { lang: 'ru' },
      // Шрифты с кириллицей: Russo One — заголовки, Jura — текст
      link: [
        { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
        { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' },
        { rel: 'stylesheet', href: 'https://fonts.googleapis.com/css2?family=Jura:wght@400;500;600;700&family=Russo+One&display=swap' },
      ],
    },
  },
})
