#!/usr/bin/env node
const { spawn } = require("child_process");
const readline = require("readline");

async function testEnhancedMcpServer() {
    console.log("🚀 Testing Enhanced BSL MCP Server...\n");
    
    const server = spawn("cargo", ["run", "--bin", "bsl-mcp-server"], {
        cwd: "../..",  // Возвращаемся в корень проекта
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
    
    // Handle server responses
    rl.on("line", (line) => {
        console.log("📨 Response:", line);
        try {
            const response = JSON.parse(line);
            console.log("✅ Parsed:", JSON.stringify(response, null, 2));
        } catch (e) {
            // Not JSON, maybe debug output
        }
    });
    
    // Handle server errors
    server.stderr.on("data", (data) => {
        console.error("🔧 [Server]", data.toString());
    });
    
    server.on("error", (err) => {
        console.error("❌ Failed to start server:", err);
    });
    
    // Send requests
    const sendRequest = (request) => {
        console.log(`\n📤 Sending: ${JSON.stringify(request, null, 2)}`);
        server.stdin.write(JSON.stringify(request) + "\n");
    };
    
    // Wait a bit for server to start
    await new Promise(resolve => setTimeout(resolve, 3000));
    
    console.log("\n" + "=".repeat(60));
    console.log("🧪 TESTING ENHANCED MCP PROTOCOL");
    console.log("=".repeat(60) + "\n");
    
    // 1. Initialize
    console.log("1️⃣ Initialize Protocol");
    sendRequest({
        jsonrpc: "2.0",
        method: "initialize",
        params: {
            protocolVersion: "0.1.0",
            clientInfo: {
                name: "enhanced-test-client",
                version: "1.0.0"
            }
        },
        id: 1
    });
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // 2. List tools (should show 4 tools now!)
    console.log("2️⃣ List Available Tools");
    sendRequest({
        jsonrpc: "2.0",
        method: "tools/list",
        params: {},
        id: 2
    });
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // 3. Test find_type tool
    console.log("3️⃣ Test find_type Tool");
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
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // 4. Test NEW analyze_code tool with Base64 example
    console.log("4️⃣ Test NEW analyze_code Tool");
    const testBslCode = `Процедура ТестированиеBase64()
    Данные = "Тестовые данные";
    КодированнаяСтрока = Base64Значение(Данные);  // ✅ Должна найтись
    Сообщить(КодированнаяСтрока);
    
    // Ошибка - несуществующая функция
    НеправильнаяФункция();  // ❌ Должна выдать ошибку
КонецПроцедуры`;

    sendRequest({
        jsonrpc: "2.0",
        method: "tools/call",
        params: {
            name: "analyze_code",
            arguments: {
                bsl_code: testBslCode,
                file_name: "test_base64.bsl"
            }
        },
        id: 4
    });
    
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // 5. Test NEW validate_syntax tool
    console.log("5️⃣ Test NEW validate_syntax Tool");
    sendRequest({
        jsonrpc: "2.0",
        method: "tools/call",
        params: {
            name: "validate_syntax",
            arguments: {
                bsl_code: "Процедура Тест() КонецПроцедуры"
            }
        },
        id: 5
    });
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // 6. Test NEW get_suggestions tool
    console.log("6️⃣ Test NEW get_suggestions Tool");
    sendRequest({
        jsonrpc: "2.0",
        method: "tools/call",
        params: {
            name: "get_suggestions",
            arguments: {
                error_code: "BSL003",
                context: "НеправильнаяФункция() не найдена"
            }
        },
        id: 6
    });
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // 7. Test syntax validation with invalid code
    console.log("7️⃣ Test Syntax Validation with Invalid Code");
    sendRequest({
        jsonrpc: "2.0",
        method: "tools/call",
        params: {
            name: "validate_syntax",
            arguments: {
                bsl_code: "Процедура Тест( // неправильный синтаксис"
            }
        },
        id: 7
    });
    
    // Wait for all responses
    await new Promise(resolve => setTimeout(resolve, 3000));
    
    // Close server
    server.stdin.end();
    server.kill();
    
    console.log("\n" + "=".repeat(60));
    console.log("🎉 Enhanced MCP Server Test completed!");
    console.log("=".repeat(60));
    console.log("✅ Tested 4 BSL MCP Tools:");
    console.log("   • find_type - поиск типов в UnifiedBslIndex");
    console.log("   • analyze_code - полный анализ BSL кода");  
    console.log("   • validate_syntax - проверка синтаксиса");
    console.log("   • get_suggestions - предложения по исправлению");
}

testEnhancedMcpServer().catch(console.error);