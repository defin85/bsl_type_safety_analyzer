#!/usr/bin/env node

const prompts = require('prompts');
const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

// Цветовая схема
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

function c(color, text) {
    return `${colors[color] || ''}${text}${colors.reset}`;
}

/**
 * Универсальная консоль разработки BSL Type Safety Analyzer v2.0
 * Поддерживает 39 функций в 6 категориях с интерактивной навигацией
 */
class UniversalDevConsole {
    constructor() {
        this.version = '2.0.0';
        this.currentProcess = null;
        this.config = this.loadConfig();
        
        // Обработка сигналов
        this.setupSignalHandlers();
    }

    /**
     * Загрузка конфигурации
     */
    loadConfig() {
        const defaultConfig = {
            enabledCategories: ['build', 'version', 'dev', 'git', 'publish', 'utils'],
            confirmDestructiveActions: true,
            showProgressBars: true,
            autoReturnToMainMenu: false,
            logErrors: true,
            favoriteActions: [],
            customCommands: {},
            shortcuts: {
                'ctrl+c': 'exit',
                'escape': 'back'
            }
        };

        try {
            const configPath = path.join(process.cwd(), '.dev-console-config.json');
            if (fs.existsSync(configPath)) {
                const userConfig = JSON.parse(fs.readFileSync(configPath, 'utf8'));
                const mergedConfig = { ...defaultConfig, ...userConfig };
                console.log(c('green', `✅ Конфигурация загружена: ${configPath}`));
                return mergedConfig;
            }
        } catch (error) {
            console.log(c('yellow', `⚠️ Ошибка загрузки конфигурации: ${error.message}`));
            console.log(c('cyan', '📝 Используется конфигурация по умолчанию'));
        }
        
        return defaultConfig;
    }

    /**
     * Сохранение конфигурации
     */
    saveConfig() {
        try {
            const configPath = path.join(process.cwd(), '.dev-console-config.json');
            fs.writeFileSync(configPath, JSON.stringify(this.config, null, 4));
            console.log(c('green', `✅ Конфигурация сохранена: ${configPath}`));
            return true;
        } catch (error) {
            console.log(c('red', `❌ Ошибка сохранения конфигурации: ${error.message}`));
            return false;
        }
    }

    /**
     * Проверка на деструктивное действие
     */
    isDestructiveAction(actionId) {
        const destructiveActions = [
            'clean-rebuild',
            'deep-cleanup', 
            'cargo-clean',
            'cleanup-project',
            'publish-marketplace',
            'publish-github',
            'git-release'
        ];
        return destructiveActions.includes(actionId);
    }

    /**
     * Подтверждение деструктивного действия
     */
    async confirmDestructive(actionTitle) {
        if (!this.config.confirmDestructiveActions) {
            return true;
        }

        const response = await prompts({
            type: 'confirm',
            name: 'confirm',
            message: `⚠️ "${actionTitle}" - потенциально опасная операция. Продолжить?`,
            initial: false
        });

        return response.confirm || false;
    }

    /**
     * Настройка обработчиков сигналов
     */
    setupSignalHandlers() {
        const cleanup = () => {
            console.log(c('yellow', '\n\n🛑 Завершение работы...'));
            if (this.currentProcess) {
                this.currentProcess.kill();
            }
            process.exit(0);
        };

        process.on('SIGINT', cleanup);
        process.on('SIGTERM', cleanup);
        
        // Обработка закрытия prompts (ESC)
        process.on('SIGTSTP', () => {
            console.log(c('yellow', '\n⏸️ Приостановлено'));
        });
    }

    /**
     * Показать заголовок
     */
    showHeader() {
        console.clear();
        console.log(c('cyan', '╔══════════════════════════════════════════════════════════════╗'));
        console.log(c('cyan', '║') + c('bright', '     🚀 BSL Analyzer - Universal Dev Console v2.0             ') + c('cyan', '║'));
        console.log(c('cyan', '║') + c('green', '                39 функций в 6 категориях                     ') + c('cyan', '║'));
        console.log(c('cyan', '╚══════════════════════════════════════════════════════════════╝'));
        console.log();
    }

