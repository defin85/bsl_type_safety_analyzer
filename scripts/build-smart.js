#!/usr/bin/env node

const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');
const os = require('os');

console.log('🧠 Smart Build System v2.0');
console.log('='.repeat(50));

// Автоопределение количества CPU
const CPU_COUNT = os.cpus().length;
console.log(`🖥️ Обнаружено процессоров: ${CPU_COUNT}`);

// Конфигурация
const CACHE_DIR = '.build-cache';
const RUST_SRC_DIRS = ['src', 'Cargo.toml', 'Cargo.lock'];
const TS_SRC_DIRS = ['vscode-extension/src', 'vscode-extension/package.json', 'vscode-extension/tsconfig.json'];

// Создание директории кеша
if (!fs.existsSync(CACHE_DIR)) {
    fs.mkdirSync(CACHE_DIR, { recursive: true });
}

// Функция для получения хеша файлов
function getDirectoryHash(directories) {
    const hash = crypto.createHash('md5');
    
    for (const dir of directories) {
        if (fs.existsSync(dir)) {
            if (fs.statSync(dir).isFile()) {
                const content = fs.readFileSync(dir);
                hash.update(content);
            } else {
                // Рекурсивный обход директории
                const files = getAllFiles(dir);
                for (const file of files) {
                    const stat = fs.statSync(file);
                    hash.update(`${file}:${stat.mtime.toISOString()}`);
                }
            }
        }
    }
    
    return hash.digest('hex');
}

function getAllFiles(dir, fileList = []) {
    const files = fs.readdirSync(dir);
    
    for (const file of files) {
        const filePath = path.join(dir, file);
        const stat = fs.statSync(filePath);
        
        if (stat.isDirectory()) {
            // Игнорируем некоторые директории
            if (!['target', 'node_modules', '.git', '.build-cache'].includes(file)) {
                getAllFiles(filePath, fileList);
            }
        } else {
            fileList.push(filePath);
        }
    }
    
    return fileList;
}

// Проверка необходимости пересборки
function needsRebuild(component, srcDirs) {
    const cacheFile = path.join(CACHE_DIR, `${component}.hash`);
    const currentHash = getDirectoryHash(srcDirs);
    
    if (!fs.existsSync(cacheFile)) {
        console.log(`📝 ${component}: Первичная сборка`);
        return { rebuild: true, hash: currentHash };
    }
    
    const cachedHash = fs.readFileSync(cacheFile, 'utf8');
    const rebuild = cachedHash !== currentHash;
    
    if (rebuild) {
        console.log(`🔄 ${component}: Исходники изменились`);
    } else {
        console.log(`✅ ${component}: Кеш актуален`);
    }
    
    return { rebuild, hash: currentHash };
}

// Сохранение хеша
function saveHash(component, hash) {
    const cacheFile = path.join(CACHE_DIR, `${component}.hash`);
    fs.writeFileSync(cacheFile, hash);
}

// Выполнение команды с выводом времени
function runCommand(name, command, options = {}) {
    console.log(`\\n🚀 ${name}...`);
    const startTime = Date.now();
    
    try {
        execSync(command, { 
            stdio: 'inherit', 
            cwd: process.cwd(),
            shell: true,
            ...options
        });
        const duration = ((Date.now() - startTime) / 1000).toFixed(1);
        console.log(`✅ ${name} завершено за ${duration}s`);
        return true;
    } catch (error) {
        console.error(`❌ ${name} не удалось:`, error.message);
        return false;
    }
}

