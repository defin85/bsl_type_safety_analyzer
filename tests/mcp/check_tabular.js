#!/usr/bin/env node
const { spawn } = require("child_process");
const readline = require("readline");

async function checkTabularSections() {
    const server = spawn("cargo", ["run", "--bin", "bsl-mcp-server"], {
        cwd: "../..",
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
    
    rl.on("line", (line) => {
        try {
            const response = JSON.parse(line);
            if (response.id === 3 && response.result) {
                const content = JSON.parse(response.result.content[0].text);
                if (content.found && content.entity.tabular_sections) {
                    console.log("\nТабличные части справочника Контрагенты (через MCP):");
                    content.entity.tabular_sections.forEach(ts => {
                        console.log(`  - ${ts.name} (${ts.display_name}) - ${ts.attributes_count} атрибутов`);
                    });
                    
                    console.log("\nПолучено через MCP инструмент find_type!");
                    server.kill();
                    process.exit(0);
                }
            }
        } catch (e) {}
    });
    
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Initialize
    server.stdin.write(JSON.stringify({
        jsonrpc: "2.0",
        method: "initialize",
        params: { protocolVersion: "0.1.0", clientInfo: { name: "test", version: "1.0.0" } },
        id: 1
    }) + "\n");
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Find type
    console.log("Запрашиваем тип через MCP...");
    server.stdin.write(JSON.stringify({
        jsonrpc: "2.0",
        method: "tools/call",
        params: { name: "find_type", arguments: { type_name: "Справочники.Контрагенты" } },
        id: 3
    }) + "\n");
    
    await new Promise(resolve => setTimeout(resolve, 5000));
    server.kill();
}

checkTabularSections().catch(console.error);