    /**
     * Определение категорий меню
     */
    getCategories() {
        return [
            {
                id: 'build',
                title: '📦   Сборка и разработка',
                icon: '📦',
                description: '8 функций сборки и разработки',
                enabled: this.config.enabledCategories.includes('build')
            },
            {
                id: 'version', 
                title: '🔄   Версионирование',
                icon: '🔄',
                description: '6 функций управления версиями',
                enabled: this.config.enabledCategories.includes('version')
            },
            {
                id: 'dev',
                title: '🔧   Разработка и качество',
                icon: '🔧',
                description: '5 функций разработки',
                enabled: this.config.enabledCategories.includes('dev')
            },
            {
                id: 'git',
                title: '📋   Git операции',
                icon: '📋',
                description: '8 Git функций',
                enabled: this.config.enabledCategories.includes('git')
            },
            {
                id: 'publish',
                title: '🚀   Публикация',
                icon: '🚀',
                description: '7 функций публикации',
                enabled: this.config.enabledCategories.includes('publish')
            },
            {
                id: 'utils',
                title: '⚙️   Утилиты и диагностика',
                icon: '⚙️',
                description: '5 утилит и диагностика',
                enabled: this.config.enabledCategories.includes('utils')
            }
        ].filter(cat => cat.enabled);
    }

    /**
     * Главное меню
     */
    async showMainMenu() {
        while (true) {
            this.showHeader();
            
            const categories = this.getCategories();
            const choices = [
                ...categories.map(cat => ({
                    title: `${cat.title}`,
                    description: cat.description,
                    value: cat.id
                })),
                { title: '❌   Выход', value: 'exit' }
            ];

            const response = await prompts({
                type: 'select',
                name: 'category',
                message: '🎯 Выберите категорию:',
                choices: choices,
                initial: 0
            });

            // Обработка отмены (ESC/Ctrl+C)
            if (!response.category) {
                console.log(c('yellow', '\n👋 До свидания!'));
                process.exit(0);
            }

            if (response.category === 'exit') {
                console.log(c('yellow', '\n👋 До свидания!'));
                process.exit(0);
            }

            // Переход к выбранной категории
            await this.showCategoryMenu(response.category);
        }
    }

    /**
     * Меню категории
     */
    async showCategoryMenu(categoryId) {
        const category = this.getCategories().find(cat => cat.id === categoryId);
        if (!category) {
            console.log(c('red', '❌ Категория не найдена'));
            return;
        }

        while (true) {
            this.showHeader();
            console.log(c('bright', `${category.title}`));
            console.log(c('gray', category.description));
            console.log();

            const actions = this.getCategoryActions(categoryId);
            const choices = [
                ...actions.map(action => ({
                    title: action.title,
                    description: action.description || '',
                    value: action.id,
                    disabled: action.disabled
                })),
                { title: '⬅️   Назад в главное меню', value: 'back' }
            ];

            const response = await prompts({
                type: 'select',
                name: 'action',
                message: 'Выберите действие:',
                choices: choices,
                initial: 0
            });

            // Обработка отмены или возврата
            if (!response.action || response.action === 'back') {
                return; // Возврат в главное меню
            }

            // Проверка на деструктивность и выполнение действия
            const action = actions.find(a => a.id === response.action);
            if (this.isDestructiveAction(response.action)) {
                const confirmed = await this.confirmDestructive(action?.title || response.action);
                if (!confirmed) {
                    console.log(c('yellow', '⏸️ Операция отменена'));
                    await prompts({
                        type: 'text',
                        name: 'continue',
                        message: 'Нажмите Enter для продолжения...',
                        initial: ''
                    });
                    continue;
                }
            }
            
            await this.executeAction(categoryId, response.action);
            
            // Автовозврат в главное меню (если включен в конфиге)
            if (!this.config.autoReturnToMainMenu) {
                const continueResponse = await prompts({
                    type: 'confirm',
                    name: 'continue',
                    message: 'Остаться в этой категории?',
                    initial: false
                });
                
                if (!continueResponse.continue) {
                    return;
                }
            }
        }
    }

