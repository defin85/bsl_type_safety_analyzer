#!/usr/bin/env node

const { execSync, spawn } = require('child_process');
const readline = require('readline');
const path = require('path');
const fs = require('fs');

// Цвета для консоли
const colors = {
    reset: '\x1b[0m',
    bright: '\x1b[1m',
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    magenta: '\x1b[35m',
    cyan: '\x1b[36m',
    gray: '\x1b[90m'
};

const c = (color, text) => {
    if (!colors[color]) {
        console.warn(`Unknown color: ${color}`);
        return text;
    }
    return `${colors[color]}${text}${colors.reset}`;
};

class InteractiveDev {
    constructor() {
        this.rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout
        });
        this.currentProcess = null;
        this.watchMode = false;
    }

    async start() {
        console.clear();
        this.showHeader();
        await this.showMainMenu();
    }

    showHeader() {
        console.log(c('cyan', '╔════════════════════════════════════════════════════════════╗'));
        console.log(c('cyan', '║') + c('bright', '          🚀 BSL Analyzer - Interactive Dev Tool            ') + c('cyan', '║'));
        console.log(c('cyan', '║') + c('yellow', '                      Версия 1.6.0                          ') + c('cyan', '║'));
        console.log(c('cyan', '╚════════════════════════════════════════════════════════════╝'));
        console.log();
    }

    async showMainMenu() {
        const projectInfo = this.getProjectInfo();
        
        console.log(c('bright', '📊 Статус проекта:'));
        console.log(`   Версия: ${c('green', projectInfo.version)}`);
        console.log(`   Статус: ${projectInfo.hasChanges ? c('yellow', 'Есть изменения') : c('green', 'Чисто')}`);
        console.log(`   Кеш: ${projectInfo.hasCache ? c('green', 'Актуален') : c('yellow', 'Отсутствует')}`);
        console.log();

        console.log(c('bright', '🎯 Выберите действие:') + '\n');
        
        // Динамическое описание watch-режима
        const chokidarAvailable = this.checkChokidarDependency();
        const watchDesc = chokidarAvailable 
            ? '👁️  Watch режим (автопересборка)'
            : '👁️  Watch режим ⚠️  (требует chokidar)';
        
        const options = [
            { key: '1', desc: '🧠 Умная сборка (рекомендуется)', cmd: 'build:smart', color: 'green' },
            { key: '2', desc: watchDesc, cmd: 'watch', color: chokidarAvailable ? 'blue' : 'yellow' },
            { key: '3', desc: '⚡ Быстрая dev сборка', cmd: 'dev', color: 'cyan' },
            { key: '4', desc: '🔧 Традиционная сборка', cmd: 'rebuild:extension', color: 'yellow' },
            { key: '5', desc: '📦 Release сборка', cmd: 'build:smart:release', color: 'magenta' },
            { key: '6', desc: '🧹 Полная пересборка с очисткой', action: 'full-rebuild', color: 'red' },
            '',
            { key: '7', desc: '🔄 Версионирование', submenu: 'version' },
            { key: '8', desc: '🚀 Git операции', submenu: 'git' },
            { key: '9', desc: '📤 Публикация расширения', submenu: 'publish' },
            { key: '0', desc: '🧹 Очистка и утилиты', submenu: 'utils' },
            '',
            { key: 's', desc: '📊 Статистика и диагностика', submenu: 'stats' },
            '',
            { key: 'h', desc: '❓ Справка по командам', action: 'help' },
            { key: 'q', desc: '❌ Выход', action: 'quit' }
        ];

        options.forEach(option => {
            if (option === '') {
                console.log();
            } else {
                const coloredDesc = option.color ? c(option.color, option.desc) : option.desc;
                console.log(`   ${c('bright', option.key)}. ${coloredDesc}`);
            }
        });

        console.log();
        const choice = await this.prompt(c('bright', '➤ Ваш выбор: '));
        await this.handleChoice(choice, options);
    }

    async showVersionMenu() {
        console.clear();
        this.showHeader();
        console.log(c('bright', '🔄 Управление версиями:') + '\n');
        
        const options = [
            { key: '1', desc: '📈 Patch версия (1.6.0 → 1.6.1)', cmd: 'version:patch' },
            { key: '2', desc: '📊 Minor версия (1.6.0 → 1.7.0)', cmd: 'version:minor' },
            { key: '3', desc: '🚀 Major версия (1.6.0 → 2.0.0)', cmd: 'version:major' },
            { key: '4', desc: '🔄 Синхронизация версий', cmd: 'version:sync' },
            '',
            { key: 'b', desc: '⬅️  Назад в главное меню', action: 'back' },
            { key: 'q', desc: '❌ Выход', action: 'quit' }
        ];

        options.forEach(option => {
            if (option === '') {
                console.log();
            } else {
                console.log(`   ${c('bright', option.key)}. ${option.desc}`);
            }
        });

        console.log();
        const choice = await this.prompt(c('bright', '➤ Ваш выбор: '));
        await this.handleChoice(choice, options);
    }

    async showGitMenu() {
        console.clear();
        this.showHeader();
        console.log(c('bright', '🚀 Git операции:') + '\n');
        
        const options = [
            { key: '1', desc: '💾 Умный коммит', action: 'smart-commit' },
            { key: '2', desc: '🚀 Умный коммит + push', action: 'smart-commit-push' },
            '',
            { key: '3', desc: '🏷️  Релиз patch (локально)', cmd: 'git:release patch' },
            { key: '4', desc: '🏷️  Релиз minor (локально)', cmd: 'git:release minor' },
            { key: '5', desc: '🏷️  Релиз major (локально)', cmd: 'git:release major' },
            '',
            { key: '6', desc: '🚀 Релиз patch + публикация', action: 'release-patch-publish' },
            { key: '7', desc: '🚀 Релиз minor + публикация', action: 'release-minor-publish' },
            { key: '8', desc: '🚀 Релиз major + публикация', action: 'release-major-publish' },
            '',
            { key: '9', desc: '🔄 Dev коммит', cmd: 'git:dev' },
            { key: '0', desc: '📤 Push текущих изменений', action: 'git-push' },
            '',
            { key: 'b', desc: '⬅️  Назад в главное меню', action: 'back' },
            { key: 'q', desc: '❌ Выход', action: 'quit' }
        ];

        options.forEach(option => {
            if (option === '') {
                console.log();
            } else {
                console.log(`   ${c('bright', option.key)}. ${option.desc}`);
            }
        });

        console.log();
        const choice = await this.prompt(c('bright', '➤ Ваш выбор: '));
        await this.handleChoice(choice, options);
    }

    async showPublishMenu() {
        console.clear();
        this.showHeader();
        console.log(c('bright', '📤 Публикация расширения:') + '\n');
        
        const projectInfo = this.getProjectInfo();
        console.log(c('cyan', `📦 Текущая версия: ${projectInfo.version}`));
        console.log(c('cyan', `📁 Статус: ${projectInfo.hasChanges ? 'Есть изменения' : 'Готово к публикации'}`));
        console.log();
        
        const options = [
            { key: '1', desc: '🏗️  Подготовить к публикации (build + package)', action: 'prepare-publish', color: 'green' },
            { key: '2', desc: '🚀 Опубликовать в VS Code Marketplace', cmd: 'publish:marketplace', color: 'blue' },
            { key: '3', desc: '📋 Проверить пакет перед публикацией', cmd: 'publish:check', color: 'yellow' },
            { key: '4', desc: '🏷️  Создать GitHub Release', cmd: 'publish:github', color: 'magenta' },
            '',
            { key: '5', desc: '📊 Показать информацию о пакете', action: 'package-info', color: 'cyan' },
            { key: '6', desc: '🔑 Настроить токен VS Code Marketplace', action: 'setup-token', color: 'yellow' },
            '',
            { key: 'b', desc: '⬅️  Назад в главное меню', action: 'back' },
            { key: 'q', desc: '❌ Выход', action: 'quit' }
        ];

        options.forEach(option => {
            if (option === '') {
                console.log();
            } else {
                const coloredDesc = option.color ? c(option.color, option.desc) : option.desc;
                console.log(`   ${c('bright', option.key)}. ${coloredDesc}`);
            }
        });

        console.log();
        const choice = await this.prompt(c('bright', '➤ Ваш выбор: '));
        await this.handleChoice(choice, options);
    }

    async showUtilsMenu() {
        console.clear();
        this.showHeader();
        console.log(c('bright', '🧹 Утилиты и очистка:') + '\n');
        
        const options = [
            { key: '1', desc: '🗑️  Очистить кеш сборки', action: 'clear-cache' },
            { key: '2', desc: '🧹 Очистить проект', cmd: 'cleanup:project' },
            { key: '3', desc: '💥 Глубокая очистка', cmd: 'deep-cleanup' },
            { key: '4', desc: '🔍 Проверить бинарники', cmd: 'check:binaries' },
            { key: '5', desc: '📦 Установить watch зависимости (chokidar)', cmd: 'watch:install' },
            '',
            { key: 'b', desc: '⬅️  Назад в главное меню', action: 'back' },
            { key: 'q', desc: '❌ Выход', action: 'quit' }
        ];

        options.forEach(option => {
            if (option === '') {
                console.log();
            } else {
                console.log(`   ${c('bright', option.key)}. ${option.desc}`);
            }
        });

        console.log();
        const choice = await this.prompt(c('bright', '➤ Ваш выбор: '));
        await this.handleChoice(choice, options);
    }

    async showStatsMenu() {
        console.clear();
        this.showHeader();
        console.log(c('bright', '📊 Статистика и диагностика:') + '\n');
        
        this.showProjectStats();
        
        const options = [
            { key: '1', desc: '🔄 Обновить статистику', action: 'refresh-stats' },
            { key: '2', desc: '📈 Бенчмарк сборки', action: 'benchmark' },
            { key: '3', desc: '🔍 Анализ размеров', action: 'size-analysis' },
            { key: '4', desc: '🗃️  Состояние кеша', action: 'cache-info' },
            '',
            { key: 'b', desc: '⬅️  Назад в главное меню', action: 'back' },
            { key: 'q', desc: '❌ Выход', action: 'quit' }
        ];

        options.forEach(option => {
            if (option === '') {
                console.log();
            } else {
                console.log(`   ${c('bright', option.key)}. ${option.desc}`);
            }
        });

        console.log();
        const choice = await this.prompt(c('bright', '➤ Ваш выбор: '));
        await this.handleChoice(choice, options);
    }

    async handleChoice(choice, options) {
        const option = options.find(o => o && o.key === choice.toLowerCase());
        
        if (!option) {
            console.log(c('red', '❌ Неверный выбор. Попробуйте еще раз.'));
            await this.sleep(1000);
            return this.showMainMenu();
        }

        if (option.action) {
            await this.handleAction(option.action);
        } else if (option.cmd) {
            await this.runCommand(option.cmd, option.desc);
        } else if (option.submenu) {
            await this.showSubmenu(option.submenu);
        }
    }

    async handleAction(action) {
        switch (action) {
            case 'help':
                await this.showHelp();
                break;
            case 'quit':
                this.cleanup();
                process.exit(0);
                break;
            case 'back':
                await this.showMainMenu();
                break;
            case 'smart-commit':
                await this.smartCommit();
                break;
            case 'clear-cache':
                await this.clearCache();
                break;
            case 'refresh-stats':
                await this.showStatsMenu();
                break;
            case 'benchmark':
                await this.runBenchmark();
                break;
            case 'size-analysis':
                await this.sizeAnalysis();
                break;
            case 'cache-info':
                await this.cacheInfo();
                break;
            case 'full-rebuild':
                await this.fullRebuild();
                break;
            case 'prepare-publish':
                await this.preparePublish();
                break;
            case 'package-info':
                await this.showPackageInfo();
                break;
            case 'setup-token':
                await this.setupMarketplaceToken();
                break;
            case 'smart-commit-push':
                await this.smartCommitPush();
                break;
            case 'release-patch-publish':
                await this.releaseWithPublish('patch');
                break;
            case 'release-minor-publish':
                await this.releaseWithPublish('minor');
                break;
            case 'release-major-publish':
                await this.releaseWithPublish('major');
                break;
            case 'git-push':
                await this.gitPush();
                break;
        }
    }

    async showSubmenu(submenu) {
        switch (submenu) {
            case 'version':
                await this.showVersionMenu();
                break;
            case 'git':
                await this.showGitMenu();
                break;
            case 'publish':
                await this.showPublishMenu();
                break;
            case 'utils':
                await this.showUtilsMenu();
                break;
            case 'stats':
                await this.showStatsMenu();
                break;
        }
    }

    async runCommand(cmd, description) {
        console.log();
        console.log(c('blue', `🚀 Выполняется: ${description || cmd}`));
        console.log(c('gray', `💻 npm run ${cmd}`));
        console.log('='.repeat(60));
        
        const startTime = Date.now();
        
        try {
            if (cmd === 'watch') {
                this.watchMode = true;
                await this.runWatchMode();
                return;
            }

            execSync(`npm run ${cmd}`, { stdio: 'inherit' });
            
            const duration = ((Date.now() - startTime) / 1000).toFixed(1);
            console.log('='.repeat(60));
            console.log(c('green', `✅ Команда выполнена успешно за ${duration}s`));
            
        } catch (error) {
            console.log('='.repeat(60));
            console.log(c('red', `❌ Ошибка исполнения команды`));
        }
        
        await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
        await this.showMainMenu();
    }

    // Проверка наличия chokidar зависимости
    checkChokidarDependency() {
        try {
            require.resolve('chokidar');
            return true;
        } catch (e) {
            return false;
        }
    }

    // Автоматическая установка chokidar
    async installChokidar() {
        console.log(c('yellow', '📦 Устанавливаю зависимость chokidar...'));
        console.log(c('gray', '💻 npm install --save-dev chokidar'));
        console.log('='.repeat(50));
        
        try {
            execSync('npm install --save-dev chokidar', { stdio: 'inherit' });
            console.log(c('green', '✅ Chokidar успешно установлен!'));
            return true;
        } catch (error) {
            console.log(c('red', '❌ Ошибка установки chokidar'));
            console.log(c('red', 'Попробуйте выполнить команду вручную: npm install --save-dev chokidar'));
            return false;
        }
    }

    async runWatchMode() {
        console.log(c('yellow', '\n👁️  Подготовка Watch режима...'));
        
        // Проверяем наличие chokidar
        if (!this.checkChokidarDependency()) {
            console.log(c('yellow', '⚠️  Зависимость chokidar не найдена'));
            console.log(c('gray', '   Watch режим требует установки file watcher библиотеки\n'));
            
            const install = await this.prompt(c('bright', '📦 Установить chokidar автоматически? (y/n): '));
            
            if (install.toLowerCase() === 'y' || install.toLowerCase() === 'yes' || install === '') {
                const success = await this.installChokidar();
                if (!success) {
                    await this.prompt(c('bright', '\n📄 Нажмите Enter для возврата в меню...'));
                    await this.showMainMenu();
                    return;
                }
                console.log();
            } else {
                console.log(c('yellow', '\n⏸️  Отменено. Для установки используйте:\n   npm run watch:install'));
                await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
                await this.showMainMenu();
                return;
            }
        }
        
        console.log(c('green', '✅ Chokidar найден, запускаю Watch режим...'));
        console.log(c('gray', 'Нажмите Ctrl+C для остановки\n'));
        
        const child = spawn('npm', ['run', 'watch'], {
            stdio: 'inherit',
            shell: true
        });
        
        this.currentProcess = child;
        
        child.on('close', (code) => {
            this.watchMode = false;
            this.currentProcess = null;
            console.log(c('yellow', '\n👁️  Watch режим остановлен'));
            this.showMainMenu();
        });
    }

    async smartCommit() {
        console.log();
        const message = await this.prompt(c('bright', '💾 Введите сообщение коммита: '));
        
        if (!message.trim()) {
            console.log(c('red', '❌ Сообщение не может быть пустым'));
            await this.sleep(1000);
            return this.showGitMenu();
        }

        // Вызываем git workflow напрямую с сообщением
        try {
            execSync(`npm run git:commit "${message}"`, { stdio: 'inherit' });
            console.log(c('green', '✅ Умный коммит выполнен успешно'));
        } catch (error) {
            console.log(c('red', '❌ Ошибка выполнения коммита'));
        }
        
        await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
        await this.showGitMenu();
    }

    async clearCache() {
        console.log();
        console.log(c('yellow', '🗑️  Очистка кеша сборки...'));
        
        try {
            if (fs.existsSync('.build-cache')) {
                fs.rmSync('.build-cache', { recursive: true, force: true });
                console.log(c('green', '✅ Кеш очищен'));
            } else {
                console.log(c('yellow', '⚠️  Кеш уже отсутствует'));
            }
        } catch (error) {
            console.log(c('red', `❌ Ошибка очистки: ${error.message}`));
        }
        
        await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
        await this.showUtilsMenu();
    }

    getProjectInfo() {
        try {
            const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
            const hasCache = fs.existsSync('.build-cache');
            const hasChanges = this.checkGitChanges();
            
            return {
                version: packageJson.version,
                hasCache,
                hasChanges
            };
        } catch (error) {
            return {
                version: 'unknown',
                hasCache: false,
                hasChanges: false
            };
        }
    }

    checkGitChanges() {
        try {
            const result = execSync('git status --porcelain', { encoding: 'utf8' });
            return result.trim().length > 0;
        } catch {
            return false;
        }
    }

    showProjectStats() {
        console.log(c('bright', '📈 Статистика проекта:'));
        
        try {
            // Размер target директории
            const targetSize = this.getDirSize('target');
            console.log(`   Размер target/: ${targetSize}`);
            
            // Размер node_modules
            const nodeModulesSize = this.getDirSize('node_modules');
            console.log(`   Размер node_modules/: ${nodeModulesSize}`);
            
            // Количество .rs файлов
            const rustFiles = this.countFiles('src', '.rs');
            console.log(`   Rust файлов: ${rustFiles}`);
            
            // Количество .ts файлов
            const tsFiles = this.countFiles('vscode-extension/src', '.ts');
            console.log(`   TypeScript файлов: ${tsFiles}`);
            
            // Размер кеша
            const cacheSize = fs.existsSync('.build-cache') ? this.getDirSize('.build-cache') : '0 B';
            console.log(`   Размер кеша: ${cacheSize}`);
            
            // Статус chokidar для watch-режима
            const chokidarStatus = this.checkChokidarDependency() 
                ? c('green', '✅ установлен')
                : c('red', '❌ не установлен');
            console.log(`   Watch зависимость: ${chokidarStatus}`);
            
        } catch (error) {
            console.log(c('yellow', '   ⚠️  Не удалось получить статистику'));
        }
        
        console.log();
    }

    getDirSize(dirPath) {
        if (!fs.existsSync(dirPath)) return '0 B';
        
        try {
            // Кроссплатформенное решение для получения размера директории
            const stats = this.getDirectoryStats(dirPath);
            const sizeMB = (stats.size / (1024 * 1024)).toFixed(1);
            return `${sizeMB} MB`;
        } catch {
            return 'Unknown';
        }
    }
    
    getProfile() {
        const buildMode = process.argv[3] || 'dev-fast';
        return buildMode === 'release' ? 'dev-fast' : buildMode;
    }
    
    getDirectoryStats(dirPath) {
        let totalSize = 0;
        let fileCount = 0;
        
        const scanDirectory = (dir) => {
            try {
                const items = fs.readdirSync(dir);
                
                for (const item of items) {
                    const itemPath = path.join(dir, item);
                    try {
                        const stats = fs.statSync(itemPath);
                        
                        if (stats.isDirectory()) {
                            scanDirectory(itemPath);
                        } else {
                            totalSize += stats.size;
                            fileCount++;
                        }
                    } catch (e) {
                        // Игнорируем недоступные файлы
                    }
                }
            } catch (e) {
                // Игнорируем недоступные директории
            }
        };
        
        scanDirectory(dirPath);
        return { size: totalSize, files: fileCount };
    }

    countFiles(dirPath, extension) {
        if (!fs.existsSync(dirPath)) return 0;
        
        let count = 0;
        
        const scanDirectory = (dir) => {
            try {
                const items = fs.readdirSync(dir);
                
                for (const item of items) {
                    const itemPath = path.join(dir, item);
                    try {
                        const stats = fs.statSync(itemPath);
                        
                        if (stats.isDirectory()) {
                            scanDirectory(itemPath);
                        } else if (item.endsWith(extension)) {
                            count++;
                        }
                    } catch (e) {
                        // Игнорируем недоступные файлы
                    }
                }
            } catch (e) {
                // Игнорируем недоступные директории
            }
        };
        
        scanDirectory(dirPath);
        return count;
    }

    async runBenchmark() {
        console.log();
        console.log(c('blue', '📈 Запуск бенчмарка сборки...'));
        console.log('='.repeat(60));
        
        const tests = [
            { name: 'Умная сборка (с кешем)', cmd: 'build:smart' },
            { name: 'Dev сборка', cmd: 'dev' },
            { name: 'Традиционная сборка', cmd: 'rebuild:extension' }
        ];
        
        for (const test of tests) {
            console.log(c('bright', `\n🏃 Тест: ${test.name}`));
            const startTime = Date.now();
            
            try {
                execSync(`npm run ${test.cmd}`, { stdio: 'ignore' });
                const duration = ((Date.now() - startTime) / 1000).toFixed(1);
                console.log(c('green', `   ✅ Завершено за ${duration}s`));
            } catch (error) {
                console.log(c('red', `   ❌ Ошибка`));
            }
        }
        
        console.log('='.repeat(60));
        await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
        await this.showStatsMenu();
    }

    async sizeAnalysis() {
        console.log();
        console.log(c('blue', '🔍 Анализ размеров...'));
        
        const paths = [
            'target/release',
            'target/dev-fast',
            'target/debug', 
            'vscode-extension/out',
            'vscode-extension/dist',
            'vscode-extension/bin'
        ];
        
        paths.forEach(p => {
            if (fs.existsSync(p)) {
                const size = this.getDirSize(p);
                console.log(`   ${p}: ${size}`);
            } else {
                console.log(`   ${p}: ${c('gray', 'не существует')}`);
            }
        });
        
        await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
        await this.showStatsMenu();
    }

    async cacheInfo() {
        console.log();
        console.log(c('blue', '🗃️  Состояние кеша:'));
        
        if (fs.existsSync('.build-cache')) {
            const files = fs.readdirSync('.build-cache');
            files.forEach(file => {
                const filePath = path.join('.build-cache', file);
                const stats = fs.statSync(filePath);
                const modified = stats.mtime.toLocaleString();
                console.log(`   ${file}: ${c('green', 'существует')} (${modified})`);
            });
        } else {
            console.log(c('yellow', '   ⚠️  Кеш отсутствует'));
        }
        
        await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
        await this.showStatsMenu();
    }

    async fullRebuild() {
        console.clear();
        this.showHeader();
        console.log(c('red', '🧹 ПОЛНАЯ ПЕРЕСБОРКА С ОЧИСТКОЙ') + '\n');
        
        console.log(c('yellow', '⚠️  Внимание! Это действие:'));
        console.log('   • Удалит ВСЕ артефакты сборки (target/, node_modules/out/)');
        console.log('   • Очистит умный кеш (.build-cache/)');
        console.log('   • Займёт значительно больше времени (~3-5 минут)');
        console.log('   • Заново скачает и скомпилирует все зависимости');
        console.log();
        
        const confirm = await this.prompt(c('bright', '🤔 Продолжить полную пересборку? (y/N): '));
        
        if (confirm.toLowerCase() !== 'y' && confirm.toLowerCase() !== 'yes') {
            console.log(c('yellow', '\n⏸️  Отменено пользователем'));
            await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
            await this.showMainMenu();
            return;
        }

        console.log();
        console.log(c('blue', '🚀 Начинаем полную пересборку...'));
        console.log('='.repeat(60));
        
        const startTime = Date.now();
        let success = true;

        // Этап 1: Очистка Cargo кеша
        console.log(c('blue', '\n🧹 [1/6] Очистка Cargo артефактов...'));
        try {
            execSync('cargo clean', { stdio: 'inherit' });
            console.log(c('green', '✅ Cargo кеш очищен'));
        } catch (error) {
            console.log(c('red', '❌ Ошибка очистки Cargo кеша'));
            success = false;
        }

        // Этап 2: Очистка умного кеша
        console.log(c('blue', '\n🧹 [2/6] Очистка умного кеша...'));
        try {
            if (fs.existsSync('.build-cache')) {
                fs.rmSync('.build-cache', { recursive: true, force: true });
                console.log(c('green', '✅ Умный кеш очищен'));
            } else {
                console.log(c('yellow', '⚠️  Умный кеш уже отсутствует'));
            }
        } catch (error) {
            console.log(c('red', '❌ Ошибка очистки умного кеша'));
            success = false;
        }

        // Этап 3: Очистка TypeScript артефактов  
        console.log(c('blue', '\n🧹 [3/6] Очистка TypeScript артефактов...'));
        try {
            const pathsToClean = [
                'vscode-extension/out',
                'vscode-extension/dist', 
                'vscode-extension/bin'
            ];
            
            for (const pathToClean of pathsToClean) {
                if (fs.existsSync(pathToClean)) {
                    fs.rmSync(pathToClean, { recursive: true, force: true });
                    console.log(`   ✅ ${pathToClean} очищен`);
                }
            }
            
            console.log(c('green', '✅ TypeScript артефакты очищены'));
        } catch (error) {
            console.log(c('red', '❌ Ошибка очистки TypeScript артефактов'));
            success = false;
        }

        // Этап 4: Пересборка Rust
        console.log(c('blue', '\n🦀 [4/6] Пересборка Rust с нуля...'));
        try {
            execSync('cargo build --profile dev-fast --jobs 4', { stdio: 'inherit' });
            console.log(c('green', '✅ Rust пересборка завершена'));
        } catch (error) {
            console.log(c('red', '❌ Ошибка пересборки Rust'));
            success = false;
        }

        // Этап 5: Копирование бинарников
        if (success) {
            console.log(c('blue', '\n📁 [5/6] Копирование оптимизированных бинарников...'));
            try {
                execSync('node scripts/copy-essential-binaries.js dev-fast', { stdio: 'inherit' });
                console.log(c('green', '✅ Бинарники скопированы'));
            } catch (error) {
                console.log(c('red', '❌ Ошибка копирования бинарников'));
                success = false;
            }
        }

        // Этап 6: Пересборка расширения
        if (success) {
            console.log(c('blue', '\n📦 [6/6] Пересборка VSCode расширения...'));
            try {
                execSync('cd vscode-extension && npm run compile', { stdio: 'inherit' });
                console.log(c('green', '✅ VSCode расширение пересобрано'));
            } catch (error) {
                console.log(c('red', '❌ Ошибка пересборки расширения'));
                success = false;
            }
        }

        // Итоговая статистика
        const totalTime = ((Date.now() - startTime) / 1000).toFixed(1);
        console.log('\n' + '='.repeat(60));
        
        if (success) {
            console.log(c('green', '🎉 ПОЛНАЯ ПЕРЕСБОРКА ЗАВЕРШЕНА УСПЕШНО!'));
            console.log(c('green', `⏱️  Общее время: ${totalTime}s`));
            console.log(c('cyan', '💡 Теперь все кеши свежие и проект полностью пересобран'));
            
            // Показываем размер нового расширения
            try {
                const vsixFiles = fs.readdirSync('vscode-extension').filter(f => f.endsWith('.vsix'));
                if (vsixFiles.length > 0) {
                    const vsixPath = path.join('vscode-extension', vsixFiles[0]);
                    const stats = fs.statSync(vsixPath);
                    const sizeMB = (stats.size / (1024 * 1024)).toFixed(1);
                    console.log(c('cyan', `📦 Размер нового .vsix: ${sizeMB} MB`));
                }
            } catch (e) {
                // Игнорируем ошибки получения статистики
            }
        } else {
            console.log(c('red', '💥 ПОЛНАЯ ПЕРЕСБОРКА НЕ УДАЛАСЬ'));
            console.log(c('red', `⏱️  Время до ошибки: ${totalTime}s`)); 
            console.log(c('yellow', '💡 Проверьте ошибки выше и повторите попытку'));
        }
        
        console.log('='.repeat(60));
        await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
        await this.showMainMenu();
    }

    async preparePublish() {
        console.log();
        console.log(c('blue', '🏗️  Подготовка к публикации...'));
        console.log('='.repeat(60));
        
        const startTime = Date.now();
        let success = true;
        
        // Этап 1: Проверка состояния Git
        console.log(c('blue', '\n🔍 [1/4] Проверка состояния Git...'));
        try {
            const hasChanges = this.checkGitChanges();
            if (hasChanges) {
                console.log(c('yellow', '⚠️  Есть несохраненные изменения'));
                const commit = await this.prompt(c('bright', '💾 Сделать коммит перед публикацией? (y/N): '));
                
                if (commit.toLowerCase() === 'y' || commit.toLowerCase() === 'yes') {
                    const message = await this.prompt(c('bright', '💬 Сообщение коммита: '));
                    if (message.trim()) {
                        execSync(`git add . && git commit -m "${message}"`, { stdio: 'inherit' });
                        console.log(c('green', '✅ Изменения закоммичены'));
                    }
                }
            } else {
                console.log(c('green', '✅ Репозиторий чист'));
            }
        } catch (error) {
            console.log(c('yellow', '⚠️  Не удалось проверить статус Git'));
        }
        
        // Этап 2: Синхронизация версий
        console.log(c('blue', '\n🔄 [2/4] Синхронизация версий...'));
        try {
            execSync('npm run version:sync', { stdio: 'inherit' });
            console.log(c('green', '✅ Версии синхронизированы'));
        } catch (error) {
            console.log(c('red', '❌ Ошибка синхронизации версий'));
            success = false;
        }
        
        // Этап 3: Умная сборка
        if (success) {
            console.log(c('blue', '\n🧠 [3/4] Умная сборка...'));
            try {
                execSync('npm run build:smart:release', { stdio: 'inherit' });
                console.log(c('green', '✅ Сборка завершена'));
            } catch (error) {
                console.log(c('red', '❌ Ошибка сборки'));
                success = false;
            }
        }
        
        // Этап 4: Создание пакета
        if (success) {
            console.log(c('blue', '\n📦 [4/4] Создание .vsix пакета...'));
            try {
                execSync('cd vscode-extension && npx @vscode/vsce package', { stdio: 'inherit' });
                console.log(c('green', '✅ Пакет создан'));
            } catch (error) {
                console.log(c('red', '❌ Ошибка создания пакета'));
                success = false;
            }
        }
        
        // Результат
        const totalTime = ((Date.now() - startTime) / 1000).toFixed(1);
        console.log('\n' + '='.repeat(60));
        
        if (success) {
            console.log(c('green', '🎉 ПОДГОТОВКА К ПУБЛИКАЦИИ ЗАВЕРШЕНА!'));
            console.log(c('green', `⏱️  Время: ${totalTime}s`));
            
            // Показываем информацию о созданном пакете
            await this.showPackageInfo();
        } else {
            console.log(c('red', '💥 ПОДГОТОВКА НЕ УДАЛАСЬ'));
            console.log(c('red', `⏱️  Время: ${totalTime}s`));
        }
        
        console.log('='.repeat(60));
        await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
        await this.showPublishMenu();
    }
    
    async showPackageInfo() {
        console.log('\n' + c('bright', '📊 Информация о пакете:'));
        
        try {
            const projectInfo = this.getProjectInfo();
            console.log(`   Версия: ${c('green', projectInfo.version)}`);
            
            // Проверяем наличие .vsix файлов
            const vsixPattern = `vscode-extension/bsl-type-safety-analyzer-*.vsix`;
            const glob = require('glob');
            const vsixFiles = glob.sync(vsixPattern);
            
            if (vsixFiles.length > 0) {
                const latestFile = vsixFiles[vsixFiles.length - 1];
                const stats = fs.statSync(latestFile);
                const sizeMB = (stats.size / (1024 * 1024)).toFixed(1);
                const modified = stats.mtime.toLocaleString();
                
                console.log(`   Файл: ${c('cyan', path.basename(latestFile))}`);
                console.log(`   Размер: ${c('yellow', sizeMB)} MB`);
                console.log(`   Создан: ${c('gray', modified)}`);
                console.log(`   Путь: ${c('gray', latestFile)}`);
                
                // Проверяем содержимое пакета
                console.log(`   Статус: ${c('green', 'Готов к публикации')}`);
            } else {
                console.log(`   Статус: ${c('red', 'Пакет не найден')}`);
                console.log(c('yellow', '   💡 Выполните сборку: npm run build:smart')); 
            }
        } catch (error) {
            console.log(c('red', `   ❌ Ошибка получения информации: ${error.message}`));
        }
    }
    
    async setupMarketplaceToken() {
        console.log('\n' + c('bright', '🔑 Настройка токена VS Code Marketplace'));
        console.log('='.repeat(50));
        
        console.log(c('yellow', '📋 Инструкции:'));
        console.log('1. Перейдите на: https://marketplace.visualstudio.com/manage');
        console.log('2. Создайте Personal Access Token с правами Marketplace');
        console.log('3. Выполните команду: npx @vscode/vsce login <ваш-publisher-id>');
        console.log('4. Введите токен когда будет запрошен');
        console.log();
        
        const setup = await this.prompt(c('bright', '🚀 Выполнить настройку сейчас? (y/N): '));
        
        if (setup.toLowerCase() === 'y' || setup.toLowerCase() === 'yes') {
            const publisherId = await this.prompt(c('bright', '👤 Publisher ID: '));
            
            if (publisherId.trim()) {
                try {
                    console.log(c('blue', `🔐 Настройка токена для ${publisherId}...`));
                    execSync(`cd vscode-extension && npx @vscode/vsce login ${publisherId}`, { stdio: 'inherit' });
                    console.log(c('green', '✅ Токен настроен успешно!'));
                } catch (error) {
                    console.log(c('red', '❌ Ошибка настройки токена'));
                }
            }
        }
        
        await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
        await this.showPublishMenu();
    }
    
    async smartCommitPush() {
        console.log();
        const message = await this.prompt(c('bright', '💾 Введите сообщение коммита: '));
        
        if (!message.trim()) {
            console.log(c('red', '❌ Сообщение не может быть пустым'));
            await this.sleep(1000);
            return this.showGitMenu();
        }

        try {
            // Коммит
            console.log(c('blue', '💾 Выполняется коммит...'));
            execSync(`git add . && git commit -m "${message}"`, { stdio: 'inherit' });
            console.log(c('green', '✅ Коммит выполнен'));
            
            // Push
            console.log(c('blue', '🚀 Отправка в удаленный репозиторий...'));
            execSync('git push', { stdio: 'inherit' });
            console.log(c('green', '✅ Push выполнен успешно'));
            
        } catch (error) {
            console.log(c('red', '❌ Ошибка Git операции'));
        }
        
        await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
        await this.showGitMenu();
    }
    
    async releaseWithPublish(type) {
        console.log();
        console.log(c('blue', `🚀 Релиз ${type} с публикацией...`));
        console.log('='.repeat(60));
        
        const startTime = Date.now();
        let success = true;
        
        try {
            // 1. Создание релиза
            console.log(c('blue', `\n🏷️  [1/3] Создание релиза ${type}...`));
            execSync(`npm run git:release ${type}`, { stdio: 'inherit' });
            console.log(c('green', `✅ Релиз ${type} создан`));
            
            // 2. Push с тегами
            console.log(c('blue', '\n🚀 [2/3] Отправка в Git с тегами...'));
            execSync('git push origin main --follow-tags', { stdio: 'inherit' });
            console.log(c('green', '✅ Push с тегами выполнен'));
            
            // 3. Публикация в Marketplace
            console.log(c('blue', '\n📤 [3/3] Публикация в VS Code Marketplace...'));
            
            const confirm = await this.prompt(c('bright', '🤔 Опубликовать в Marketplace сейчас? (y/N): '));
            if (confirm.toLowerCase() === 'y' || confirm.toLowerCase() === 'yes') {
                execSync('npm run publish:marketplace', { stdio: 'inherit' });
                console.log(c('green', '✅ Опубликовано в Marketplace!'));
            } else {
                console.log(c('yellow', '⏸️  Публикация отложена'));
            }
            
        } catch (error) {
            console.log(c('red', `❌ Ошибка релиза: ${error.message}`));
            success = false;
        }
        
        const totalTime = ((Date.now() - startTime) / 1000).toFixed(1);
        console.log('\n' + '='.repeat(60));
        
        if (success) {
            console.log(c('green', `🎉 РЕЛИЗ ${type.toUpperCase()} ЗАВЕРШЕН!`));
            console.log(c('green', `⏱️  Время: ${totalTime}s`));
        } else {
            console.log(c('red', `💥 РЕЛИЗ ${type.toUpperCase()} НЕ УДАЛСЯ`));
        }
        
        console.log('='.repeat(60));
        await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
        await this.showGitMenu();
    }
    
    async gitPush() {
        console.log();
        console.log(c('blue', '📤 Отправка изменений в удаленный репозиторий...'));
        
        try {
            // Проверяем статус
            const status = execSync('git status --porcelain', { encoding: 'utf8' });
            if (status.trim().length === 0) {
                console.log(c('yellow', '⚠️  Нет изменений для отправки'));
            } else {
                console.log(c('yellow', '⚠️  Есть несохраненные изменения. Сначала сделайте коммит.'));
                return;
            }
            
            // Проверяем есть ли коммиты для push
            try {
                execSync('git log @{u}..HEAD --oneline', { stdio: 'ignore' });
                execSync('git push', { stdio: 'inherit' });
                console.log(c('green', '✅ Push выполнен успешно'));
            } catch {
                console.log(c('yellow', '⚠️  Нет коммитов для отправки или проблемы с удаленным репозиторием'));
            }
            
        } catch (error) {
            console.log(c('red', `❌ Ошибка push: ${error.message}`));
        }
        
        await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
        await this.showGitMenu();
    }

    async showHelp() {
        console.clear();
        this.showHeader();
        console.log(c('bright', '❓ Справка по командам:') + '\n');
        
        console.log(c('green', '🧠 Умная сборка') + ' - быстрая сборка с кешированием (рекомендуется)');
        console.log(c('blue', '👁️  Watch режим') + ' - автоматическая пересборка при изменениях');
        console.log(c('cyan', '⚡ Dev сборка') + ' - быстрая разработческая сборка');
        console.log(c('yellow', '🔧 Традиционная') + ' - полная пересборка без кеша');
        console.log(c('magenta', '📦 Release сборка') + ' - оптимизированная сборка для продакшена');
        console.log(c('red', '🧹 Полная пересборка') + ' - cargo clean + полная пересборка (решает проблемы с кешем)');
        console.log();
        console.log(c('bright', 'Горячие клавиши:'));
        console.log('   h - справка');
        console.log('   q - выход');
        console.log('   b - назад (в подменю)');
        console.log('   Ctrl+C - остановить текущую операцию');
        
        await this.prompt(c('bright', '\n📄 Нажмите Enter для продолжения...'));
        await this.showMainMenu();
    }

    prompt(question) {
        return new Promise(resolve => {
            this.rl.question(question, resolve);
        });
    }

    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    cleanup() {
        if (this.currentProcess) {
            this.currentProcess.kill();
        }
        this.rl.close();
    }
}

// Обработка Ctrl+C
process.on('SIGINT', () => {
    console.log(c('yellow', '\n\n🛑 Завершение работы...'));
    process.exit(0);
});

// Запуск
const app = new InteractiveDev();
app.start().catch(console.error);