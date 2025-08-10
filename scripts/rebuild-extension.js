#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('🚀 BSL Analyzer Extension Rebuild Tool v1.0');
console.log('='.repeat(50));

// Проверяем синхронизацию версий
function checkVersionSync() {
    try {
        // Читаем версии
        const cargoContent = fs.readFileSync('Cargo.toml', 'utf8');
        const cargoVersion = cargoContent.match(/version\s*=\s*"([^"]+)"/)?.[1];

        const extensionPackage = JSON.parse(fs.readFileSync(path.join('vscode-extension', 'package.json'), 'utf8'));
        const extensionVersion = extensionPackage.version;

        const rootPackage = JSON.parse(fs.readFileSync('package.json', 'utf8'));
        const rootVersion = rootPackage.version;

        if (cargoVersion !== extensionVersion || extensionVersion !== rootVersion) {
            console.log('⚠️  Version mismatch detected:');
            console.log(`   Cargo.toml: ${cargoVersion}`);
            console.log(`   Extension:  ${extensionVersion}`);
            console.log(`   Root:       ${rootVersion}`);
            console.log('💡 Run: npm run version:sync to fix');
            console.log('');
        } else {
            console.log(`✅ All versions synchronized: ${extensionVersion}`);
        }
    } catch (error) {
        console.log('⚠️  Could not check version sync:', error.message);
    }
}

checkVersionSync();

// Определяем текущую версию из extension package
let currentVersion = 'unknown';
try {
    const extPkg = JSON.parse(fs.readFileSync(path.join('vscode-extension', 'package.json'), 'utf8'));
    currentVersion = extPkg.version;
} catch (_) { }

const steps = [
    {
        name: 'Building Rust (release)',
        command: 'npm run build:rust:release',
        icon: '🦀'
    },
    {
        name: 'Copying essential binaries to extension',
        command: 'node scripts/copy-essential-binaries.js release',
        icon: '📁'
    },
    {
        name: 'Compiling TypeScript',
        command: 'cd vscode-extension && npm run compile',
        icon: '📝'
    },
    {
        name: 'Packaging VSCode extension',
        command: 'cd vscode-extension && npx @vscode/vsce package',
        icon: '📦'
    },
    {
        name: 'Organizing output',
        command: null, // Special case - handled by Node.js
        icon: '📁'
    }
];

let success = true;

for (let i = 0; i < steps.length; i++) {
    const step = steps[i];
    console.log(`\n${step.icon} [${i + 1}/${steps.length}] ${step.name}...`);

    try {
        if (step.command === null) {
            // Special handling for file operations
            const glob = require('glob');

            // Create dist directory
            const distDir = path.join('vscode-extension', 'dist');
            if (!fs.existsSync(distDir)) {
                fs.mkdirSync(distDir, { recursive: true });
            }

            // Move .vsix files to dist
            const vsixFiles = glob.sync('vscode-extension/bsl-type-safety-analyzer-*.vsix');
            for (const file of vsixFiles) {
                const filename = path.basename(file);
                const newPath = path.join(distDir, filename);
                fs.renameSync(file, newPath);
                console.log(`   Moved ${filename} to dist/`);
            }

            // Оставляем только самый новый .vsix (по времени изменения)
            const distFiles = glob.sync(path.join(distDir, 'bsl-type-safety-analyzer-*.vsix'));
            if (distFiles.length > 1) {
                const sorted = distFiles.map(f => ({ f, m: fs.statSync(f).mtimeMs }))
                    .sort((a, b) => b.m - a.m);
                const keep = sorted[0].f;
                for (const entry of sorted.slice(1)) {
                    fs.unlinkSync(entry.f);
                    console.log(`   Removed old package: ${path.basename(entry.f)}`);
                }
                console.log(`   Keeping latest package: ${path.basename(keep)}`);
            }
        } else {
            execSync(step.command, {
                stdio: 'inherit',
                cwd: process.cwd(),
                shell: true
            });
        }
        console.log(`✅ ${step.name} completed`);
    } catch (error) {
        console.error(`❌ ${step.name} failed:`, error.message);
        success = false;
        break;
    }
}

if (success) {
    console.log('\n' + '='.repeat(50));
    console.log('🎉 SUCCESS: Extension rebuilt successfully!');
    console.log('='.repeat(50));
    const distDir = path.join('vscode-extension', 'dist');
    try {
        const vsixFiles = fs.readdirSync(distDir).filter(f => f.endsWith('.vsix'));
        if (vsixFiles.length) {
            const latest = vsixFiles.map(f => ({ f, m: fs.statSync(path.join(distDir, f)).mtimeMs }))
                .sort((a, b) => b.m - a.m)[0].f;
            const stats = fs.statSync(path.join(distDir, latest));
            const fileSizeInMB = (stats.size / (1024 * 1024)).toFixed(1);
            console.log(`📊 Package size: ${fileSizeInMB} MB`);
            console.log(`📁 Location: ${path.join(distDir, latest)}`);
            console.log('\n📋 To install:');
            console.log('   1. Ctrl+Shift+P');
            console.log('   2. Extensions: Install from VSIX');
            console.log(`   3. Select: ${path.join(distDir, latest)}`);
        } else {
            console.log('⚠️ No VSIX found in dist directory');
        }
    } catch (e) {
        console.log('📊 Package created (listing failed)');
    }
} else {
    console.log('\n' + '='.repeat(50));
    console.log('💥 FAILED: Extension rebuild failed');
    console.log('='.repeat(50));
    process.exit(1);
}