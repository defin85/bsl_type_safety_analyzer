#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

console.log('🔥 BSL Analyzer Deep Cleanup Tool');
console.log('='.repeat(60));

// Categorize binaries in src/bin/
const binaries = {
    core: [
        'mcp_server.rs',           // MCP server for LLM integration
        'build_unified_index.rs',  // Main index builder
        'query_type.rs',          // Type query tool
        'syntaxcheck.rs',         // Syntax checker
        'extract_platform_docs.rs', // Platform docs extractor
        'extract_hybrid_docs.rs',  // Hybrid docs extractor  
        'incremental_update.rs',   // Index updates
        'check_type_compatibility.rs' // Type compatibility
    ],
    utility: [
        'analyze_objects.rs',      // Object analysis utility
        'analyze_real_report.rs',  // Report analyzer
        'demo_context_analysis.rs' // Demo tool
    ],
    test_debug: [
        'debug_local_functions.rs',
        'test_advanced_semantic.rs',
        'test_analyzer.rs', 
        'test_auto_keywords.rs',
        'test_base64_analysis.rs',
        'test_cli_integration.rs',
        'test_client.rs',
        'test_direct_base64.rs',
        'test_full_integration.rs',
        'test_metadata_parser.rs',
        'test_method_validation.rs',
        'test_method_verifier.rs',
        'test_semantic.rs'
    ]
};

console.log('📊 Binary Analysis:');
console.log(`   Core binaries: ${binaries.core.length}`);
console.log(`   Utility binaries: ${binaries.utility.length}`);
console.log(`   Test/Debug binaries: ${binaries.test_debug.length}`);

const srcBinPath = path.join('src', 'bin');
const testBinPath = path.join('tests', 'bin');

// Create tests/bin directory if it doesn't exist
if (!fs.existsSync('tests')) {
    fs.mkdirSync('tests');
}
if (!fs.existsSync(testBinPath)) {
    fs.mkdirSync(testBinPath);
}

let movedCount = 0;
let errors = [];

console.log('\n🔄 Moving test/debug binaries to tests/bin/:');

// Move test and debug binaries
for (const binary of binaries.test_debug) {
    const srcPath = path.join(srcBinPath, binary);
    const destPath = path.join(testBinPath, binary);
    
    try {
        if (fs.existsSync(srcPath)) {
            fs.renameSync(srcPath, destPath);
            console.log(`   ✅ Moved: ${binary}`);
            movedCount++;
        } else {
            console.log(`   ⚠️  Not found: ${binary}`);
        }
    } catch (error) {
        console.error(`   ❌ Failed to move ${binary}:`, error.message);
        errors.push(binary);
    }
}

console.log('\n📋 Remaining in src/bin/:');
try {
    const remainingFiles = fs.readdirSync(srcBinPath).filter(f => f.endsWith('.rs'));
    const coreRemaining = remainingFiles.filter(f => binaries.core.includes(f));
    const utilityRemaining = remainingFiles.filter(f => binaries.utility.includes(f));
    const unknownRemaining = remainingFiles.filter(f => 
        !binaries.core.includes(f) && 
        !binaries.utility.includes(f) &&
        !binaries.test_debug.includes(f)
    );
    
    console.log(`   ✅ Core: ${coreRemaining.length} files`);
    coreRemaining.forEach(f => console.log(`      - ${f}`));
    
    console.log(`   🔧 Utility: ${utilityRemaining.length} files`);
    utilityRemaining.forEach(f => console.log(`      - ${f}`));
    
    if (unknownRemaining.length > 0) {
        console.log(`   ❓ Unknown: ${unknownRemaining.length} files`);
        unknownRemaining.forEach(f => console.log(`      - ${f}`));
    }
} catch (error) {
    console.error('Failed to read src/bin directory:', error.message);
}

// Clean up target directory (if exists)
console.log('\n🧹 Target directory cleanup:');
try {
    const targetPath = 'target';
    if (fs.existsSync(targetPath)) {
        const targetStats = fs.statSync(targetPath);
        if (targetStats.isDirectory()) {
            // Get size info
            const debugPath = path.join(targetPath, 'debug');
            if (fs.existsSync(debugPath)) {
                const debugFiles = fs.readdirSync(debugPath, { recursive: true });
                console.log(`   📁 target/debug contains ${debugFiles.length} files`);
                console.log(`   💡 This directory is in .gitignore (correct)`);
            }
            
            const releasePath = path.join(targetPath, 'release');
            if (fs.existsSync(releasePath)) {
                const releaseFiles = fs.readdirSync(releasePath);
                console.log(`   📦 target/release contains ${releaseFiles.length} files`);
                console.log(`   💡 These are the production binaries`);
            }
        }
    }
} catch (error) {
    console.error('Failed to analyze target directory:', error.message);
}

// Check tests directory structure
console.log('\n🧪 Tests directory structure:');
try {
    const testsPath = 'tests';
    if (fs.existsSync(testsPath)) {
        const testFiles = fs.readdirSync(testsPath, { withFileTypes: true });
        const dirs = testFiles.filter(f => f.isDirectory()).map(f => f.name);
        const files = testFiles.filter(f => f.isFile()).map(f => f.name);
        
        console.log(`   📁 Directories: ${dirs.join(', ')}`);
        console.log(`   📄 Test files: ${files.length}`);
        
        // Check if bin directory was created
        if (dirs.includes('bin')) {
            const binFiles = fs.readdirSync(path.join(testsPath, 'bin'));
            console.log(`   🔧 tests/bin/ now contains: ${binFiles.length} test binaries`);
        }
    }
} catch (error) {
    console.error('Failed to analyze tests directory:', error.message);
}

console.log('\n' + '='.repeat(60));
console.log('🎉 Deep cleanup completed!');
console.log(`✅ Moved ${movedCount} test/debug binaries to tests/bin/`);

if (errors.length > 0) {
    console.log(`❌ Errors: ${errors.length}`);
    errors.forEach(e => console.log(`   - ${e}`));
}

console.log('\n📋 Result:');
console.log('   ├── src/bin/ - Only core and utility binaries');
console.log('   ├── tests/bin/ - All test and debug binaries');
console.log('   ├── target/ - Build artifacts (in .gitignore)');
console.log('   └── Clean project structure');

console.log('\n💡 Next steps:');
console.log('   1. Update Cargo.toml [[bin]] sections if needed');
console.log('   2. Run: cargo build --release');
console.log('   3. Test: npm run rebuild:extension');
console.log('   4. Commit: git add . && git commit -m "refactor: organize binaries structure"');