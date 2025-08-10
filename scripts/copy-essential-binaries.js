#!/usr/bin/env node

/**
 * Скрипт для копирования только основных бинарников в VSCode расширение
 * Исключает тестовые и вспомогательные файлы для уменьшения размера .vsix пакета
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('📦 BSL Analyzer - Essential Binaries Copy Tool');
console.log('='.repeat(60));

// Определяем профиль сборки из аргументов
const buildProfile = process.argv[2] || 'dev-fast';
console.log(`🔧 Используется профиль: ${buildProfile}`);

// Список основных бинарников для VSCode расширения (без тестовых и debug инструментов)
const ESSENTIAL_BINARIES = [
    // Основные инструменты анализа
    'bsl-analyzer.exe',           // Главный анализатор
    'lsp_server.exe',             // LSP сервер для интеграции с редактором
    'syntaxcheck.exe',            // Синтаксический анализатор

    // Система типов
    'build_unified_index.exe',    // Построение индекса типов
    'query_type.exe',             // Поиск типов
    'check_type_compatibility.exe', // Проверка совместимости типов

    // Работа с платформой и конфигурацией
    'extract_platform_docs.exe',  // Извлечение документации платформы
    'extract_hybrid_docs.exe',    // Извлечение гибридной документации
    'incremental_update.exe',     // Инкрементальные обновления

    // MCP интеграция для LLM
    'bsl-mcp-server.exe'          // MCP сервер для интеграции с Claude/GPT
];

// Определяем пути
const sourceDir = `target/${buildProfile}`;
const targetDir = 'vscode-extension/bin';

// Проверяем существование исходной директории (без авто-сборок или fallback)
if (!fs.existsSync(sourceDir)) {
    console.error(`❌ Директория сборки не найдена: ${sourceDir}`);
    console.log('💡 Сначала выполните сборку явно: npm run build:rust' +
        (buildProfile === 'release' ? ':release' : buildProfile === 'dev-fast' ? '' : ':dev'));
    process.exit(1);
}

// Создаем целевую директорию если не существует
if (!fs.existsSync(targetDir)) {
    fs.mkdirSync(targetDir, { recursive: true });
    console.log(`📁 Создана директория: ${targetDir}`);
}

// Очищаем старые файлы
console.log('🧹 Очистка старых бинарников...');
try {
    const oldFiles = fs.readdirSync(targetDir)
        .filter(file => file.endsWith('.exe') || file.endsWith('.pdb'));

    for (const file of oldFiles) {
        fs.unlinkSync(path.join(targetDir, file));
    }
    console.log(`   Удалено ${oldFiles.length} старых файлов`);
} catch (error) {
    console.log('   (Старые файлы не найдены)');
}

// Копируем только основные бинарники
console.log('📋 Копирование основных бинарников:');
let copiedCount = 0;
let totalSize = 0;

for (const binary of ESSENTIAL_BINARIES) {
    const sourcePath = path.join(sourceDir, binary);
    const targetPath = path.join(targetDir, binary);

    if (fs.existsSync(sourcePath)) {
        try {
            fs.copyFileSync(sourcePath, targetPath);
            const stats = fs.statSync(sourcePath);
            const sizeMB = (stats.size / (1024 * 1024)).toFixed(1);
            totalSize += stats.size;

            console.log(`   ✅ ${binary} (${sizeMB} MB)`);
            copiedCount++;
        } catch (error) {
            console.log(`   ❌ Ошибка копирования ${binary}: ${error.message}`);
        }
    } else {
        console.log(`   ⚠️  ${binary} - не найден`);
    }
}

// Копируем README для документации
const readmePath = path.join(targetDir, 'README.md');
if (!fs.existsSync(readmePath)) {
    const readmeContent = `# BSL Analyzer Binaries

This directory contains essential binaries for BSL Analyzer VSCode extension.

## Included tools:

- **bsl-analyzer.exe** - Main static analyzer
- **lsp_server.exe** - Language Server Protocol implementation  
- **syntaxcheck.exe** - Syntax validator
- **build_unified_index.exe** - Type system index builder
- **query_type.exe** - Type information queries
- **check_type_compatibility.exe** - Type compatibility checker
- **extract_platform_docs.exe** - Platform documentation extractor
- **extract_hybrid_docs.exe** - Hybrid documentation processor
- **incremental_update.exe** - Incremental analysis updates
- **bsl-mcp-server.exe** - MCP server for LLM integration

Total size optimized from 155+ MB to ~${(totalSize / (1024 * 1024)).toFixed(1)} MB by excluding test and debug binaries.
`;

    fs.writeFileSync(readmePath, readmeContent);
    console.log('   📝 README.md создан');
}

// Итоговая статистика
const finalSizeMB = (totalSize / (1024 * 1024)).toFixed(1);
console.log('='.repeat(60));
console.log(`🎉 ГОТОВО: Скопировано ${copiedCount}/${ESSENTIAL_BINARIES.length} бинарников`);
console.log(`📊 Размер: ${finalSizeMB} MB (вместо 155+ MB)`);
console.log(`🚀 Экономия: ~${(155 - finalSizeMB).toFixed(1)} MB`);
console.log(`📁 Расположение: ${targetDir}`);
console.log('');

// Проверяем критичные инструменты
const criticalMissing = ESSENTIAL_BINARIES.filter(binary =>
    !fs.existsSync(path.join(targetDir, binary))
);

if (criticalMissing.length > 0) {
    console.log('⚠️  Отсутствуют критичные бинарники:');
    criticalMissing.forEach(binary => console.log(`   - ${binary}`));
    console.log('💡 Возможно нужна пересборка или проверка Cargo.toml');
} else {
    console.log('✅ Все критичные инструменты на месте');
}