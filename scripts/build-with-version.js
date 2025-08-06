#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');

console.log('🚀 BSL Analyzer Build with Auto-Versioning');
console.log('='.repeat(60));

// Получаем тип версии из аргументов
const args = process.argv.slice(2);
const versionType = args[0] || 'patch';
const skipVersioning = args.includes('--skip-version');

function runCommand(command, description) {
    console.log(`\n📋 ${description}...`);
    try {
        const output = execSync(command, { stdio: 'inherit', encoding: 'utf8' });
        console.log(`✅ ${description} completed`);
        return true;
    } catch (error) {
        console.error(`❌ ${description} failed:`, error.message);
        return false;
    }
}

// Основная логика сборки
async function buildWithVersioning() {
    let success = true;
    
    // 1. Обновляем версии (если не пропускаем)
    if (!skipVersioning) {
        console.log('🔄 Step 1: Version synchronization');
        if (!runCommand(`node scripts/version-sync.js ${versionType}`, 'Version synchronization')) {
            return false;
        }
    } else {
        console.log('⏭️  Step 1: Skipping version update (--skip-version)');
    }
    
    // 2. Собираем Rust проект
    console.log('\n🦀 Step 2: Building Rust project');
    if (!runCommand('cargo build --release --jobs 4', 'Rust build')) {
        return false;
    }
    
    // 3. Пересобираем расширение
    console.log('\n📦 Step 3: Rebuilding VSCode extension');
    if (!runCommand('npm run rebuild:extension', 'Extension rebuild')) {
        return false;
    }
    
    // 4. Показываем финальную информацию
    console.log('\n' + '='.repeat(60));
    console.log('🎉 BUILD COMPLETED SUCCESSFULLY!');
    console.log('='.repeat(60));
    
    // Читаем финальную версию
    try {
        const packageJson = JSON.parse(fs.readFileSync('package.json', 'utf8'));
        const version = packageJson.version;
        
        console.log(`📊 Project Version: ${version}`);
        console.log(`📁 Extension Location: vscode-extension/dist/bsl-analyzer-${version}.vsix`);
        
        if (!skipVersioning) {
            console.log('\n💡 Next steps:');
            console.log(`   1. Test the extension: Install bsl-analyzer-${version}.vsix`);
            console.log(`   2. Commit changes: git add . && git commit -m "build: version ${version}"`);
            console.log(`   3. Create tag: git tag v${version}`);
        }
        
    } catch (error) {
        console.error('⚠️  Could not read final version:', error.message);
    }
    
    return true;
}

// Проверяем валидность типа версии
if (!skipVersioning && !['major', 'minor', 'patch'].includes(versionType)) {
    console.error('❌ Invalid version type. Use: major, minor, patch');
    console.error('   Or use --skip-version to build without version update');
    process.exit(1);
}

// Запускаем сборку
buildWithVersioning().then(success => {
    process.exit(success ? 0 : 1);
}).catch(error => {
    console.error('❌ Build failed:', error);
    process.exit(1);
});