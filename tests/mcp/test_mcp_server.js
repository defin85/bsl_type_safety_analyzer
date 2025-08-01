#!/usr/bin/env node
const { spawn } = require("child_process");
const readline = require("readline");

async function testMcpServer() {
    console.log("Testing BSL MCP Server...\n");
    
    const server = spawn("cargo", ["run", "--bin", "bsl-mcp-server"], {
        cwd: "../..",  // Возвращаемся в корень проекта
        env: {
            ...process.env,
            BSL_CONFIG_PATH: "C:\\1CProject\\Unicom\\conf_files",
            BSL_PLATFORM_VERSION: "8.3.25"
        }
    });
    
    const rl = readline.createInterface({
        input: server.stdout,
        crlfDelay: Infinity
    });
    
    // Handle server responses
    rl.on("line", (line) => {
        console.log("Response:", line);
        try {
            const response = JSON.parse(line);
            console.log("Parsed:", JSON.stringify(response, null, 2));
        } catch (e) {
            // Not JSON
        }
    });
    
    // Handle server errors
    server.stderr.on("data", (data) => {
        console.error("[Server]", data.toString());
    });
    
    server.on("error", (err) => {
        console.error("Failed to start server:", err);
    });
    
    // Send requests
    const sendRequest = (request) => {
        console.log("\nSending:", JSON.stringify(request));
        server.stdin.write(JSON.stringify(request) + "\n");
    };
    
    // Wait a bit for server to start
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Test sequence
    console.log("\n=== Testing MCP Protocol ===\n");
    
    // 1. Initialize
    sendRequest({
        jsonrpc: "2.0",
        method: "initialize",
        params: {
            protocolVersion: "0.1.0",
            clientInfo: {
                name: "test-client",
                version: "1.0.0"
            }
        },
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
    
    // 3. Call find_type tool
    sendRequest({
        jsonrpc: "2.0",
        method: "tools/call",
        params: {
            name: "find_type",
            arguments: {
                type_name: "Массив"
            }
        },
        id: 3
    });
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // 4. Try to find configuration type - Контрагенты
    sendRequest({
        jsonrpc: "2.0",
        method: "tools/call",
        params: {
            name: "find_type",
            arguments: {
                type_name: "Справочники.Контрагенты"
            }
        },
        id: 4
    });
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // 5. Try to find document type - РеализацияТоваровУслуг
    sendRequest({
        jsonrpc: "2.0",
        method: "tools/call",
        params: {
            name: "find_type",
            arguments: {
                type_name: "Документы.РеализацияТоваровУслуг"
            }
        },
        id: 5
    });
    
    // Wait for all responses
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Close server
    server.stdin.end();
    server.kill();
    
    console.log("\nTest completed\!");
}

testMcpServer().catch(console.error);