    /**
     * Получение действий для категории
     */
    getCategoryActions(categoryId) {
        const actionsMap = {
            // 📦 Сборка и разработка (8 функций)
            build: [
                { id: 'dev-build', title: '⚡    Быстрая dev сборка', description: 'npm run dev' },
                { id: 'smart-build', title: '🧠    Smart сборка с кешированием', description: 'npm run build:smart' },
                { id: 'smart-dev', title: '🧠    Smart dev сборка', description: 'npm run build:smart:dev' },
                { id: 'smart-release', title: '🧠    Smart release сборка', description: 'npm run build:smart:release' },
                { id: 'release-build', title: '🏗️    Release сборка (полная)', description: 'npm run build:release' },
                { id: 'watch-mode', title: '👁️    Watch режим (мониторинг)', description: 'Файловый мониторинг + автопересборка' },
                { id: 'rebuild-extension', title: '📦    Пересборка расширения', description: 'npm run rebuild:extension' },
                { id: 'clean-rebuild', title: '🧹    Очистка и полная пересборка', description: 'cargo clean + npm cleanup + release' }
            ],
            
            // 🔄 Версионирование (6 функций)
            version: [
                { id: 'version-patch', title: '🔢 Увеличить patch (x.x.X)', description: 'npm run version:patch' },
                { id: 'version-minor', title: '🔢 Увеличить minor (x.X.x)', description: 'npm run version:minor' },
                { id: 'version-major', title: '🔢 Увеличить major (X.x.x)', description: 'npm run version:major' },
                { id: 'version-sync', title: '🔄 Синхронизация версий', description: 'npm run version:sync' },
                { id: 'build-patch', title: '🏗️ Сборка с patch версией', description: 'npm run build:patch' },
                { id: 'build-minor', title: '🏗️ Сборка с minor версией', description: 'npm run build:minor' }
            ],
            
            // 🔧 Разработка и качество (5 функций)
            dev: [
                { id: 'run-tests', title: '🧪 Запустить тесты', description: 'cargo test' },
                { id: 'check-code', title: '🔍 Проверить код (clippy)', description: 'cargo clippy' },
                { id: 'format-code', title: '📝 Форматировать код', description: 'cargo fmt' },
                { id: 'check-binaries', title: '🔍 Проверить бинарные файлы', description: 'npm run check:binaries' },
                { id: 'project-info', title: '📊 Информация о проекте', description: 'Версии, зависимости, git статус' }
            ],
            
            // 📋 Git операции (8 функций)
            git: [
                { id: 'git-status', title: '📊 Git статус', description: 'git status' },
                { id: 'smart-commit', title: '📝 Умный коммит', description: 'git add + commit (интерактивно)' },
                { id: 'commit-push', title: '📤 Коммит и пуш', description: 'git add + commit + push' },
                { id: 'git-dev', title: '🔧 Dev workflow', description: 'npm run git:dev' },
                { id: 'git-release', title: '🚀 Release workflow', description: 'npm run git:release' },
                { id: 'git-commit', title: '💾 Простой коммит', description: 'npm run git:commit' },
                { id: 'git-version', title: '🏷️ Version workflow', description: 'npm run git:version' },
                { id: 'git-log', title: '📜 История коммитов', description: 'git log --oneline -10' }
            ],
            
            // 🚀 Публикация (7 функций)
            publish: [
                { id: 'package-extension', title: '📦 Упаковать расширение', description: 'npm run package:extension' },
                { id: 'publish-marketplace', title: '🏪 Опубликовать в Marketplace', description: 'npm run publish:marketplace' },
                { id: 'publish-github', title: '🐙 Опубликовать на GitHub', description: 'npm run publish:github' },
                { id: 'publish-check', title: '🔍 Проверить публикацию', description: 'npm run publish:check' },
                { id: 'clean-packages', title: '🧹 Очистить старые пакеты', description: 'npm run clean:old-packages' },
                { id: 'copy-binaries', title: '📋 Копировать бинарники', description: 'npm run copy:binaries:release' },
                { id: 'build-versioned', title: '🏗️ Сборка с версией', description: 'Интерактивный выбор patch/minor/major' }
            ],
            
            // ⚙️ Утилиты и диагностика (5 функций)
            utils: [
                { id: 'cleanup-project', title: '🧹 Очистка проекта', description: 'npm run cleanup:project' },
                { id: 'deep-cleanup', title: '🗑️ Глубокая очистка', description: 'npm run deep-cleanup' },
                { id: 'watch-install', title: '👁️ Установить watch зависимости', description: 'npm install chokidar' },
                { id: 'cargo-clean', title: '🦀 Очистить Cargo cache', description: 'cargo clean' },
                { id: 'show-logs', title: '📄 Показать логи ошибок', description: 'Просмотр error log файла' }
            ]
        };
        
        return actionsMap[categoryId] || [];
    }