// Основная логика
async function smartBuild() {
    const buildMode = process.argv[2] || 'fast'; // fast, dev, release
    
    console.log(`📊 Режим сборки: ${buildMode}`);
    console.log(`⏰ Начало: ${new Date().toLocaleTimeString()}`);
    
    let totalTime = Date.now();
    let operations = 0;
    
    // 1. Проверяем Rust код
    const rustCheck = needsRebuild('rust', RUST_SRC_DIRS);
    if (rustCheck.rebuild) {
        const rustCommand = {
            'dev': `cargo build --jobs ${CPU_COUNT}`,
            'fast': `cargo build --profile dev-fast --jobs ${CPU_COUNT}`,
            'release': `cargo build --profile dev-fast --jobs ${CPU_COUNT}`
        }[buildMode] || `cargo build --profile dev-fast --jobs ${CPU_COUNT}`;
        
        if (runCommand('Rust сборка', rustCommand)) {
            saveHash('rust', rustCheck.hash);
            operations++;
        } else {
            process.exit(1);
        }
    }
    
    // 2. Копируем бинарники (если Rust пересобирался или бинарники отсутствуют)
    const binDir = 'vscode-extension/bin';
    const needsBinariesCopy = rustCheck.rebuild || !fs.existsSync(path.join(binDir, 'bsl-analyzer.exe'));
    
    if (needsBinariesCopy) {
        const profile = {
            'dev': 'debug',
            'fast': 'dev-fast',
            'release': 'dev-fast'
        }[buildMode] || 'dev-fast';
        const copyCmd = `node scripts/copy-essential-binaries.js ${profile}`;
            
        if (runCommand('Копирование основных бинарников', copyCmd)) {
            operations++;
        }
    } else {
        console.log('✅ Бинарники: Копирование не требуется');
    }
    
    // 3. Проверяем TypeScript код
    const tsCheck = needsRebuild('typescript', TS_SRC_DIRS);
    if (tsCheck.rebuild) {
        if (runCommand('TypeScript сборка', 'cd vscode-extension && npm run compile')) {
            saveHash('typescript', tsCheck.hash);
            operations++;
        } else {
            process.exit(1);
        }
    }
    
    // 4. Пакетируем расширение (только если что-то изменилось)
    if (rustCheck.rebuild || tsCheck.rebuild) {
        if (runCommand('Пакетирование VSCode расширения', 'cd vscode-extension && npx @vscode/vsce package')) {
            operations++;
        }
    } else {
        console.log('✅ Пакетирование: Не требуется');
    }
    
    // Статистика
    const totalDuration = ((Date.now() - totalTime) / 1000).toFixed(1);
    
    console.log('\\n' + '='.repeat(50));
    console.log('🎉 УМНАЯ СБОРКА ЗАВЕРШЕНА');
    console.log('='.repeat(50));
    console.log(`⏱️  Общее время: ${totalDuration}s`);
    console.log(`🔧 Выполнено операций: ${operations}/4`);
    console.log(`💾 Экономия от кеширования: ${4 - operations} операций`);
    
    if (operations === 0) {
        console.log('🚀 Все компоненты актуальны - сборка не требовалась!');
    }
}

// Парсинг аргументов для компонентов
const args = process.argv.slice(2);
const componentArg = args.find(arg => arg.startsWith('--component='));
const targetComponent = componentArg ? componentArg.split('=')[1] : null;

// Умная сборка конкретного компонента
async function smartComponentBuild(component) {
    console.log(`🎯 Умная сборка компонента: ${component}`);
    console.log('='.repeat(50));
    
    const totalTime = Date.now();
    let operations = 0;
    
    switch(component) {
        case 'rust':
            const rustCheck = needsRebuild('rust', RUST_SRC_DIRS);
            if (rustCheck.rebuild) {
                const profile = getProfile();
                const rustCmd = `cargo build --profile ${profile}`;
                if (runCommand('Rust сборка', rustCmd)) {
                    saveHash('rust', rustCheck.hash);
                    operations++;
                }
            } else {
                console.log('✅ Rust: Сборка не требуется');
            }
            break;
            
        case 'extension':
            const tsCheck = needsRebuild('typescript', TS_SRC_DIRS);
            if (tsCheck.rebuild) {
                if (runCommand('TypeScript сборка', 'cd vscode-extension && npm run compile')) {
                    saveHash('typescript', tsCheck.hash);
                    operations++;
                }
            } else {
                console.log('✅ TypeScript: Сборка не требуется');
            }
            break;
            
        default:
            console.log(`❌ Неизвестный компонент: ${component}`);
            return;
    }
    
    const duration = ((Date.now() - totalTime) / 1000).toFixed(1);
    console.log(`\n🎆 Компонент ${component} обработан за ${duration}s`);
    
    if (operations === 0) {
        console.log('🚀 Компонент актуален - сборка не требовалась!');
    }
}

// Запуск
if (targetComponent) {
    smartComponentBuild(targetComponent).catch(console.error);
} else {
    smartBuild().catch(console.error);
}