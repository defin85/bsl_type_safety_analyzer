#!/usr/bin/env node
const { spawn } = require("child_process");
const readline = require("readline");

async function testMcpQuick() {
    console.log("🚀 Quick Test: Enhanced BSL MCP Server\n");
    
    const server = spawn("cargo", ["run", "--bin", "bsl-mcp-server"], {
        cwd: "../..",
        env: {
            ...process.env,
            BSL_CONFIG_PATH: "examples/ConfTest",
            BSL_PLATFORM_VERSION: "8.3.25"
        }
    });
    
    const rl = readline.createInterface({
        input: server.stdout,
        crlfDelay: Infinity
    });
    
    let responseCount = 0;
    
    rl.on("line", (line) => {
        console.log("📨", line);
        try {
            const response = JSON.parse(line);
            if (response.result || response.error) {
                responseCount++;
                console.log("✅ Response", responseCount, ":", JSON.stringify(response, null, 2));
            }
        } catch (e) {
            // Not JSON
        }
    });
    
    server.stderr.on("data", (data) => {
        console.error("🔧", data.toString().trim());
    });
    
    const sendRequest = (request) => {
        console.log(`\n📤 ${request.method}:`, JSON.stringify(request));
        server.stdin.write(JSON.stringify(request) + "\n");
    };
    
    // Wait for server startup
    await new Promise(resolve => setTimeout(resolve, 3000));
    
    console.log("\n=== Testing 4 Enhanced MCP Tools ===\n");
    
    // 1. Initialize
    sendRequest({
        jsonrpc: "2.0",
        method: "initialize",
        params: { protocolVersion: "0.1.0" },
        id: 1
    });
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // 2. List tools
    sendRequest({
        jsonrpc: "2.0",
        method: "tools/list",
        params: {},
        id: 2
    });
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // 3. Test analyze_code
    sendRequest({
        jsonrpc: "2.0",
        method: "tools/call",
        params: {
            name: "analyze_code",
            arguments: {
                bsl_code: "Процедура Тест()\n    НесуществующаяФункция();\nКонецПроцедуры",
                file_name: "test.bsl"
            }
        },
        id: 3
    });
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Close
    server.stdin.end();
    server.kill();
    
    console.log("\n🎉 Quick test completed!");
}

testMcpQuick().catch(console.error);