    /**
     * Выполнение действия
     */
    async executeAction(categoryId, actionId) {
        console.log(c('blue', `\n🔧 Выполняется: ${categoryId}/${actionId}`));
        
        // Реализация всех функций по категориям
        switch (`${categoryId}/${actionId}`) {
            // 📦 Сборка и разработка (8 функций)
            case 'build/dev-build':
                return await this.runCommand('npm run dev', 'Быстрая dev сборка');
            case 'build/smart-build':
                return await this.runCommand('npm run build:smart', 'Smart сборка с кешированием');
            case 'build/smart-dev':
                return await this.runCommand('npm run build:smart:dev', 'Smart dev сборка');
            case 'build/smart-release':
                return await this.runCommand('npm run build:smart:release', 'Smart release сборка');
            case 'build/release-build':
                return await this.runCommand('npm run build:release', 'Release сборка (полная)');
            case 'build/watch-mode':
                return await this.startWatchMode();
            case 'build/rebuild-extension':
                return await this.runCommand('npm run rebuild:extension', 'Пересборка расширения');
            case 'build/clean-rebuild':
                return await this.cleanAndRebuild();
                
            // 🔄 Версионирование (6 функций)
            case 'version/version-patch':
                return await this.runCommand('npm run version:patch', 'Увеличение patch версии');
            case 'version/version-minor':
                return await this.runCommand('npm run version:minor', 'Увеличение minor версии');
            case 'version/version-major':
                return await this.runCommand('npm run version:major', 'Увеличение major версии');
            case 'version/version-sync':
                return await this.runCommand('npm run version:sync', 'Синхронизация версий');
            case 'version/build-patch':
                return await this.runCommand('npm run build:patch', 'Сборка с patch версией');
            case 'version/build-minor':
                return await this.runCommand('npm run build:minor', 'Сборка с minor версией');
                
            // 🔧 Разработка и качество (5 функций)
            case 'dev/run-tests':
                return await this.runCommand('cargo test', 'Запуск тестов');
            case 'dev/check-code':
                return await this.runCommand('cargo clippy', 'Проверка кода (clippy)');
            case 'dev/format-code':
                return await this.runCommand('cargo fmt', 'Форматирование кода');
            case 'dev/check-binaries':
                return await this.runCommand('npm run check:binaries', 'Проверка бинарных файлов');
            case 'dev/project-info':
                return await this.showProjectInfo();
                
            // 📋 Git операции (8 функций)
            case 'git/git-status':
                return await this.runCommand('git status', 'Git статус');
            case 'git/smart-commit':
                return await this.smartCommit();
            case 'git/commit-push':
                return await this.commitAndPush();
            case 'git/git-dev':
                return await this.runCommand('npm run git:dev', 'Dev workflow');
            case 'git/git-release':
                return await this.runCommand('npm run git:release minor', 'Release workflow');
            case 'git/git-commit':
                return await this.runCommand('npm run git:commit', 'Простой коммит');
            case 'git/git-version':
                return await this.runCommand('npm run git:version', 'Version workflow');
            case 'git/git-log':
                return await this.runCommand('git log --oneline -10', 'Последние 10 коммитов');
                
            // 🚀 Публикация (7 функций)
            case 'publish/package-extension':
                return await this.runCommand('npm run package:extension', 'Упаковка расширения');
            case 'publish/publish-marketplace':
                return await this.runCommand('npm run publish:marketplace', 'Публикация в VS Code Marketplace');
            case 'publish/publish-github':
                return await this.runCommand('npm run publish:github', 'Публикация на GitHub');
            case 'publish/publish-check':
                return await this.runCommand('npm run publish:check', 'Проверка публикации');
            case 'publish/clean-packages':
                return await this.runCommand('npm run clean:old-packages', 'Очистка старых пакетов');
            case 'publish/copy-binaries':
                return await this.runCommand('npm run copy:binaries:release', 'Копирование release бинарников');
            case 'publish/build-versioned':
                return await this.buildVersioned();
                
            // ⚙️ Утилиты и диагностика (5 функций)
            case 'utils/cleanup-project':
                return await this.runCommand('npm run cleanup:project', 'Очистка проекта');
            case 'utils/deep-cleanup':
                return await this.runCommand('npm run deep-cleanup', 'Глубокая очистка');
            case 'utils/watch-install':
                return await this.runCommand('npm run watch:install', 'Установка watch зависимостей');
            case 'utils/cargo-clean':
                return await this.runCommand('cargo clean', 'Очистка Cargo cache');
            case 'utils/show-logs':
                return await this.showErrorLogs();
                
            default:
                console.log(c('yellow', '⚠️ Функция будет реализована в Этапе 2'));
                break;
        }
        
        // Пауза для всех действий
        await prompts({
            type: 'text',
            name: 'continue',
            message: 'Нажмите Enter для продолжения...',
            initial: ''
        });
    }
    
