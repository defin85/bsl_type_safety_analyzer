#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const BIN_DIR = path.join(__dirname, '..', 'vscode-extension', 'bin');
const REQUIRED_BINARIES = [
    'lsp_server.exe',
    'bsl-analyzer.exe', 
    'build_unified_index.exe',
    'query_type.exe'
];

console.log('🔍 BSL Analyzer Binary Checker');
console.log('='.repeat(40));

if (!fs.existsSync(BIN_DIR)) {
    console.error('❌ Binary directory not found:', BIN_DIR);
    console.log('\n💡 Run: npm run copy:binaries');
    process.exit(1);
}

const files = fs.readdirSync(BIN_DIR).filter(f => f.endsWith('.exe'));
console.log(`📁 Found ${files.length} executable files in bin/`);

let missingRequired = [];
let allPresent = true;

for (const binary of REQUIRED_BINARIES) {
    const exists = files.includes(binary);
    const status = exists ? '✅' : '❌';
    console.log(`${status} ${binary}`);
    
    if (!exists) {
        missingRequired.push(binary);
        allPresent = false;
    }
}

if (files.length > REQUIRED_BINARIES.length) {
    const extraCount = files.length - REQUIRED_BINARIES.length;
    console.log(`📎 Plus ${extraCount} additional utilities`);
}

console.log('\n' + '='.repeat(40));

if (allPresent) {
    console.log('🎉 All required binaries present!');
    
    // Calculate total size
    let totalSize = 0;
    for (const file of files) {
        const filePath = path.join(BIN_DIR, file);
        const stats = fs.statSync(filePath);
        totalSize += stats.size;
    }
    
    const sizeMB = (totalSize / (1024 * 1024)).toFixed(1);
    console.log(`📊 Total size: ${sizeMB} MB`);
    
    process.exit(0);
} else {
    console.log('💥 Missing required binaries:');
    missingRequired.forEach(binary => console.log(`   - ${binary}`));
    
    console.log('\n💡 To fix:');
    console.log('   1. npm run build:rust:release');
    console.log('   2. npm run copy:binaries:release');
    
    process.exit(1);
}