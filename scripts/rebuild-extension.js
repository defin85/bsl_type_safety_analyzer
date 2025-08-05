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

const steps = [
    {
        name: 'Building Rust binaries',
        command: 'cargo build --release',
        icon: '🦀'
    },
    {
        name: 'Copying binaries to extension',
        command: 'cp target/release/*.exe vscode-extension/bin/ 2>/dev/null || cp target/release/lsp_server target/release/bsl-analyzer target/release/build_unified_index target/release/query_type vscode-extension/bin/',
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
            const vsixFiles = glob.sync('vscode-extension/bsl-analyzer-*.vsix');
            for (const file of vsixFiles) {
                const filename = path.basename(file);
                const newPath = path.join(distDir, filename);
                fs.renameSync(file, newPath);
                console.log(`   Moved ${filename} to dist/`);
            }
            
            // Clean old packages (keep only latest)
            const distFiles = glob.sync(path.join(distDir, 'bsl-analyzer-*.vsix'));
            const latestFile = path.join(distDir, 'bsl-analyzer-1.3.1.vsix');
            
            for (const file of distFiles) {
                if (file !== latestFile) {
                    fs.unlinkSync(file);
                    console.log(`   Removed old package: ${path.basename(file)}`);
                }
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
    
    // Check file size
    try {
        const stats = fs.statSync('vscode-extension/dist/bsl-analyzer-1.3.1.vsix');
        const fileSizeInMB = (stats.size / (1024 * 1024)).toFixed(1);
        console.log(`📊 Package size: ${fileSizeInMB} MB`);
        console.log(`📁 Location: vscode-extension/dist/bsl-analyzer-1.3.1.vsix`);
    } catch (e) {
        console.log('📊 Package created in vscode-extension/dist/');
    }
    
    console.log('\n📋 To install:');
    console.log('   1. Press Ctrl+Shift+P in VS Code');
    console.log('   2. Type: Extensions: Install from VSIX');
    console.log('   3. Select: vscode-extension/dist/bsl-analyzer-1.3.1.vsix');
} else {
    console.log('\n' + '='.repeat(50));
    console.log('💥 FAILED: Extension rebuild failed');
    console.log('='.repeat(50));
    process.exit(1);
}