    /**
     * Умный коммит (базовая реализация)
     */
    async smartCommit() {
        console.log(c('magenta', '\n📝 Умный коммит...'));
        
        // Показать статус
        await this.runCommand('git status --porcelain', 'Проверка изменений');
        
        const response = await prompts({
            type: 'text',
            name: 'message',
            message: 'Введите сообщение коммита:',
            validate: value => value.trim().length > 0 ? true : 'Сообщение не может быть пустым'
        });
        
        if (!response.message) return false;
        
        await this.runCommand('git add .', 'Добавление файлов');
        return await this.runCommand(`git commit -m "${response.message}"`, 'Создание коммита');
    }
    
    /**
     * Коммит и пуш (базовая реализация)
     */
    async commitAndPush() {
        console.log(c('magenta', '\n📤 Коммит и отправка...'));
        
        await this.runCommand('git status --porcelain', 'Проверка изменений');
        
        const response = await prompts({
            type: 'text',
            name: 'message',
            message: 'Введите сообщение коммита:',
            validate: value => value.trim().length > 0 ? true : 'Сообщение не может быть пустым'
        });
        
        if (!response.message) return false;
        
        await this.runCommand('git add .', 'Добавление файлов');
        await this.runCommand(`git commit -m "${response.message}"`, 'Создание коммита');
        return await this.runCommand('git push', 'Отправка в репозиторий');
    }

    /**
     * Watch режим с файловым мониторингом
     */
    async startWatchMode() {
        console.log(c('cyan', '\n👁️ Запуск Watch режима...'));
        
        // Проверка chokidar
        try {
            require('chokidar');
        } catch (error) {
            console.log(c('yellow', '⚠️ Зависимость chokidar не найдена'));
            
            const installResponse = await prompts({
                type: 'confirm',
                name: 'install',
                message: 'Установить chokidar автоматически?',
                initial: true
            });
            
            if (!installResponse.install) {
                console.log(c('red', '❌ Watch режим требует chokidar'));
                return false;
            }
            
            await this.runCommand('npm install --save-dev chokidar', 'Установка chokidar');
        }
        
        console.log(c('green', '✅ Запуск watch режима...'));
        console.log(c('gray', 'Нажмите Ctrl+C для остановки'));
        
        const { spawn } = require('child_process');
        const child = spawn('npm', ['run', 'watch'], {
            stdio: 'inherit',
            shell: true
        });
        
        this.currentProcess = child;
        
        return new Promise((resolve) => {
            child.on('close', (code) => {
                this.currentProcess = null;
                console.log(c('yellow', '\n👁️ Watch режим остановлен'));
                resolve(code === 0);
            });
        });
    }

    /**
     * Полная очистка и пересборка
     */
    async cleanAndRebuild() {
        console.log(c('yellow', '\n🧹 Полная очистка и пересборка...'));
        
        const confirmResponse = await prompts({
            type: 'confirm',
            name: 'confirm',
            message: '⚠️ Это займет несколько минут. Продолжить?',
            initial: false
        });
        
        if (!confirmResponse.confirm) {
            console.log(c('yellow', '⏸️ Операция отменена'));
            return false;
        }
        
        console.log(c('gray', 'Выполняется полная очистка и пересборка...\n'));
        
        await this.runCommand('cargo clean', 'Очистка Cargo');
        await this.runCommand('npm run cleanup:project', 'Очистка проекта');
        return await this.runCommand('npm run build:release', 'Полная пересборка');
    }

