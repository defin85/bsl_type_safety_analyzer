#!/usr/bin/env node

/**
 * Wrapper для cargo команд с автоматическим определением CARGO_BUILD_JOBS
 * Использование: node scripts/cargo-wrapper.js [cargo-args]
 */

const { execSync } = require('child_process');
const os = require('os');

// Функция для получения количества процессоров
function getCpuCount() {
    const cpus = os.cpus().length;
    console.log(`🔧 Detected ${cpus} CPU cores`);
    return cpus;
}

// Получаем аргументы для cargo
const args = process.argv.slice(2);

// Устанавливаем CARGO_BUILD_JOBS
const cpuCount = getCpuCount();
process.env.CARGO_BUILD_JOBS = cpuCount;

// Проверяем, есть ли --jobs в аргументах, если нет - добавляем
const hasJobsFlag = args.some(arg => arg.startsWith('--jobs') || arg === '-j');
if (!hasJobsFlag && args.includes('build')) {
    // Добавляем флаг --jobs после build
    const buildIndex = args.indexOf('build');
    args.splice(buildIndex + 1, 0, '--jobs', String(cpuCount));
}

// Формируем команду
const command = `cargo ${args.join(' ')}`;
console.log(`🚀 Running: ${command}`);
console.log(`📊 CARGO_BUILD_JOBS=${cpuCount}`);

try {
    execSync(command, { 
        stdio: 'inherit',
        env: {
            ...process.env,
            CARGO_BUILD_JOBS: String(cpuCount)
        }
    });
} catch (error) {
    console.error('❌ Cargo command failed');
    process.exit(1);
}