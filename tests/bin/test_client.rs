use std::process::Stdio;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Testing MCP server with Rust client...");

    // Запускаем сервер как дочерний процесс
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "bsl-mcp-server"])
        .env("BSL_CONFIG_PATH", "examples\\ConfTest")
        .env("BSL_PLATFORM_VERSION", "8.3.25")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    // Даем серверу время запуститься
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    eprintln!("Server should be running. Check if tools are available...");

    // Здесь можно было бы отправить JSON-RPC запросы, но это требует полной реализации клиента
    // Для простоты просто убиваем процесс

    child.kill().await?;

    Ok(())
}
