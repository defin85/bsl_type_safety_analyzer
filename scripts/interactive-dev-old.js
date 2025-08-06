#!/usr/bin/env node

const readline = require('readline');
const { execSync, spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

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

function c(color, text) {
    return `${colors[color] || ''}${text}${colors.reset}`;
}

class SimpleInteractiveDev {
    constructor() {
        // Один единственный readline на весь скрипт
        this.rl = readline.createInterface({
            input: process.stdin,
            output: process.stdout
        });
        this.currentProcess = null;
    }

    // Простой метод для вопросов
    async ask(question) {
        return new Promise(resolve => {
            this.rl.question(question, resolve);
        });
    }

    // Выполнение команды с выводом (без паузы)
    async runCommand(command, description = '') {
        console.log(c('cyan', `\n🔧 ${description || command}`));
        console.log('='.repeat(50));
        
        try {
            const startTime = Date.now();
            execSync(command, { 
                cwd: process.cwd(),
                stdio: 'inherit',
                encoding: 'utf8'
            });
            const duration = ((Date.now() - startTime) / 1000).toFixed(1);
            console.log('='.repeat(50));
            console.log(c('green', `✅ Команда выполнена успешно за ${duration}s`));
        } catch (error) {
            console.log('='.repeat(50));
            console.log(c('red', `❌ Ошибка: ${error.message}`));
        }
    }
    
    // Выполнение команды с паузой (для финальных команд)
    async runCommandWithPause(command, description = '') {
        await this.runCommand(command, description);
        await this.pause();
    }

    // Пауза для чтения результата
    async pause() {
        console.log(c('gray', '\nНажмите Enter для продолжения...'));
        await this.ask('');
    }

    // Главное меню
    showMainMenu() {
        console.clear();
        console.log(c('bright', '🚀 BSL Type Safety Analyzer - Interactive Dev Console v1.6.0'));
        console.log('');
        console.log(c('cyan', '📦 СБОРКА:'));
        console.log('  1) Быстрая dev сборка');
        console.log('  2) Smart сборка с кешированием');
        console.log('  3) Release сборка (полная)');
        console.log('  4) Watch режим (файловый мониторинг)');
        console.log('  5) Очистка и полная пересборка');
        console.log('');
        console.log(c('yellow', '🔧 РАЗРАБОТКА:'));
        console.log('  6) Запустить тесты');
        console.log('  7) Проверить код (clippy)');
        console.log('  8) Форматировать код');
        console.log('');
        console.log(c('magenta', '📋 GIT ОПЕРАЦИИ:'));
        console.log('  9) Git статус');
        console.log(' 10) Умный коммит');
        console.log(' 11) Коммит и пуш');
        console.log('');
        console.log(c('green', '🚀 ПУБЛИКАЦИЯ:'));
        console.log(' 12) Собрать расширение');
        console.log(' 13) Версия patch (x.x.X)');
        console.log(' 14) Версия minor (x.X.x)');
        console.log(' 15) Информация о проекте');
        console.log('');
        console.log(c('red', '  0) Выход'));
        console.log('');
    }

    // Главный цикл
    async run() {
        console.log(c('bright', '🎯 Инициализация интерактивной консоли разработки...'));
        
        while (true) {
            this.showMainMenu();
            
            const choice = await this.ask(c('bright', 'Выберите действие (0-15): '));
            
            switch (choice.trim()) {
                case '1':
                    await this.runCommandWithPause('npm run dev', 'Быстрая dev сборка');
                    break;
                    
                case '2':
                    await this.runCommandWithPause('npm run build:smart', 'Smart сборка с кешированием');
                    break;
                    
                case '3':
                    await this.runCommandWithPause('npm run build:release', 'Release сборка (полная)');
                    break;
                    
                case '4':
                    await this.startWatchMode();
                    break;
                    
                case '5':
                    await this.cleanAndRebuild();
                    break;
                    
                case '6':
                    await this.runCommandWithPause('cargo test', 'Запуск тестов');
                    break;
                    
                case '7':
                    await this.runCommandWithPause('cargo clippy', 'Проверка кода');
                    break;
                    
                case '8':
                    await this.runCommandWithPause('cargo fmt', 'Форматирование кода');
                    break;
                    
                case '9':
                    await this.runCommandWithPause('git status', 'Git статус');
                    break;
                    
                case '10':
                    await this.smartCommit();
                    break;
                    
                case '11':
                    await this.commitAndPush();
                    break;
                    
                case '12':
                    await this.runCommandWithPause('npm run rebuild:extension', 'Сборка расширения');
                    break;
                    
                case '13':
                    await this.runCommandWithPause('npm run version:patch', 'Увеличение patch версии');
                    break;
                    
                case '14':
                    await this.runCommandWithPause('npm run version:minor', 'Увеличение minor версии');
                    break;
                    
                case '15':
                    await this.showProjectInfo();
                    break;
                    
                case '0':
                case 'exit':
                case 'quit':
                    await this.exit();
                    return;
                    
                default:
                    console.log(c('red', '\n❌ Неверный выбор. Попробуйте снова.'));
                    await this.pause();
                    break;
            }
        }
    }

    // Watch режим
    async startWatchMode() {
        console.log(c('cyan', '\n👁️  Запуск Watch режима...'));
        
        // Проверка chokidar
        try {
            require('chokidar');
        } catch (error) {
            console.log(c('yellow', '⚠️  Зависимость chokidar не найдена'));
            console.log(c('cyan', '🔧 Автоматическая установка chokidar...'));
            
            await this.runCommand('npm install --save-dev chokidar', 'Установка chokidar');
            console.log(c('green', '✅ chokidar установлен'));
        }
        
        console.log(c('green', '✅ Запуск watch режима...'));
        console.log(c('gray', 'Нажмите Ctrl+C для остановки'));
        
        const child = spawn('npm', ['run', 'watch'], {
            stdio: 'inherit',
            shell: true
        });
        
        this.currentProcess = child;
        
        child.on('close', (code) => {
            this.currentProcess = null;
            console.log(c('yellow', '\n👁️  Watch режим остановлен'));
            // Возвращаемся в меню
        });
    }

    // Очистка и пересборка
    async cleanAndRebuild() {
        console.log(c('yellow', '\n🧹 Полная очистка и пересборка...'));
        console.log(c('gray', 'Это займет несколько минут...\n'));
        
        await this.runCommand('cargo clean', 'Очистка Cargo');
        await this.runCommand('npm run cleanup:project', 'Очистка проекта');
        await this.runCommandWithPause('npm run build:release', 'Полная пересборка');
    }

    // Умный коммит
    async smartCommit() {
        console.log(c('magenta', '\n📝 Умный коммит...'));
        
        // Показать статус
        try {
            execSync('git status --porcelain', { stdio: 'inherit' });
        } catch (error) {
            console.log(c('red', '❌ Ошибка получения статуса git'));
            await this.pause();
            return;
        }
        
        const message = await this.ask('Введите сообщение коммита: ');
        
        if (!message.trim()) {
            console.log(c('red', '❌ Пустое сообщение коммита'));
            await this.pause();
            return;
        }
        
        await this.runCommand('git add .', 'Добавление файлов');
        await this.runCommandWithPause(`git commit -m "${message}"`, 'Создание коммита');
    }

    // Коммит и пуш
    async commitAndPush() {
        console.log(c('magenta', '\n📝 Коммит и отправка в репозиторий...'));
        
        // Показать статус
        try {
            execSync('git status --porcelain', { stdio: 'inherit' });
        } catch (error) {
            console.log(c('red', '❌ Ошибка получения статуса git'));
            await this.pause();
            return;
        }
        
        const message = await this.ask('Введите сообщение коммита: ');
        
        if (!message.trim()) {
            console.log(c('red', '❌ Пустое сообщение коммита'));
            await this.pause();
            return;
        }
        
        await this.runCommand('git add .', 'Добавление файлов');
        await this.runCommand(`git commit -m "${message}"`, 'Создание коммита');
        await this.runCommandWithPause('git push', 'Отправка в удаленный репозиторий');
    }

    // Информация о проекте
    async showProjectInfo() {
        console.log(c('cyan', '\n📊 Информация о проекте:'));
        
        try {
            const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
            console.log(c('bright', `\n📦 Название: ${packageJson.name}`));
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
                console.log(c('bright', `🌿 Ветка: ${branch}`));
                console.log(c('bright', `📊 Коммитов: ${commits}`));
            } catch (error) {
                console.log(c('gray', '📊 Git информация недоступна'));
            }
            
            // Проверка chokidar
            try {
                require('chokidar');
                console.log(c('green', '👁️  Watch зависимость: установлена'));
            } catch (error) {
                console.log(c('red', '👁️  Watch зависимость: не установлена'));
            }
            
        } catch (error) {
            console.log(c('red', `❌ Ошибка чтения информации: ${error.message}`));
        }
        
        await this.pause();
    }

    // Выход
    async exit() {
        console.log(c('yellow', '\n👋 Завершение работы...'));
        this.cleanup();
        process.exit(0);
    }

    // Очистка при завершении
    cleanup() {
        if (this.currentProcess) {
            this.currentProcess.kill();
        }
        if (this.rl) {
            this.rl.close();
        }
    }
}

// Запуск
const app = new SimpleInteractiveDev();

// Обработка сигналов
process.on('SIGINT', () => {
    console.log(c('yellow', '\n\n🛑 Завершение работы...'));
    app.cleanup();
    process.exit(0);
});

process.on('SIGTERM', () => {
    app.cleanup();
    process.exit(0);
});

// Запуск приложения
app.run().catch(error => {
    console.error(c('red', `❌ Критическая ошибка: ${error.message}`));
    app.cleanup();
    process.exit(1);
});