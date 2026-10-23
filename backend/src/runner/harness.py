# ── Обвязка CodeClash: вызывает функцию игрока на тестах ──
# Код игрока вставлен выше. Импорты под псевдонимами, чтобы не задеть его переменные json/sys/time.
# Ожидаемых ответов здесь нет: сравнивает сервер, чтобы их нельзя было подсмотреть из кода.
import json as __cc_json
import sys as __cc_sys
import time as __cc_time


def __cc_main():
    data = __cc_json.loads(__cc_sys.stdin.read())
    fn = globals().get(data['function_name'])

    if not callable(fn):
        print(f"Функция {data['function_name']} не найдена", file=__cc_sys.stderr)
        __cc_sys.exit(1)

    results = []
    for args in data['tests']:
        start = __cc_time.perf_counter()
        try:
            got = fn(*args)
            results.append({'got': got, 'time_ms': (__cc_time.perf_counter() - start) * 1000})
        except Exception as err:
            results.append({
                'got': None,
                'error': f'{type(err).__name__}: {err}',
                'time_ms': (__cc_time.perf_counter() - start) * 1000,
            })

    # default=str — чтобы set или свой класс в ответе не уронили сериализацию
    print('\n__CODECLASH__' + __cc_json.dumps(results, default=str, ensure_ascii=False))


__cc_main()
