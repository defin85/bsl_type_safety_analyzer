#!/bin/bash

# Универсальная функция для определения количества процессоров
get_cpu_count() {
    if command -v nproc >/dev/null 2>&1; then
        # Linux/Unix
        nproc
    elif [[ -n "$NUMBER_OF_PROCESSORS" ]]; then
        # Windows
        echo $NUMBER_OF_PROCESSORS
    elif command -v sysctl >/dev/null 2>&1; then
        # macOS
        sysctl -n hw.ncpu
    else
        # Fallback
        echo 4
    fi
}

# Устанавливаем CARGO_BUILD_JOBS автоматически
export CARGO_BUILD_JOBS=$(get_cpu_count)
echo "Set CARGO_BUILD_JOBS=$CARGO_BUILD_JOBS"

# Функция для удобного запуска cargo с правильными настройками
cargo_with_jobs() {
    CARGO_BUILD_JOBS=$(get_cpu_count) "$@"
}

# Экспортируем функцию для использования в других скриптах
export -f cargo_with_jobs
export -f get_cpu_count