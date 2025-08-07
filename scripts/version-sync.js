#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

console.log('🔄 BSL Analyzer Version Sync Tool');
console.log('='.repeat(50));

// Пути к файлам
const cargoTomlPath = 'Cargo.toml';
const extensionPackageJsonPath = path.join('vscode-extension', 'package.json');
const rootPackageJsonPath = 'package.json';

// Чтение текущих версий
function readVersion(filePath, versionPattern) {
    try {
        const content = fs.readFileSync(filePath, 'utf8');
        const match = content.match(versionPattern);
        return match ? match[1] : null;
    } catch (error) {
        console.error(`❌ Ошибка чтения ${filePath}:`, error.message);
        return null;
    }
}

// Запись новой версии
function writeVersion(filePath, content, oldVersion, newVersion, versionPattern) {
    try {
        const newContent = content.replace(versionPattern, (match, version) => {
            return match.replace(version, newVersion);
        });
        fs.writeFileSync(filePath, newContent, 'utf8');
        return true;
    } catch (error) {
        console.error(`❌ Ошибка записи ${filePath}:`, error.message);
        return false;
    }
}

// Увеличение версии (семантическое версионирование)
function incrementVersion(version, type = 'patch') {
    const parts = version.split('.').map(Number);
    
    switch (type) {
        case 'major':
            parts[0]++;
            parts[1] = 0;
            parts[2] = 0;
            break;
        case 'minor':
            parts[1]++;
            parts[2] = 0;
            break;
        case 'patch':
        default:
            parts[2]++;
            break;
    }
    
    return parts.join('.');
}

// Основная логика
function syncVersions(incrementType = null) {
    console.log(`📊 Текущие версии:`);
    
    // Чтение версий
    const cargoContent = fs.readFileSync(cargoTomlPath, 'utf8');
    const cargoVersion = readVersion(cargoTomlPath, /version\s*=\s*"([^"]+)"/);
    
    const extensionContent = fs.readFileSync(extensionPackageJsonPath, 'utf8');
    const extensionVersion = readVersion(extensionPackageJsonPath, /"version"\s*:\s*"([^"]+)"/);
    
    const rootContent = fs.readFileSync(rootPackageJsonPath, 'utf8');
    const rootVersion = readVersion(rootPackageJsonPath, /"version"\s*:\s*"([^"]+)"/);
    
    console.log(`   Cargo.toml: ${cargoVersion}`);
    console.log(`   Extension:  ${extensionVersion}`);
    console.log(`   Root:       ${rootVersion}`);
    
    // Определяем базовую версию (берем максимальную)
    const versions = [cargoVersion, extensionVersion, rootVersion].filter(v => v);
    if (versions.length === 0) {
        console.error('❌ Не удалось прочитать ни одну версию');
        process.exit(1);
    }
    
    // Находим максимальную версию
    const maxVersion = versions.reduce((max, current) => {
        const maxParts = max.split('.').map(Number);
        const currentParts = current.split('.').map(Number);
        
        for (let i = 0; i < 3; i++) {
            if (currentParts[i] > maxParts[i]) return current;
            if (currentParts[i] < maxParts[i]) return max;
        }
        return max;
    });
    
    // Увеличиваем версию или оставляем как есть
    let newVersion;
    if (incrementType) {
        newVersion = incrementVersion(maxVersion, incrementType);
        console.log(`\n🆙 Новая версия: ${newVersion} (${incrementType})`);
    } else {
        newVersion = maxVersion;
        console.log(`\n🔄 Синхронизация версий: ${newVersion}`);
    }
    
    // Обновляем файлы
    let updated = 0;
    
    console.log('\n🔄 Обновление файлов:');
    
    // Cargo.toml
    if (writeVersion(cargoTomlPath, cargoContent, cargoVersion, newVersion, /version\s*=\s*"([^"]+)"/)) {
        console.log(`   ✅ Cargo.toml: ${cargoVersion} → ${newVersion}`);
        updated++;
    }
    
    // Extension package.json
    if (writeVersion(extensionPackageJsonPath, extensionContent, extensionVersion, newVersion, /"version"\s*:\s*"([^"]+)"/)) {
        console.log(`   ✅ Extension: ${extensionVersion} → ${newVersion}`);
        updated++;
    }
    
    // Root package.json
    if (writeVersion(rootPackageJsonPath, rootContent, rootVersion, newVersion, /"version"\s*:\s*"([^"]+)"/)) {
        console.log(`   ✅ Root: ${rootVersion} → ${newVersion}`);
        updated++;
    }
    
    console.log(`\n🎉 Успешно обновлено файлов: ${updated}/3`);
    console.log(`📦 Единая версия проекта: ${newVersion}`);
    
    return newVersion;
}

// CLI интерфейс
const args = process.argv.slice(2);
const incrementType = args[0] || null; // null означает только синхронизацию

if (incrementType && !['major', 'minor', 'patch'].includes(incrementType)) {
    console.error('❌ Неверный тип версии. Используйте: major, minor, patch');
    process.exit(1);
}

const newVersion = syncVersions(incrementType);

console.log('\n💡 Рекомендуемые следующие шаги:');
console.log('   1. npm run build:rust:release  (автоматически использует все CPU)');
console.log('   2. npm run rebuild:extension');
console.log(`   3. git add . && git commit -m "bump: version ${newVersion}"`);