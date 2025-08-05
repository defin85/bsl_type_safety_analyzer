#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('🚀 BSL Analyzer Extension Rebuild Tool v1.0');
console.log('='.repeat(50));

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
        name: 'Moving package to root',
        command: 'mv vscode-extension/bsl-analyzer-*.vsix . 2>/dev/null || true',
        icon: '🔄'
    },
    {
        name: 'Cleaning old packages',
        command: 'find . -name "bsl-analyzer-*.vsix" ! -name "bsl-analyzer-1.3.1.vsix" -delete 2>/dev/null || true',
        icon: '🧹'
    }
];

let success = true;

for (let i = 0; i < steps.length; i++) {
    const step = steps[i];
    console.log(`\n${step.icon} [${i + 1}/${steps.length}] ${step.name}...`);
    
    try {
        execSync(step.command, { 
            stdio: 'inherit', 
            cwd: process.cwd(),
            shell: true 
        });
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
        const stats = fs.statSync('bsl-analyzer-1.3.1.vsix');
        const fileSizeInMB = (stats.size / (1024 * 1024)).toFixed(1);
        console.log(`📊 Package size: ${fileSizeInMB} MB`);
    } catch (e) {
        console.log('📊 Package created');
    }
    
    console.log('\n📋 To install:');
    console.log('   1. Press Ctrl+Shift+P in VS Code');
    console.log('   2. Type: Extensions: Install from VSIX');
    console.log('   3. Select: bsl-analyzer-1.3.1.vsix');
} else {
    console.log('\n' + '='.repeat(50));
    console.log('💥 FAILED: Extension rebuild failed');
    console.log('='.repeat(50));
    process.exit(1);
}