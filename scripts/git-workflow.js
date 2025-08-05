#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');

console.log('🔄 BSL Analyzer Git Workflow Tool');
console.log('='.repeat(50));

// Получаем параметры командной строки
const args = process.argv.slice(2);
const command = args[0];
const versionType = args[1] || 'patch';

function runCommand(cmd, description) {
    console.log(`\n📋 ${description}...`);
    try {
        execSync(cmd, { stdio: 'inherit' });
        console.log(`✅ ${description} completed`);
        return true;
    } catch (error) {
        console.error(`❌ ${description} failed`);
        return false;
    }
}

function getCurrentVersion() {
    try {
        const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
        return pkg.version;
    } catch (error) {
        return 'unknown';
    }
}

// Различные рабочие потоки
const workflows = {
    // Полный релиз с версионированием
    'release': async () => {
        console.log(`🚀 Starting release workflow (${versionType})`);
        
        const oldVersion = getCurrentVersion();
        console.log(`📊 Current version: ${oldVersion}`);
        
        // 1. Версионирование и сборка
        if (!runCommand(`npm run build:${versionType}`, 'Build with version increment')) {
            return false;
        }
        
        const newVersion = getCurrentVersion();
        console.log(`🆙 New version: ${newVersion}`);
        
        // 2. Git операции
        if (!runCommand('git add .', 'Stage all changes')) {
            return false;
        }
        
        if (!runCommand(`git commit -m "build: release version ${newVersion}"`, 'Commit release')) {
            return false;
        }
        
        if (!runCommand(`git tag v${newVersion}`, 'Create version tag')) {
            return false;
        }
        
        console.log('\n🎉 Release workflow completed!');
        console.log(`📦 Version: ${oldVersion} → ${newVersion}`);
        console.log(`🏷️  Tag: v${newVersion}`);
        console.log(`📁 Extension: vscode-extension/dist/bsl-analyzer-${newVersion}.vsix`);
        
        return true;
    },
    
    // Быстрая разработка без версионирования
    'dev': async () => {
        console.log('⚡ Starting development workflow');
        
        if (!runCommand('npm run build:release', 'Build without version increment')) {
            return false;
        }
        
        console.log('\n✅ Development build completed!');
        console.log('💡 Use "npm run git:release" when ready to release');
        
        return true;
    },
    
    // Только версионирование без сборки
    'version': async () => {
        console.log(`📊 Version increment workflow (${versionType})`);
        
        const oldVersion = getCurrentVersion();
        
        if (!runCommand(`npm run version:${versionType}`, 'Version increment')) {
            return false;
        }
        
        const newVersion = getCurrentVersion();
        
        console.log(`\n✅ Version updated: ${oldVersion} → ${newVersion}`);
        console.log('💡 Run "npm run build:release" to build with new version');
        
        return true;
    },
    
    // Commit с автоматическим определением типа версии
    'commit': async () => {
        const message = args[1] || 'update: changes';
        console.log(`📝 Smart commit workflow: "${message}"`);
        
        // Анализируем сообщение коммита для определения типа версии
        let autoVersionType = null;
        
        if (message.match(/^(feat|feature):/i)) {
            autoVersionType = 'minor';
        } else if (message.match(/^(fix|bugfix):/i)) {
            autoVersionType = 'patch';
        } else if (message.match(/^(major|breaking):/i)) {
            autoVersionType = 'major';
        }
        
        if (autoVersionType) {
            console.log(`🤖 Auto-detected version type: ${autoVersionType}`);
            
            const oldVersion = getCurrentVersion();
            
            if (!runCommand(`npm run build:${autoVersionType}`, 'Build with auto-version')) {
                return false;
            }
            
            const newVersion = getCurrentVersion();
            
            if (!runCommand('git add .', 'Stage changes')) {
                return false;
            }
            
            if (!runCommand(`git commit -m "${message} (v${newVersion})"`, 'Commit with version')) {
                return false;
            }
            
            console.log(`\n🎉 Smart commit completed: ${oldVersion} → ${newVersion}`);
        } else {
            // Обычный коммит без версионирования
            if (!runCommand('git add .', 'Stage changes')) {
                return false;
            }
            
            if (!runCommand(`git commit -m "${message}"`, 'Regular commit')) {
                return false;
            }
            
            console.log('\n✅ Regular commit completed');
            console.log('💡 Tip: Use "feat:", "fix:", or "major:" prefixes for auto-versioning');
        }
        
        return true;
    }
};

// Основная логика
async function main() {
    if (!command || !workflows[command]) {
        console.log('❌ Invalid command. Available workflows:');
        console.log('');
        console.log('  release [patch|minor|major]  - Full release with versioning');
        console.log('  dev                          - Development build without versioning');
        console.log('  version [patch|minor|major]  - Version increment only');
        console.log('  commit "message"             - Smart commit with auto-versioning');
        console.log('');
        console.log('Examples:');
        console.log('  npm run git:release minor');
        console.log('  npm run git:dev');
        console.log('  npm run git:commit "feat: add new feature"');
        console.log('  npm run git:version patch');
        process.exit(1);
    }
    
    const success = await workflows[command]();
    process.exit(success ? 0 : 1);
}

main().catch(error => {
    console.error('❌ Workflow failed:', error);
    process.exit(1);
});