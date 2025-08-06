#!/usr/bin/env node

const { spawn } = require('child_process');
const chokidar = require('chokidar');
const path = require('path');
const fs = require('fs');

console.log('👁️  BSL Analyzer Smart Watch Mode v2.0 - с умным кешированием!');
console.log('='.repeat(70));

// Проверяем наличие chokidar
try {
    require.resolve('chokidar');
} catch (e) {
    console.log('❌ Требуется установить chokidar:');
    console.log('   npm install --save-dev chokidar');
    process.exit(1);
}

let isBuilding = false;
let buildQueue = new Set();

// Функция для выполнения сборки с умным кешированием
function runBuild(component) {
    if (isBuilding) {
        buildQueue.add(component);
        return;
    }
    
    isBuilding = true;
    console.log(`\\n🔄 [${new Date().toLocaleTimeString()}] Умная пересборка ${component} с кешированием...`);
    
    // НОВОЕ: Все сборки идут через smart build с кешированием!
    const commands = {
        'rust': 'node scripts/build-smart.js dev --component=rust',     // Умная сборка Rust
        'extension': 'node scripts/build-smart.js dev --component=extension', // Умная сборка Extension  
        'smart': 'node scripts/build-smart.js dev'                     // Полная умная сборка
    };
    
    const command = commands[component] || commands.smart;
    
    const child = spawn(command, {
        shell: true,
        stdio: 'inherit'
    });
    
    child.on('close', (code) => {
        isBuilding = false;
        
        if (code === 0) {
            console.log(`✅ [${new Date().toLocaleTimeString()}] ${component} пересобран успешно`);
        } else {
            console.log(`❌ [${new Date().toLocaleTimeString()}] Ошибка сборки ${component}`);
        }
        
        // Обрабатываем очередь
        if (buildQueue.size > 0) {
            const next = Array.from(buildQueue)[0];
            buildQueue.clear();
            runBuild(next);
        }
    });
}

// Настройка watcher'ов с умным кешированием
console.log('📁 Отслеживаемые директории (с кеш-оптимизацией):');
console.log('   src/ - Rust исходники → умная сборка с кешом');
console.log('   Cargo.toml, Cargo.lock - Rust конфигурация → авто-определение изменений');
console.log('   vscode-extension/src/ - TypeScript исходники → инкрементальная компиляция');
console.log('   vscode-extension/package.json - Extension конфигурация → минимальная пересборка');

// Rust файлы
const rustWatcher = chokidar.watch([
    'src/**/*.rs',
    'Cargo.toml',
    'Cargo.lock'
], {
    ignored: /target|node_modules|\\.git/,
    persistent: true,
    ignoreInitial: true
});

rustWatcher.on('change', (filePath) => {
    console.log(`📝 Изменен Rust файл: ${path.relative('.', filePath)}`);
    runBuild('rust');
});

// TypeScript файлы
const tsWatcher = chokidar.watch([
    'vscode-extension/src/**/*.ts',
    'vscode-extension/package.json',
    'vscode-extension/tsconfig.json'
], {
    ignored: /node_modules|\\.git|out/,
    persistent: true,
    ignoreInitial: true
});

tsWatcher.on('change', (filePath) => {
    console.log(`📝 Изменен TypeScript файл: ${path.relative('.', filePath)}`);
    runBuild('extension');
});

// Обработка ошибок
rustWatcher.on('error', error => {
    console.error('❌ Ошибка Rust watcher:', error);
});

tsWatcher.on('error', error => {
    console.error('❌ Ошибка TypeScript watcher:', error);
});

console.log('\\n✅ Smart Watch режим запущен!');
console.log('🧠 Редактируйте файлы - умная пересборка с кешированием!');
console.log('🚀 Нет изменений = мгновенное завершение, есть изменения = собираем только нужное!');
console.log('🛑 Способы остановки: Ctrl+C или нажмите "q" + Enter');

// Дополнительная остановка по клавише 'q'
const readline = require('readline');
const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

// Обработка ввода
rl.on('line', (input) => {
    if (input.trim().toLowerCase() === 'q') {
        console.log('👋 Остановка по запросу пользователя...');
        cleanup();
    }
});

// Функция очистки ресурсов
function cleanup() {
    console.log('🛑 Остановка watch режима...');
    rl.close();
    rustWatcher.close();
    tsWatcher.close();
    process.exit(0);
}

// Graceful shutdown
process.on('SIGINT', () => {
    console.log('\\n🛑 Остановка по Ctrl+C...');
    cleanup();
});