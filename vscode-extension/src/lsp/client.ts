import * as vscode from 'vscode';
import { 
    TypeInfoParams, 
    ValidateMethodParams, 
    IndexingProgressParams
} from '../types';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
    RevealOutputChannelOn,
    Trace,
    ExecutableOptions
} from 'vscode-languageclient/node';
import { getBinaryPath } from '../utils/binaryPath';
import { BslAnalyzerConfig } from '../config/configHelper';
import * as fs from 'fs';

let client: LanguageClient | null = null;
let outputChannel: vscode.OutputChannel;

/**
 * Инициализирует модуль LSP клиента
 */
export function initializeLspClient(channel: vscode.OutputChannel) {
    outputChannel = channel;
}

/**
 * Запускает LSP сервер
 */
export async function startLanguageClient(context: vscode.ExtensionContext): Promise<void> {
    const serverMode = BslAnalyzerConfig.serverMode;
    const tcpPort = BslAnalyzerConfig.serverTcpPort;
    const traceLevel = BslAnalyzerConfig.serverTrace;
    
    // Используем getBinaryPath для получения пути к LSP серверу
    let serverPath: string;
    try {
        // Всегда используем общую логику выбора бинарников
        serverPath = getBinaryPath('lsp_server', context);
        outputChannel.appendLine(`🚀 LSP server path resolved: ${serverPath}`);
    } catch (error: unknown) {
        const errorMessage = error instanceof Error ? error.message : String(error);
        outputChannel.appendLine(`❌ Failed to locate LSP server: ${errorMessage}`);
        vscode.window.showWarningMessage(
            'BSL Analyzer: LSP server not found. Extension features will be limited.',
            'Show Details'
        ).then(selection => {
            if (selection === 'Show Details') {
                outputChannel.show();
            }
        });
        return;
    }
    
    // Проверяем существование файла
    if (!fs.existsSync(serverPath)) {
        outputChannel.appendLine(`❌ LSP server file not found: ${serverPath}`);
        vscode.window.showWarningMessage(
            'BSL Analyzer: LSP server binary not found. Please build the project first.',
            'Open Build Instructions'
        ).then(selection => {
            if (selection === 'Open Build Instructions') {
                vscode.env.openExternal(vscode.Uri.parse('https://github.com/bsl-analyzer-team/bsl-type-safety-analyzer#building'));
            }
        });
        return;
    }
    
    outputChannel.appendLine(`🔧 Starting LSP server in ${serverMode} mode...`);
    outputChannel.appendLine(`📍 Server path: ${serverPath}`);
    
    // Server options configuration
    let serverOptions: ServerOptions;
    
    if (serverMode === 'stdio') {
        // STDIO mode - запускаем сервер как процесс
        const execOptions: ExecutableOptions = {
            env: { 
                ...process.env,
                RUST_LOG: 'info', // Use 'info' as default log level
                RUST_BACKTRACE: '1'
            }
        };
        
        serverOptions = {
            run: {
                command: serverPath,
                args: ['lsp'],
                options: execOptions
            },
            debug: {
                command: serverPath,
                args: ['lsp', '--debug'],
                options: execOptions
            }
        };
    } else {
        // TCP mode - подключаемся к серверу
        outputChannel.appendLine(`📡 Connecting to LSP server on port ${tcpPort}...`);
        serverOptions = {
            run: {
                transport: TransportKind.socket,
                port: tcpPort
            } as any,
            debug: {
                transport: TransportKind.socket,
                port: tcpPort
            } as any
        };
    }
    
    // Client options configuration
    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'bsl' },
            { scheme: 'untitled', language: 'bsl' }
        ],
        synchronize: {
            fileEvents: [
                vscode.workspace.createFileSystemWatcher('**/*.bsl'),
                vscode.workspace.createFileSystemWatcher('**/*.os'),
                vscode.workspace.createFileSystemWatcher('**/Configuration.xml')
            ],
            configurationSection: 'bslAnalyzer'
        },
        outputChannel: outputChannel,
        revealOutputChannelOn: RevealOutputChannelOn.Never,
        traceOutputChannel: outputChannel,
        middleware: {
            // Перехватываем workspace-related notifications
            workspace: {
                configuration: (params, token, next) => {
                    outputChannel.appendLine(`📊 Configuration request: ${JSON.stringify(params)}`);
                    return next(params, token);
                }
            }
        }
    };
    
    // Устанавливаем уровень трассировки
    if (traceLevel && traceLevel !== 'off') {
        // Convert string to Trace enum
        if (traceLevel === 'messages') {
            (clientOptions as any).trace = Trace.Messages;
        } else if (traceLevel === 'verbose') {
            (clientOptions as any).trace = Trace.Verbose;
        }
    }
    
    // Create the language client
    client = new LanguageClient(
        'bslAnalyzer',
        'BSL Type Safety Analyzer',
        serverOptions,
        clientOptions
    );
    
    // Start the client
    try {
        outputChannel.appendLine('🚀 Starting LSP client...');
        await client.start();
        outputChannel.appendLine('✅ LSP client started successfully');
        
        // Регистрируем обработчики custom requests
        registerCustomHandlers();
        
        // Регистрируем обработчик прогресса индексации
        client.onNotification('bsl/indexingProgress', (params: IndexingProgressParams) => {
            handleIndexingProgress(params);
        });
        
    } catch (error) {
        outputChannel.appendLine(`❌ Failed to start LSP client: ${error}`);
        vscode.window.showErrorMessage(`Failed to start BSL Analyzer: ${error}`);
    }
}

