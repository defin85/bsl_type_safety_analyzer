#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

console.log('🎯 Creating Minimal BSL Analyzer Extension');
console.log('='.repeat(50));

// Минимальный набор бинарников для расширения
const essentialBinaries = [
    'bsl-analyzer.exe',         // Основной анализатор (7.2MB)
    'lsp_server.exe',          // LSP сервер (5.7MB)  
    'build_unified_index.exe', // Построение индекса (2.2MB)
    'query_type.exe',          // Поиск типов (обычно небольшой)
    'syntaxcheck.exe'          // Проверка синтаксиса (обычно небольшой)
];

const sourceBinDir = path.join('target', 'release');
const targetBinDir = path.join('vscode-extension', 'bin');
const backupBinDir = path.join('vscode-extension', 'bin-full');

function createMinimalExtension() {
    // 1. Создать резервную копию всех бинарников
    if (!fs.existsSync(backupBinDir)) {
        console.log('📁 Creating backup of full binaries...');
        fs.mkdirSync(backupBinDir, { recursive: true });
        
        const allFiles = fs.readdirSync(targetBinDir);
        for (const file of allFiles) {
            if (file.endsWith('.exe')) {
                fs.copyFileSync(
                    path.join(targetBinDir, file),
                    path.join(backupBinDir, file)
                );
            }
        }
        console.log(`✅ Backed up ${allFiles.filter(f => f.endsWith('.exe')).length} binaries`);
    }
    
    // 2. Очистить текущую bin директорию
    console.log('🧹 Clearing current bin directory...');
    const currentFiles = fs.readdirSync(targetBinDir);
    for (const file of currentFiles) {
        if (file.endsWith('.exe')) {
            fs.unlinkSync(path.join(targetBinDir, file));
        }
    }
    
    // 3. Скопировать только необходимые бинарники
    console.log('📦 Copying essential binaries...');
    let totalSize = 0;
    let copiedCount = 0;
    
    for (const binary of essentialBinaries) {
        const sourcePath = path.join(sourceBinDir, binary);
        const targetPath = path.join(targetBinDir, binary);
        
        if (fs.existsSync(sourcePath)) {
            fs.copyFileSync(sourcePath, targetPath);
            const stats = fs.statSync(targetPath);
            const sizeMB = (stats.size / 1024 / 1024).toFixed(1);
            console.log(`   ✅ ${binary} (${sizeMB} MB)`);
            totalSize += stats.size;
            copiedCount++;
        } else {
            console.log(`   ⚠️  ${binary} not found`);
        }
    }
    
    const totalSizeMB = (totalSize / 1024 / 1024).toFixed(1);
    console.log(`\n📊 Result:`);
    console.log(`   Binaries: ${copiedCount}/${essentialBinaries.length}`);
    console.log(`   Total size: ${totalSizeMB} MB (was ~47 MB)`);
    console.log(`   Size reduction: ${((47 - totalSizeMB) / 47 * 100).toFixed(0)}%`);
    
    // 4. Создать README для пользователей
    const readmeContent = `# BSL Analyzer - Essential Binaries

This extension includes essential BSL Analyzer binaries:

## Included Tools:
- **bsl-analyzer.exe** - Main static analyzer
- **lsp_server.exe** - Language Server Protocol support  
- **build_unified_index.exe** - Type index builder
- **query_type.exe** - Type information queries
- **syntaxcheck.exe** - Syntax validation

## Full Tool Suite:
For additional tools (27 binaries total), download from:
https://github.com/your-org/bsl-analyzer/releases

## Usage:
The extension will automatically detect and use bundled binaries.
No additional configuration required.
`;

    fs.writeFileSync(path.join(targetBinDir, 'README.md'), readmeContent);
    
    console.log('\n💡 To restore full binaries:');
    console.log('   npm run restore:full-binaries');
    console.log('\n🚀 Ready to build minimal extension:');
    console.log('   npm run rebuild:extension');
}

function restoreFullBinaries() {
    console.log('🔄 Restoring full binaries...');
    
    if (!fs.existsSync(backupBinDir)) {
        console.error('❌ No backup found. Run create-minimal first.');
        return;
    }
    
    // Копировать все файлы из backup обратно
    const backupFiles = fs.readdirSync(backupBinDir);
    let restoredCount = 0;
    
    for (const file of backupFiles) {
        if (file.endsWith('.exe')) {
            fs.copyFileSync(
                path.join(backupBinDir, file),
                path.join(targetBinDir, file)
            );
            restoredCount++;
        }
    }
    
    console.log(`✅ Restored ${restoredCount} binaries`);
    console.log('🚀 Ready to build full extension:');
    console.log('   npm run rebuild:extension');
}

// CLI interface
const command = process.argv[2];

if (command === 'restore') {
    restoreFullBinaries();
} else {
    createMinimalExtension();
}