#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

console.log('🧹 BSL Analyzer Project Cleanup Tool');
console.log('='.repeat(50));

// Files and directories to remove
const itemsToRemove = [
    // Temporary analysis files
    'temp_archive_analysis.md',
    'temp_detailed_parser_validation.md', 
    'temp_final_parser_analysis.md',
    'temp_parser_verification_results.md',
    'temp_html_analysis/',
    'temp_shlang_extract/',
    
    // Output directories
    'output/',
    'test_output/',
    
    // Test files in root
    'test_activation.bsl',
    'test_perf.bsl', 
    'test_real_project.bsl',
    'test_config/',
    
    // Obsolete documentation
    'ACTIVATION_GUIDE.md',
    'INSTALLATION_GUIDE.md',
    'STANDALONE_EXTENSION_GUIDE.md', 
    'ICON_QUALITY_GUIDE.md',
    'MONETIZATION_STRATEGY.md',
    
    // Duplicate scripts (we have better ones now)
    'rebuild_extension.bat',
    'rebuild_extension.sh',
    
    // Old Python scripts
    'create_logo_png.py',
    
    // Log files
    'output.log',
    
    // Old roadmap (we have CHANGELOG.md)
    'roadmap.md'
];

let removedCount = 0;
let failedCount = 0;

for (const item of itemsToRemove) {
    const fullPath = path.resolve(item);
    
    try {
        if (fs.existsSync(fullPath)) {
            const stats = fs.statSync(fullPath);
            
            if (stats.isDirectory()) {
                // Remove directory recursively
                fs.rmSync(fullPath, { recursive: true, force: true });
                console.log(`🗂️  Removed directory: ${item}`);
            } else {
                // Remove file
                fs.unlinkSync(fullPath);
                console.log(`📄 Removed file: ${item}`);
            }
            removedCount++;
        } else {
            console.log(`⚠️  Not found: ${item}`);
        }
    } catch (error) {
        console.error(`❌ Failed to remove ${item}:`, error.message);
        failedCount++;
    }
}

console.log('\n' + '='.repeat(50));
console.log(`🎉 Cleanup completed!`);
console.log(`✅ Removed: ${removedCount} items`);
if (failedCount > 0) {
    console.log(`❌ Failed: ${failedCount} items`);
}

console.log('\n📋 Remaining structure:');
console.log('   ├── Core project files (Cargo.toml, README.md, etc.)');
console.log('   ├── src/ - Rust source code');
console.log('   ├── docs/ - Project documentation');
console.log('   ├── examples/ - Test configurations');
console.log('   ├── tests/ - Unit and integration tests');
console.log('   ├── vscode-extension/ - VSCode extension');
console.log('   ├── scripts/ - Automation scripts');
console.log('   └── .husky/ - Git hooks');

console.log('\n💡 Next steps:');
console.log('   1. Review git status: git status');
console.log('   2. Commit cleanup: git add . && git commit -m "cleanup: remove temporary and obsolete files"');
console.log('   3. Test build: npm run rebuild:extension');