    /**
     * Показать детальную информацию о проекте
     */
    async showProjectInfo() {
        console.log(c('cyan', '\n📊 Детальная информация о проекте:'));
        console.log('='.repeat(60));
        
        try {
            const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
            console.log(c('bright', `📦 Название: ${packageJson.name}`));
            console.log(c('bright', `🔢 Версия: ${packageJson.version}`));
            console.log(c('bright', `📝 Описание: ${packageJson.description || 'Не указано'}`));
            
            // Cargo.toml
            if (fs.existsSync('Cargo.toml')) {
                const cargoToml = fs.readFileSync('Cargo.toml', 'utf8');
                const versionMatch = cargoToml.match(/version\s*=\s*"([^"]+)"/);
                if (versionMatch) {
                    console.log(c('bright', `🦀 Cargo версия: ${versionMatch[1]}`));
                }
            }
            
            // Git информация
            try {
                const branch = execSync('git branch --show-current', { encoding: 'utf8' }).trim();
                const commits = execSync('git rev-list --count HEAD', { encoding: 'utf8' }).trim();
                const lastCommit = execSync('git log -1 --format="%h %s"', { encoding: 'utf8' }).trim();
                console.log(c('bright', `🌿 Ветка: ${branch}`));
                console.log(c('bright', `📊 Коммитов: ${commits}`));
                console.log(c('bright', `📝 Последний: ${lastCommit}`));
            } catch (error) {
                console.log(c('gray', '📊 Git информация недоступна'));
            }
            
            // Проверка зависимостей
            const deps = packageJson.dependencies || {};
            const devDeps = packageJson.devDependencies || {};
            console.log(c('bright', `📚 Зависимости: ${Object.keys(deps).length}`));
            console.log(c('bright', `🔧 Dev зависимости: ${Object.keys(devDeps).length}`));
            
            // Проверка VSCode расширения
            const extensionPath = 'vscode-extension/package.json';
            if (fs.existsSync(extensionPath)) {
                const extPackage = JSON.parse(fs.readFileSync(extensionPath, 'utf8'));
                console.log(c('bright', `📱 VSCode расширение: v${extPackage.version}`));
            }
            
            // Проверка кеша
            const cacheExists = fs.existsSync('.build-cache');
            console.log(c(cacheExists ? 'green' : 'gray', `💾 Build cache: ${cacheExists ? 'активен' : 'отсутствует'}`));
            
            // Проверка chokidar для watch
            try {
                require('chokidar');
                console.log(c('green', '👁️ Watch зависимость: установлена'));
            } catch (error) {
                console.log(c('red', '👁️ Watch зависимость: не установлена'));
            }
            
        } catch (error) {
            console.log(c('red', `❌ Ошибка чтения информации: ${error.message}`));
        }
        