/**
 * Останавливает LSP сервер
 */
export async function stopLanguageClient(): Promise<void> {
    if (client) {
        outputChannel.appendLine('🛑 Stopping LSP client...');
        try {
            await client.stop();
            outputChannel.appendLine('✅ LSP client stopped');
        } catch (error) {
            outputChannel.appendLine(`⚠️ Error stopping LSP client: ${error}`);
        }
        client = null;
    }
}

/**
 * Перезапускает LSP сервер
 */
export async function restartLanguageClient(context: vscode.ExtensionContext): Promise<void> {
    outputChannel.appendLine('🔄 Restarting LSP server...');
    await stopLanguageClient();
    // Небольшая задержка перед перезапуском
    await new Promise(resolve => setTimeout(resolve, 500));
    await startLanguageClient(context);
}

/**
 * Возвращает текущий клиент LSP
 */
export function getLanguageClient(): LanguageClient | null {
    return client;
}

/**
 * Проверяет, запущен ли LSP клиент
 */
export function isClientRunning(): boolean {
    return client !== null && client.isRunning();
}

/**
 * Регистрирует обработчики кастомных запросов
 */
function registerCustomHandlers() {
    if (!client) return;
    
    // Обработчик запросов информации о типе
    client.onRequest('bsl/typeInfo', async (params: TypeInfoParams) => {
        outputChannel.appendLine(`📋 Type info request: ${JSON.stringify(params)}`);
        // Здесь можно добавить обработку запроса
        return null;
    });
    
    // Обработчик запросов валидации метода
    client.onRequest('bsl/validateMethod', async (params: ValidateMethodParams) => {
        outputChannel.appendLine(`✓ Method validation request: ${JSON.stringify(params)}`);
        // Здесь можно добавить обработку запроса
        return null;
    });
}

/**
 * Обработчик прогресса индексации от сервера
 */
function handleIndexingProgress(params: IndexingProgressParams) {
    outputChannel.appendLine(`📊 Indexing progress: Step ${params.step}/${params.totalSteps} - ${params.message} (${params.percentage}%)`);
    
    // Здесь можно обновить UI с прогрессом
    // Например, вызвать event emitter для обновления status bar
}

/**
 * Отправляет запрос на сервер для выполнения кастомной команды
 */
export async function sendCustomRequest<T = unknown>(method: string, params?: unknown): Promise<T> {
    if (!client || !client.isRunning()) {
        throw new Error('LSP client is not running');
    }
    
    try {
        const result = await client.sendRequest(method, params);
        return result as T;
    } catch (error) {
        outputChannel.appendLine(`❌ Custom request failed: ${error}`);
        throw error;
    }
}

/**
 * Отправляет уведомление на сервер
 */
export function sendCustomNotification(method: string, params?: unknown): void {
    if (!client || !client.isRunning()) {
        outputChannel.appendLine(`⚠️ Cannot send notification: LSP client is not running`);
        return;
    }
    
    client.sendNotification(method, params);
}