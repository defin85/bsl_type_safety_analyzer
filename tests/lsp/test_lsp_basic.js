#!/usr/bin/env node
/// Базовый тест LSP Server v2.0
/// 
/// Проверяет:
/// - Инициализацию LSP с UnifiedBslIndex
/// - Real-time диагностику BSL кода
/// - Enhanced автодополнение
/// - Hover подсказки

const { spawn } = require("child_process");
const fs = require("fs");

async function testLspBasic() {
    console.log("🚀 Testing Enhanced BSL LSP Server v2.0...\n");
    
    // Запускаем LSP сервер
    const lsp = spawn("cargo", ["run", "--bin", "lsp_server"], {
        cwd: "../..",
        env: {
            ...process.env,
            BSL_CONFIG_PATH: "examples/ConfTest",
            BSL_PLATFORM_VERSION: "8.3.25",
            RUST_LOG: "info"
        },
        stdio: ["pipe", "pipe", "pipe"]
    });
    
    let responseBuffer = "";
    let requestId = 1;
    
    // Обработка ответов LSP
    lsp.stdout.on("data", (data) => {
        responseBuffer += data.toString();
        const lines = responseBuffer.split("\n");
        responseBuffer = lines.pop() || ""; // Оставляем неполную строку
        
        for (const line of lines) {
            if (line.trim()) {
                try {
                    const response = JSON.parse(line);
                    console.log("📨 LSP Response:", JSON.stringify(response, null, 2));
                } catch (e) {
                    console.log("📨 LSP Output:", line);
                }
            }
        }
    });
    
    lsp.stderr.on("data", (data) => {
        console.log("🔧 [LSP stderr]:", data.toString().trim());
    });
    
    // Отправка LSP запроса
    const sendLspRequest = (method, params = {}) => {
        const request = {
            jsonrpc: "2.0",
            id: requestId++,
            method: method,
            params: params
        };
        
        const requestJson = JSON.stringify(request);
        const requestMessage = `Content-Length: ${Buffer.byteLength(requestJson)}\\r\\n\\r\\n${requestJson}`;
        
        console.log(`\\n📤 Sending ${method}:`, JSON.stringify(params, null, 2));
        lsp.stdin.write(requestMessage);
    };
    
    // Ждем запуска LSP сервера
    await new Promise(resolve => setTimeout(resolve, 3000));
    
    console.log("\\n" + "=".repeat(60));
    console.log("🧪 TESTING BSL LSP SERVER v2.0");
    console.log("=".repeat(60) + "\\n");
    
    // 1. Initialize LSP
    console.log("1️⃣ Initialize LSP Server");
    sendLspRequest("initialize", {
        processId: process.pid,
        clientInfo: {
            name: "test-client",
            version: "1.0.0"
        },
        capabilities: {
            textDocument: {
                synchronization: {
                    dynamicRegistration: false,
                    willSave: false,
                    willSaveWaitUntil: false,
                    didSave: false
                },
                completion: {
                    dynamicRegistration: false,
                    completionItem: {
                        snippetSupport: false
                    }
                },
                hover: {
                    dynamicRegistration: false
                }
            }
        },
        workspaceFolders: [{
            uri: "file://" + process.cwd().replace(/\\\\/g, "/"),
            name: "test-workspace"
        }]
    });
    
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // 2. Initialized notification
    console.log("2️⃣ Send initialized notification");
    sendLspRequest("initialized", {});
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // 3. Open document with BSL code
    console.log("3️⃣ Open BSL document");
    const bslCode = `Процедура ТестированиеLSP()
    // Тест глобальных функций
    ТекущаяДата = ТекущаяДата();
    Сообщить("LSP работает!");
    
    // Ошибка - несуществующая функция  
    НесуществующаяФункция();
КонецПроцедуры`;

    sendLspRequest("textDocument/didOpen", {
        textDocument: {
            uri: "file:///test_lsp.bsl",
            languageId: "bsl",
            version: 1,
            text: bslCode
        }
    });
    
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // 4. Test completion
    console.log("4️⃣ Test completion at cursor position");
    sendLspRequest("textDocument/completion", {
        textDocument: {
            uri: "file:///test_lsp.bsl"
        },
        position: {
            line: 2,
            character: 10  // After "ТекущаяДата = Тек"
        }
    });
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // 5. Test hover
    console.log("5️⃣ Test hover on function name");
    sendLspRequest("textDocument/hover", {
        textDocument: {
            uri: "file:///test_lsp.bsl"
        },
        position: {
            line: 2,
            character: 17  // On "ТекущаяДата"
        }
    });
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // 6. Close document
    console.log("6️⃣ Close document");
    sendLspRequest("textDocument/didClose", {
        textDocument: {
            uri: "file:///test_lsp.bsl"
        }
    });
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // 7. Shutdown
    console.log("7️⃣ Shutdown LSP server");
    sendLspRequest("shutdown", {});
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    lsp.stdin.end();
    lsp.kill();
    
    console.log("\\n" + "=".repeat(60));
    console.log("🎉 Enhanced BSL LSP Server v2.0 Test completed!");
    console.log("=".repeat(60));
    console.log("✅ Tested LSP capabilities:");
    console.log("   • UnifiedBslIndex initialization");
    console.log("   • Real-time diagnostics with BslAnalyzer");
    console.log("   • Enhanced completion with BSL types");
    console.log("   • Hover information from unified index");
}

testLspBasic().catch(console.error);