        return true;
    }

    /**
     * Сборка с выбором версии
     */
    async buildVersioned() {
        console.log(c('cyan', '\n🏗️ Сборка с версионированием'));
        
        const versionResponse = await prompts({
            type: 'select',
            name: 'version',
            message: 'Выберите тип версии для сборки:',
            choices: [
                { title: '🔸 Patch (x.x.X)', value: 'patch', description: 'Исправления и мелкие изменения' },
                { title: '🔹 Minor (x.X.x)', value: 'minor', description: 'Новые функции с обратной совместимостью' },
                { title: '🔺 Major (X.x.x)', value: 'major', description: 'Крупные изменения, могут нарушить совместимость' }
            ],
            initial: 0
        });
        
        if (!versionResponse.version) return false;
        
        const command = `npm run build:${versionResponse.version}`;
        return await this.runCommand(command, `Сборка с ${versionResponse.version} версией`);
    }

    /**
     * Показать логи ошибок
     */
    async showErrorLogs() {
        console.log(c('cyan', '\n📄 Логи ошибок консоли'));
        console.log('='.repeat(60));
        
        const logPath = path.join(process.cwd(), '.dev-console-errors.log');
        
        if (!fs.existsSync(logPath)) {
            console.log(c('green', '✅ Лог файл отсутствует - ошибок не было'));
            return true;
        }
        
        try {
            const logContent = fs.readFileSync(logPath, 'utf8');
            const lines = logContent.trim().split('\n');
            
            console.log(c('yellow', `📊 Найдено записей: ${lines.length}`));
            console.log();
            
            // Показываем последние 10 записей
            const recentLines = lines.slice(-10);
            recentLines.forEach((line, index) => {
                try {
                    const entry = JSON.parse(line);
                    const timestamp = new Date(entry.timestamp).toLocaleString();
                    console.log(c('red', `[${timestamp}]`));
                    console.log(c('gray', `Команда: ${entry.command}`));
                    console.log(c('yellow', `Ошибка: ${entry.error}`));
                    if (index < recentLines.length - 1) console.log();
                } catch (parseError) {
                    console.log(c('gray', line));
                }
            });
            
            if (lines.length > 10) {
                console.log(c('gray', `\n... и еще ${lines.length - 10} записей`));
            }
            
            // Предложить очистить лог
            const clearResponse = await prompts({
                type: 'confirm',
                name: 'clear',
                message: 'Очистить лог файл?',
                initial: false
            });
            
            if (clearResponse.clear) {
                fs.writeFileSync(logPath, '');
                console.log(c('green', '✅ Лог файл очищен'));
            }
            
        } catch (error) {
            console.log(c('red', `❌ Ошибка чтения лога: ${error.message}`));
        }
        
        return true;
    }

    /**
     * Выполнение системной команды
     */
    async runCommand(command, description = '') {
        console.log(c('cyan', `\n🔧 ${description || command}`));
        console.log('='.repeat(60));
        
        try {
            const startTime = Date.now();
            execSync(command, { 
                cwd: process.cwd(),
                stdio: 'inherit',
                encoding: 'utf8'
            });
            const duration = ((Date.now() - startTime) / 1000).toFixed(1);
            console.log('='.repeat(60));
            console.log(c('green', `✅ Команда выполнена успешно за ${duration}s`));
            return true;
        } catch (error) {
            console.log('='.repeat(60));
            console.log(c('red', `❌ Ошибка: ${error.message}`));
            
            if (this.config.logErrors) {
                this.logError(command, error);
            }
            
            return false;
        }
    }

    /**
     * Логирование ошибок
     */
    logError(command, error) {
        const logEntry = {
            timestamp: new Date().toISOString(),
            command: command,
            error: error.message,
            code: error.code
        };
        
        try {
            const logPath = path.join(process.cwd(), '.dev-console-errors.log');
            fs.appendFileSync(logPath, JSON.stringify(logEntry) + '\n');
        } catch (logError) {
            // Молча игнорируем ошибки логирования
        }
    }

    /**
     * Получение информации о проекте
     */
    getProjectInfo() {
        try {
            const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
            const hasCache = fs.existsSync('.build-cache');
            let hasChanges = false;
            
            try {
                const result = execSync('git status --porcelain', { encoding: 'utf8' });
                hasChanges = result.trim().length > 0;
            } catch {
                // Git недоступен
            }
            
            return {
                name: packageJson.name,
                version: packageJson.version,
                description: packageJson.description,
                hasCache,
                hasChanges
            };
        } catch (error) {
            return {
                name: 'unknown',
                version: 'unknown',
                description: 'Проект не найден',
                hasCache: false,
                hasChanges: false
            };
        }
    }

    /**
     * Проверка доступности команды
     */
    isCommandAvailable(command) {
        try {
            execSync(`${command} --version`, { stdio: 'ignore' });
            return true;
        } catch {
            return false;
        }
    }

    /**
     * Запуск консоли
     */
    async start() {
        console.log(c('bright', '🎯 Инициализация Universal Dev Console v2.0...'));
        
        // Показываем информацию о проекте
        const info = this.getProjectInfo();
        console.log(c('green', `📦 Проект: ${info.name} v${info.version}`));
        
        if (info.hasChanges) {
            console.log(c('yellow', '⚠️ Есть незакоммиченные изменения'));
        }
        
        console.log(c('blue', '🚀 Загружается главное меню...\n'));
        
        // Небольшая пауза для читаемости
        await new Promise(resolve => setTimeout(resolve, 1000));
        
        // Запускаем главное меню
        await this.showMainMenu();
    }
}

// Запуск приложения
const console_app = new UniversalDevConsole();

console_app.start().catch(error => {
    console.error(c('red', `❌ Критическая ошибка: ${error.message}`));
    process.exit(1);
});