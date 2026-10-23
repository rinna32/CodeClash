// ── Обвязка CodeClash: вызывает функцию игрока на тестах ──
// Код игрока вставлен выше, поэтому его функцию достаём через eval по имени.
// Ожидаемых ответов здесь нет: сравнивает сервер, чтобы их нельзя было подсмотреть из кода.
;(() => {
  const input = JSON.parse(require('fs').readFileSync(0, 'utf8'))

  let fn
  try {
    fn = eval(input.function_name)
  } catch {
    fn = undefined
  }

  if (typeof fn !== 'function') {
    console.error(`Функция ${input.function_name} не найдена`)
    process.exit(1)
  }

  const results = input.tests.map((args) => {
    const start = performance.now()
    try {
      const got = fn(...args)
      return { got: got === undefined ? null : got, time_ms: performance.now() - start }
    } catch (err) {
      return { got: null, error: String(err), time_ms: performance.now() - start }
    }
  })

  // Перевод строки перед маркером — на случай, если игрок вывел что-то без \n
  process.stdout.write('\n__CODECLASH__' + JSON.stringify(results) + '\n')
})()
