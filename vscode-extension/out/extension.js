"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.deactivate = exports.activate = void 0;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const child_process_1 = require("child_process");
const node_1 = require("vscode-languageclient/node");
let client = null;
let indexServerPath;
let outputChannel;
let globalExtensionContext;
let globalIndexingProgress = {
    isIndexing: false,
    currentStep: 'Idle',
    progress: 0,
    totalSteps: 4,
    currentStepNumber: 0
};
// Event emitter для обновления прогресса
const progressEmitter = new vscode.EventEmitter();
// Функции управления прогрессом индексации
function startIndexing(totalSteps = 4) {
    globalIndexingProgress = {
        isIndexing: true,
        currentStep: 'Initializing...',
        progress: 0,
        totalSteps,
        currentStepNumber: 0,
        startTime: new Date()
    };
    updateStatusBar(undefined, globalIndexingProgress);
    progressEmitter.fire(globalIndexingProgress);
    outputChannel.appendLine(`🚀 Index building started with ${totalSteps} steps`);
}
function updateIndexingProgress(stepNumber, stepName, progress) {
    if (!globalIndexingProgress.isIndexing) {
        outputChannel.appendLine(`⚠️ updateIndexingProgress called but indexing is not active`);
        return;
    }
    const elapsed = globalIndexingProgress.startTime ?
        (new Date().getTime() - globalIndexingProgress.startTime.getTime()) / 1000 : 0;
    // Простая оценка времени: elapsed * (100 / progress) - elapsed
    const eta = progress > 5 ? Math.round((elapsed * (100 / progress)) - elapsed) : undefined;
    globalIndexingProgress = {
        ...globalIndexingProgress,
        currentStep: stepName,
        progress: Math.min(progress, 100),
        currentStepNumber: stepNumber,
        estimatedTimeRemaining: eta ? `${eta}s` : undefined
    };
    updateStatusBar(undefined, globalIndexingProgress);
    progressEmitter.fire(globalIndexingProgress);
    outputChannel.appendLine(`📊 Step ${stepNumber}/${globalIndexingProgress.totalSteps}: ${stepName} (${progress}%)`);
}
function finishIndexing(success = true) {
    const elapsed = globalIndexingProgress.startTime ?
        (new Date().getTime() - globalIndexingProgress.startTime.getTime()) / 1000 : 0;
    globalIndexingProgress = {
        isIndexing: false,
        currentStep: success ? 'Completed' : 'Failed',
        progress: 100,
        totalSteps: globalIndexingProgress.totalSteps,
        currentStepNumber: globalIndexingProgress.totalSteps
    };
    updateStatusBar(success ? 'BSL Analyzer: Index Ready' : 'BSL Analyzer: Index Failed', undefined);
    progressEmitter.fire(globalIndexingProgress);
    const statusIcon = success ? '✅' : '❌';
    outputChannel.appendLine(`${statusIcon} Index building ${success ? 'completed' : 'failed'} in ${elapsed.toFixed(1)}s`);
    if (success) {
        vscode.window.showInformationMessage(`BSL Index built successfully in ${elapsed.toFixed(1)}s`);
    }
}
function activate(context) {
    console.log('BSL Analyzer v1.8.0 extension is being activated');
    try {
        // Save context globally for use in other functions
        globalExtensionContext = context;
        // Initialize output channel
        outputChannel = vscode.window.createOutputChannel('BSL Analyzer');
        context.subscriptions.push(outputChannel);
        outputChannel.appendLine('🚀 BSL Analyzer v1.8.0 activation started (with Platform Documentation UI)');
        outputChannel.appendLine(`Extension path: ${context.extensionPath}`);
        // Show immediate notification for debugging
        vscode.window.showInformationMessage('BSL Analyzer v1.8.0 is activating...');
        outputChannel.show(); // Показываем Output канал для отладки
        // Initialize configuration
        initializeConfiguration();
        // Register status bar
        registerStatusBar(context);
        // Start LSP client FIRST (it may register some commands)
        // Запускаем с задержкой для стабильности
        setTimeout(() => {
            outputChannel.appendLine('🚀 Starting LSP server with delay...');
            startLanguageClient(context);
        }, 1000);
        // Register sidebar providers
        registerSidebarProviders(context);
        // Register our custom commands AFTER LSP client
        registerCommands(context);
        // Auto-initialize index if configured
        initializeIndexIfNeeded(context);
        // Show welcome message
        showWelcomeMessage();
        outputChannel.appendLine('✅ BSL Analyzer v1.9.0 activated successfully with auto-indexing support');
    }
    catch (error) {
        console.error('BSL Analyzer activation failed:', error);
        outputChannel?.appendLine(`❌ Activation failed: ${error}`);
        vscode.window.showErrorMessage(`BSL Analyzer activation failed: ${error}`);
    }
}
exports.activate = activate;
function initializeConfiguration() {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    indexServerPath = config.get('indexServerPath', '');
    if (!indexServerPath) {
        // First, try bundled binaries from extension
        const extensionPath = vscode.extensions.getExtension('bsl-analyzer-team.bsl-analyzer')?.extensionPath;
        if (extensionPath) {
            const bundledBinPath = path.join(extensionPath, 'bin');
            if (fs.existsSync(bundledBinPath)) {
                indexServerPath = bundledBinPath;
                outputChannel.appendLine(`Using bundled BSL Analyzer binaries at: ${indexServerPath}`);
            }
        }
        // No fallback - extension must be self-contained
        if (!indexServerPath) {
            outputChannel.appendLine(`❌ BSL Analyzer binaries not found in extension.`);
            outputChannel.appendLine(`💡 Please run 'npm run copy:binaries' to update extension binaries.`);
        }
    }
}
async function initializeIndexIfNeeded(context) {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    const autoIndexBuild = config.get('autoIndexBuild', false);
    const configPath = config.get('configurationPath', '');
    if (!autoIndexBuild || !configPath) {
        outputChannel.appendLine('ℹ️ Auto-index build is disabled or configuration path is not set');
        return;
    }
    // Check if index already exists in cache
    const platformVersion = config.get('platformVersion', '8.3.25');
    const indexCachePath = path.join(require('os').homedir(), '.bsl_analyzer', 'project_indices', `${path.basename(configPath)}_${require('crypto').createHash('md5').update(configPath).digest('hex').slice(0, 8)}`, platformVersion);
    if (fs.existsSync(path.join(indexCachePath, 'unified_index.json'))) {
        outputChannel.appendLine('✅ BSL Index already exists in cache, skipping auto-build');
        updateStatusBar('BSL Analyzer: Index Ready');
        return;
    }
    outputChannel.appendLine('🚀 Auto-building BSL index on extension activation...');
    // Build index automatically
    try {
        startIndexing(4);
        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: 'Auto-building BSL Index',
            cancellable: false
        }, async (progress) => {
            updateIndexingProgress(1, 'Loading platform cache...', 10);
            progress.report({ increment: 25, message: 'Loading platform cache...' });
            updateIndexingProgress(2, 'Parsing configuration...', 35);
            progress.report({ increment: 25, message: 'Parsing configuration...' });
            updateIndexingProgress(3, 'Building unified index...', 70);
            progress.report({ increment: 35, message: 'Building unified index...' });
            const args = [
                '--config', configPath,
                '--platform-version', platformVersion
            ];
            const platformDocsArchive = getPlatformDocsArchive();
            if (platformDocsArchive) {
                args.push('--platform-docs-archive', platformDocsArchive);
                outputChannel.appendLine(`📚 Using platform documentation: ${platformDocsArchive}`);
            }
            const result = await executeBslCommand('build_unified_index', args);
            updateIndexingProgress(4, 'Finalizing index...', 90);
            progress.report({ increment: 15, message: 'Finalizing...' });
            finishIndexing(true);
            outputChannel.appendLine('✅ Auto-index build completed successfully');
        });
    }
    catch (error) {
        finishIndexing(false);
        outputChannel.appendLine(`❌ Auto-index build failed: ${error}`);
    }
}
function showWelcomeMessage() {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    const configPath = config.get('configurationPath', '');
    if (!configPath) {
        vscode.window.showInformationMessage('BSL Analyzer is ready! Configure your 1C configuration path in settings to enable full functionality.', 'Open Settings').then(selection => {
            if (selection === 'Open Settings') {
                vscode.commands.executeCommand('workbench.action.openSettings', 'bslAnalyzer.configurationPath');
            }
        });
    }
    else {
        vscode.window.showInformationMessage('BSL Analyzer is ready! Use Ctrl+Shift+P and search for "BSL" to explore features.');
    }
}
function startLanguageClient(context) {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    let serverPath = config.get('serverPath', '');
    const serverMode = config.get('serverMode', 'stdio');
    const tcpPort = config.get('tcpPort', 8080);
    const traceLevel = config.get('trace.server', 'off');
    // Используем getBinaryPath для получения пути к LSP серверу
    try {
        if (!serverPath) {
            // Если путь не указан явно, используем общую логику выбора бинарников
            serverPath = getBinaryPath('lsp_server');
            outputChannel.appendLine(`🚀 LSP server path resolved: ${serverPath}`);
        }
    }
    catch (error) {
        outputChannel.appendLine(`❌ Failed to locate LSP server: ${error.message || error}`);
        vscode.window.showWarningMessage('BSL Analyzer: LSP server not found. Extension features will be limited.', 'Show Details').then(selection => {
            if (selection === 'Show Details') {
                outputChannel.show();
            }
        });
        updateStatusBar('BSL Analyzer: No LSP');
        return;
    }
    outputChannel.appendLine(`🚀 Starting LSP server: ${serverPath}`);
    outputChannel.appendLine(`📝 Server mode: ${serverMode}`);
    let serverOptions;
    if (serverMode === 'tcp') {
        // TCP mode - not implemented yet
        outputChannel.appendLine(`❌ TCP mode is not implemented yet. Using stdio mode instead.`);
        serverOptions = {
            command: serverPath,
            args: [],
            transport: node_1.TransportKind.stdio,
            options: {
                env: { ...process.env, RUST_LOG: 'info' }
            }
        };
    }
    else {
        // STDIO mode (рекомендуется)
        serverOptions = {
            command: serverPath,
            args: [],
            transport: node_1.TransportKind.stdio,
            options: {
                env: { ...process.env, RUST_LOG: 'info' }
            }
        };
    }
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'bsl' }],
        synchronize: {
            fileEvents: [
                vscode.workspace.createFileSystemWatcher('**/*.bsl'),
                vscode.workspace.createFileSystemWatcher('**/*.os'),
                vscode.workspace.createFileSystemWatcher('**/bsl-rules.toml'),
                vscode.workspace.createFileSystemWatcher('**/lsp-config.toml')
            ]
        },
        revealOutputChannelOn: node_1.RevealOutputChannelOn.Error,
        initializationOptions: {
            enableRealTimeAnalysis: config.get('enableRealTimeAnalysis', true),
            enableMetrics: config.get('enableMetrics', true),
            maxFileSize: config.get('maxFileSize', 1048576),
            rulesConfig: config.get('rulesConfig', ''),
            configurationPath: getConfigurationPath(),
            platformVersion: getPlatformVersion()
        },
        diagnosticCollectionName: 'bsl-analyzer',
        outputChannel: outputChannel,
        middleware: {
            executeCommand: (command, args, next) => {
                outputChannel.appendLine(`🔄 LSP executeCommand: ${command}`);
                return next(command, args);
            }
        }
    };
    client = new node_1.LanguageClient('bslAnalyzer', 'BSL Analyzer Language Server', serverOptions, clientOptions);
    // Добавляем обработчики ошибок
    client.onDidChangeState(event => {
        outputChannel.appendLine(`📊 LSP state changed: ${event.oldState} -> ${event.newState}`);
        if (event.newState === 1) { // Starting
            updateStatusBar('BSL Analyzer: Starting...');
        }
        else if (event.newState === 2) { // Running  
            updateStatusBar('BSL Analyzer: Ready');
        }
        else if (event.newState === 3) { // Stopped
            updateStatusBar('BSL Analyzer: Stopped');
        }
    });
    // Start the client с улучшенной обработкой ошибок
    client.start().then(() => {
        console.log('BSL Analyzer LSP client started successfully');
        updateStatusBar('BSL Analyzer: Ready');
        outputChannel.appendLine('✅ LSP client started successfully');
        // Добавляем в subscriptions только при успешном запуске
        context.subscriptions.push({
            dispose: async () => {
                if (client) {
                    try {
                        // Проверяем состояние перед остановкой
                        const state = client.state;
                        if (state === 2) { // Running
                            outputChannel.appendLine('🛑 Stopping LSP client...');
                            await client.stop();
                        }
                        else {
                            outputChannel.appendLine(`⚠️ LSP client not running, state: ${state}`);
                        }
                    }
                    catch (error) {
                        outputChannel.appendLine(`⚠️ Error stopping LSP client: ${error}`);
                    }
                }
            }
        });
    }).catch(error => {
        console.error('Failed to start BSL Analyzer LSP client:', error);
        outputChannel.appendLine(`❌ LSP client startup failed: ${error.message}`);
        // Если это ошибка соединения, пытаемся очистить клиент
        if (error.message?.includes('connection') || error.message?.includes('disposed')) {
            outputChannel.appendLine('🔄 Connection error detected, cleaning up...');
            try {
                if (client) {
                    const state = client.state;
                    outputChannel.appendLine(`Current client state: ${state}`);
                    // Не пытаемся остановить если клиент в состоянии starting (1)
                    if (state !== 1) {
                        client.stop().catch(() => {
                            // Игнорируем ошибки остановки
                        });
                    }
                }
            }
            catch (cleanupError) {
                outputChannel.appendLine(`⚠️ Cleanup error: ${cleanupError}`);
            }
        }
        // Cleanup client on error
        client = null;
        // Более мягкое отображение ошибки
        vscode.window.showWarningMessage(`BSL Analyzer LSP server failed to start. Extension features will be limited.`, 'Show Details').then(selection => {
            if (selection === 'Show Details') {
                outputChannel.show();
            }
        });
        updateStatusBar('BSL Analyzer: LSP Failed');
    });
}
function registerCommands(context) {
    outputChannel.appendLine('📝 Registering BSL Analyzer commands...');
    // Helper function to safely register commands
    const safeRegisterCommand = (commandId, callback) => {
        try {
            // Dispose existing command if it exists
            vscode.commands.getCommands(true).then(existingCommands => {
                if (existingCommands.includes(commandId)) {
                    outputChannel.appendLine(`⚠️ Command ${commandId} already exists, will be replaced`);
                }
            });
            const disposable = vscode.commands.registerCommand(commandId, callback);
            context.subscriptions.push(disposable);
            outputChannel.appendLine(`✅ Registered command: ${commandId}`);
            return disposable;
        }
        catch (error) {
            outputChannel.appendLine(`❌ Failed to register command ${commandId}: ${error}`);
            return null;
        }
    };
    // Note: bslAnalyzer.analyzeFile is handled by LSP server, but we need a UI wrapper
    safeRegisterCommand('bslAnalyzer.analyzeCurrentFile', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'bsl') {
            vscode.window.showWarningMessage('Please open a BSL file to analyze');
            return;
        }
        try {
            // Delegate to LSP server with proper URI
            if (client && client.isRunning()) {
                await client.sendRequest('workspace/executeCommand', {
                    command: 'bslAnalyzer.analyzeFile',
                    arguments: [editor.document.uri.toString()]
                });
                vscode.window.showInformationMessage('✅ File analysis completed');
            }
            else {
                vscode.window.showErrorMessage('LSP server not running');
            }
        }
        catch (error) {
            vscode.window.showErrorMessage(`Analysis failed: ${error}`);
        }
    });
    // Helper function for direct analysis
    async function performDirectAnalysis(document) {
        outputChannel.appendLine(`📁 Analyzing file: ${document.fileName}`);
        outputChannel.appendLine(`📊 File size: ${document.getText().length} characters`);
        outputChannel.appendLine(`🔤 Language: ${document.languageId}`);
        // TODO: Add direct BSL analysis using bundled bsl-analyzer.exe
        if (indexServerPath) {
            outputChannel.appendLine(`🔧 Using BSL analyzer at: ${indexServerPath}`);
        }
    }
    // Note: bslAnalyzer.analyzeWorkspace is now handled by LSP server automatically
    // We removed the duplicate command registration to avoid conflicts
    // Generate reports
    safeRegisterCommand('bslAnalyzer.generateReports', async () => {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders || workspaceFolders.length === 0) {
            vscode.window.showWarningMessage('No workspace folder is open');
            return;
        }
        const outputDir = await vscode.window.showInputBox({
            prompt: 'Enter output directory for reports',
            value: './reports'
        });
        if (!outputDir) {
            return;
        }
        updateStatusBar('BSL Analyzer: Generating reports...');
        try {
            if (!client) {
                throw new Error('LSP client is not running');
            }
            await client.sendRequest('workspace/executeCommand', {
                command: 'bslAnalyzer.generateReports',
                arguments: [workspaceFolders[0].uri.toString(), outputDir]
            });
            const openReports = await vscode.window.showInformationMessage('Reports generated successfully', 'Open Reports Folder');
            if (openReports) {
                vscode.commands.executeCommand('vscode.openFolder', vscode.Uri.file(outputDir));
            }
            updateStatusBar('BSL Analyzer: Ready');
        }
        catch (error) {
            vscode.window.showErrorMessage(`Report generation failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });
    // Show metrics
    safeRegisterCommand('bslAnalyzer.showMetrics', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'bsl') {
            vscode.window.showWarningMessage('Please open a BSL file to show metrics');
            return;
        }
        try {
            if (!client) {
                throw new Error('LSP client is not running');
            }
            const metrics = await client.sendRequest('workspace/executeCommand', {
                command: 'bslAnalyzer.getMetrics',
                arguments: [editor.document.uri.toString()]
            });
            // Show metrics in a webview
            showMetricsWebview(context, metrics);
        }
        catch (error) {
            vscode.window.showErrorMessage(`Failed to get metrics: ${error}`);
        }
    });
    // Configure rules
    safeRegisterCommand('bslAnalyzer.configureRules', async () => {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders || workspaceFolders.length === 0) {
            vscode.window.showWarningMessage('No workspace folder is open');
            return;
        }
        const rulesFile = vscode.Uri.joinPath(workspaceFolders[0].uri, 'bsl-rules.toml');
        try {
            // Check if rules file exists
            await vscode.workspace.fs.stat(rulesFile);
            // File exists, open it
            const document = await vscode.workspace.openTextDocument(rulesFile);
            await vscode.window.showTextDocument(document);
        }
        catch {
            // File doesn't exist, create it
            const createFile = await vscode.window.showInformationMessage('Rules configuration file not found. Would you like to create one?', 'Create Rules File');
            if (createFile) {
                try {
                    if (!client) {
                        throw new Error('LSP client is not running');
                    }
                    await client.sendRequest('workspace/executeCommand', {
                        command: 'bslAnalyzer.createRulesConfig',
                        arguments: [rulesFile.toString()]
                    });
                    const document = await vscode.workspace.openTextDocument(rulesFile);
                    await vscode.window.showTextDocument(document);
                }
                catch (error) {
                    vscode.window.showErrorMessage(`Failed to create rules file: ${error}`);
                }
            }
        }
    });
    // Search BSL Type
    safeRegisterCommand('bslAnalyzer.searchType', async () => {
        const typeName = await vscode.window.showInputBox({
            prompt: 'Enter BSL type name to search (e.g., "Массив", "Справочники.Номенклатура")',
            placeHolder: 'Type name...'
        });
        if (!typeName) {
            return;
        }
        updateStatusBar('BSL Analyzer: Searching type...');
        try {
            const result = await executeBslCommand('query_type', [
                '--name', typeName,
                '--config', getConfigurationPath(),
                '--platform-version', getPlatformVersion(),
                '--show-all-methods'
            ]);
            showTypeInfoWebview(context, typeName, result);
            updateStatusBar('BSL Analyzer: Ready');
        }
        catch (error) {
            vscode.window.showErrorMessage(`Type search failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });
    // Search Method in Type
    safeRegisterCommand('bslAnalyzer.searchMethod', async () => {
        const typeName = await vscode.window.showInputBox({
            prompt: 'Enter type name (e.g., "Массив", "Справочники.Номенклатура")',
            placeHolder: 'Type name...'
        });
        if (!typeName) {
            return;
        }
        const methodName = await vscode.window.showInputBox({
            prompt: 'Enter method name to search',
            placeHolder: 'Method name...'
        });
        if (!methodName) {
            return;
        }
        updateStatusBar('BSL Analyzer: Searching method...');
        try {
            const result = await executeBslCommand('query_type', [
                '--name', typeName,
                '--config', getConfigurationPath(),
                '--platform-version', getPlatformVersion(),
                '--show-all-methods'
            ]);
            showMethodInfoWebview(context, typeName, methodName, result);
            updateStatusBar('BSL Analyzer: Ready');
        }
        catch (error) {
            vscode.window.showErrorMessage(`Method search failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });
    // Build Unified BSL Index
    safeRegisterCommand('bslAnalyzer.buildIndex', async () => {
        const configPath = getConfigurationPath();
        if (!configPath) {
            vscode.window.showWarningMessage('Please configure the 1C configuration path in settings');
            return;
        }
        const choice = await vscode.window.showInformationMessage('Building unified BSL index. This may take a few seconds...', 'Build Index', 'Cancel');
        if (choice !== 'Build Index') {
            return;
        }
        // Инициализируем систему прогресса
        startIndexing(4); // 4 основных этапа индексации
        try {
            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: 'Building BSL Index',
                cancellable: false
            }, async (progress) => {
                // Этап 1: Инициализация
                updateIndexingProgress(1, 'Loading platform cache...', 10);
                progress.report({ increment: 25, message: 'Loading platform cache...' });
                await new Promise(resolve => setTimeout(resolve, 500)); // Задержка для демонстрации
                // Этап 2: Парсинг конфигурации  
                updateIndexingProgress(2, 'Parsing configuration...', 35);
                progress.report({ increment: 25, message: 'Parsing configuration...' });
                await new Promise(resolve => setTimeout(resolve, 500)); // Задержка для демонстрации
                // Этап 3: Построение индекса
                updateIndexingProgress(3, 'Building unified index...', 70);
                progress.report({ increment: 35, message: 'Building unified index...' });
                await new Promise(resolve => setTimeout(resolve, 300)); // Задержка для демонстрации
                const args = [
                    '--config', configPath,
                    '--platform-version', getPlatformVersion()
                ];
                const platformDocsArchive = getPlatformDocsArchive();
                if (platformDocsArchive) {
                    args.push('--platform-docs-archive', platformDocsArchive);
                }
                const result = await executeBslCommand('build_unified_index', args);
                // Этап 4: Завершение
                updateIndexingProgress(4, 'Finalizing index...', 90);
                progress.report({ increment: 15, message: 'Finalizing...' });
                // Завершаем индексацию
                finishIndexing(true);
                // Извлекаем количество типов из результата для более информативного сообщения
                let typesCount = 'unknown';
                const typesMatch = result.match(/(\d+)\s+entities/i);
                if (typesMatch) {
                    typesCount = typesMatch[1];
                }
                vscode.window.showInformationMessage(`✅ BSL Index built successfully with ${typesCount} types`);
                return result;
            });
        }
        catch (error) {
            finishIndexing(false);
            vscode.window.showErrorMessage(`Index build failed: ${error}`);
            outputChannel.appendLine(`Index build error: ${error}`);
        }
    });
    // Show Index Statistics
    safeRegisterCommand('bslAnalyzer.showIndexStats', async () => {
        const configPath = getConfigurationPath();
        if (!configPath) {
            vscode.window.showWarningMessage('Please configure the 1C configuration path in settings');
            return;
        }
        updateStatusBar('BSL Analyzer: Loading stats...');
        try {
            const result = await executeBslCommand('query_type', [
                '--name', 'stats',
                '--config', configPath,
                '--platform-version', getPlatformVersion()
            ]);
            showIndexStatsWebview(context, result);
            updateStatusBar('BSL Analyzer: Ready');
        }
        catch (error) {
            vscode.window.showErrorMessage(`Failed to load index stats: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });
    // Incremental Index Update
    safeRegisterCommand('bslAnalyzer.incrementalUpdate', async () => {
        const configPath = getConfigurationPath();
        if (!configPath) {
            vscode.window.showWarningMessage('Please configure the 1C configuration path in settings');
            return;
        }
        // Инициализируем систему прогресса для инкрементального обновления (3 этапа)
        startIndexing(3);
        try {
            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: 'Incremental Index Update',
                cancellable: false
            }, async (progress) => {
                // Этап 1: Анализ изменений
                updateIndexingProgress(1, 'Analyzing changes...', 20);
                progress.report({ increment: 30, message: 'Analyzing changes...' });
                await new Promise(resolve => setTimeout(resolve, 400)); // Задержка для демонстрации
                // Этап 2: Обновление индекса
                updateIndexingProgress(2, 'Updating index...', 60);
                progress.report({ increment: 50, message: 'Updating index...' });
                await new Promise(resolve => setTimeout(resolve, 600)); // Задержка для демонстрации
                const result = await executeBslCommand('incremental_update', [
                    '--config', configPath,
                    '--platform-version', getPlatformVersion(),
                    '--verbose'
                ]);
                // Этап 3: Завершение
                updateIndexingProgress(3, 'Finalizing...', 95);
                progress.report({ increment: 20, message: 'Finalizing...' });
                finishIndexing(true);
                vscode.window.showInformationMessage(`✅ Index updated successfully: ${result}`);
                return result;
            });
        }
        catch (error) {
            finishIndexing(false);
            vscode.window.showErrorMessage(`Incremental update failed: ${error}`);
            outputChannel.appendLine(`Incremental update error: ${error}`);
        }
    });
    // Explore Type Methods & Properties
    safeRegisterCommand('bslAnalyzer.exploreType', async () => {
        const editor = vscode.window.activeTextEditor;
        let typeName = '';
        if (editor && editor.selection && !editor.selection.isEmpty) {
            typeName = editor.document.getText(editor.selection);
        }
        if (!typeName) {
            typeName = await vscode.window.showInputBox({
                prompt: 'Enter type name to explore',
                placeHolder: 'Type name...'
            }) || '';
        }
        if (!typeName) {
            return;
        }
        updateStatusBar('BSL Analyzer: Loading type info...');
        try {
            const result = await executeBslCommand('query_type', [
                '--name', typeName,
                '--config', getConfigurationPath(),
                '--platform-version', getPlatformVersion(),
                '--show-all-methods'
            ]);
            showTypeExplorerWebview(context, typeName, result);
            updateStatusBar('BSL Analyzer: Ready');
        }
        catch (error) {
            vscode.window.showErrorMessage(`Type exploration failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });
    // Validate Method Call
    safeRegisterCommand('bslAnalyzer.validateMethodCall', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'bsl') {
            vscode.window.showWarningMessage('Please open a BSL file and select a method call');
            return;
        }
        let selectedText = '';
        if (editor.selection && !editor.selection.isEmpty) {
            selectedText = editor.document.getText(editor.selection);
        }
        if (!selectedText) {
            vscode.window.showWarningMessage('Please select a method call to validate');
            return;
        }
        updateStatusBar('BSL Analyzer: Validating method call...');
        try {
            // Parse method call from selected text
            const methodCallInfo = parseMethodCall(selectedText);
            if (!methodCallInfo) {
                vscode.window.showWarningMessage('Invalid method call format');
                return;
            }
            const result = await executeBslCommand('query_type', [
                '--name', methodCallInfo.typeName,
                '--config', getConfigurationPath(),
                '--platform-version', getPlatformVersion(),
                '--show-all-methods'
            ]);
            showMethodValidationWebview(context, methodCallInfo, result);
            updateStatusBar('BSL Analyzer: Ready');
        }
        catch (error) {
            vscode.window.showErrorMessage(`Method validation failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });
    // Check Type Compatibility
    safeRegisterCommand('bslAnalyzer.checkTypeCompatibility', async () => {
        const fromType = await vscode.window.showInputBox({
            prompt: 'Enter source type name',
            placeHolder: 'e.g., Справочники.Номенклатура'
        });
        if (!fromType) {
            return;
        }
        const toType = await vscode.window.showInputBox({
            prompt: 'Enter target type name',
            placeHolder: 'e.g., СправочникСсылка'
        });
        if (!toType) {
            return;
        }
        updateStatusBar('BSL Analyzer: Checking compatibility...');
        try {
            // This would need a specialized command in the Rust binary
            const result = await executeBslCommand('check_type_compatibility', [
                '--from', fromType,
                '--to', toType,
                '--config', getConfigurationPath(),
                '--platform-version', getPlatformVersion()
            ]);
            showTypeCompatibilityWebview(context, fromType, toType, result);
            updateStatusBar('BSL Analyzer: Ready');
        }
        catch (error) {
            vscode.window.showErrorMessage(`Type compatibility check failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });
    // Restart server
    safeRegisterCommand('bslAnalyzer.restartServer', async () => {
        updateStatusBar('BSL Analyzer: Restarting...');
        outputChannel.appendLine('🔄 Restarting LSP server...');
        try {
            if (client) {
                outputChannel.appendLine('🛑 Stopping existing LSP client...');
                await client.stop();
                outputChannel.appendLine('✅ LSP client stopped');
            }
            // Небольшая задержка перед перезапуском
            await new Promise(resolve => setTimeout(resolve, 1000));
            outputChannel.appendLine('🚀 Starting new LSP client...');
            startLanguageClient(context);
            vscode.window.showInformationMessage('✅ BSL Analyzer server restarted');
            outputChannel.appendLine('✅ LSP server restart completed');
        }
        catch (error) {
            outputChannel.appendLine(`❌ Failed to restart LSP server: ${error}`);
            vscode.window.showErrorMessage(`Failed to restart server: ${error}`);
            updateStatusBar('BSL Analyzer: Restart Failed');
        }
    });
    // Команда для тестирования системы прогресса (только для отладки)
    safeRegisterCommand('bslAnalyzer.testProgress', async () => {
        outputChannel.appendLine('🧪 Testing progress system...');
        outputChannel.appendLine(`📊 StatusBar exists: ${!!statusBarItem}`);
        outputChannel.appendLine(`📊 Global progress state: ${JSON.stringify(globalIndexingProgress)}`);
        startIndexing(5);
        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: 'Testing Progress System',
            cancellable: false
        }, async (progress) => {
            // Имитируем 5 этапов с задержками
            for (let i = 1; i <= 5; i++) {
                const stepName = `Step ${i}: Processing...`;
                const progressPercent = Math.floor((i / 5) * 100);
                outputChannel.appendLine(`📊 Updating progress: ${i}/${5} - ${stepName} (${progressPercent}%)`);
                updateIndexingProgress(i, stepName, progressPercent);
                progress.report({
                    increment: 20,
                    message: stepName
                });
                outputChannel.appendLine(`📊 After update - StatusBar text: ${statusBarItem?.text}`);
                outputChannel.appendLine(`📊 Global progress after update: ${JSON.stringify(globalIndexingProgress)}`);
                // Задержка для демонстрации
                await new Promise(resolve => setTimeout(resolve, 2000)); // Увеличил до 2 секунд
            }
            outputChannel.appendLine('🏁 Finishing indexing...');
            finishIndexing(true);
            outputChannel.appendLine(`📊 Final StatusBar text: ${statusBarItem?.text}`);
        });
        outputChannel.appendLine('✅ Progress system test completed');
    });
    // Commands are already added to context.subscriptions in safeRegisterCommand function  
    outputChannel.appendLine('✅ Successfully registered 13 extension commands (analysis commands handled by LSP)');
}
let statusBarItem;
function registerStatusBar(context) {
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    // Use our UI wrapper that properly passes file URI to LSP
    statusBarItem.command = 'bslAnalyzer.analyzeCurrentFile';
    statusBarItem.text = 'BSL Analyzer: Starting...';
    statusBarItem.tooltip = 'Click to analyze current file (via LSP)';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);
}
function updateStatusBar(text, progress) {
    if (!statusBarItem)
        return;
    if (progress && progress.isIndexing) {
        // Показываем прогресс индексации
        const spinner = '$(loading~spin)';
        const progressBar = Math.floor(progress.progress / 10); // 0-10 блоков
        const progressText = '▓'.repeat(progressBar) + '░'.repeat(10 - progressBar);
        statusBarItem.text = `${spinner} BSL: ${progress.currentStep} (${progress.progress}%)`;
        statusBarItem.tooltip = `Indexing BSL project...\n${progress.currentStep}\nProgress: ${progress.progress}%\nStep ${progress.currentStepNumber}/${progress.totalSteps}${progress.estimatedTimeRemaining ? `\nETA: ${progress.estimatedTimeRemaining}` : ''}`;
        statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
        statusBarItem.color = undefined; // Используем цвет темы
    }
    else {
        // Обычное состояние
        statusBarItem.text = text || 'BSL Analyzer: Ready';
        statusBarItem.tooltip = 'Click to analyze current file';
        statusBarItem.backgroundColor = undefined;
        statusBarItem.color = undefined;
    }
}
function showMetricsWebview(context, metrics) {
    const panel = vscode.window.createWebviewPanel('bslMetrics', 'BSL Code Quality Metrics', vscode.ViewColumn.Two, {
        enableScripts: true,
        retainContextWhenHidden: true
    });
    panel.webview.html = getMetricsWebviewContent(metrics);
}
function getMetricsWebviewContent(metrics) {
    return `
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>BSL Code Quality Metrics</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                margin: 20px;
                background-color: var(--vscode-editor-background);
                color: var(--vscode-editor-foreground);
            }
            .metric-card {
                background: var(--vscode-editor-inactiveSelectionBackground);
                border: 1px solid var(--vscode-panel-border);
                border-radius: 6px;
                padding: 16px;
                margin: 12px 0;
            }
            .metric-title {
                font-size: 18px;
                font-weight: bold;
                margin-bottom: 8px;
            }
            .metric-value {
                font-size: 24px;
                font-weight: bold;
                color: var(--vscode-charts-blue);
            }
            .metric-description {
                font-size: 14px;
                color: var(--vscode-descriptionForeground);
                margin-top: 4px;
            }
            .quality-score {
                text-align: center;
                font-size: 48px;
                margin: 20px 0;
            }
            .score-excellent { color: #28a745; }
            .score-good { color: #17a2b8; }
            .score-warning { color: #ffc107; }
            .score-poor { color: #dc3545; }
        </style>
    </head>
    <body>
        <h1>🔍 Code Quality Metrics</h1>
        
        <div class="metric-card">
            <div class="metric-title">Overall Quality Score</div>
            <div class="quality-score ${getScoreClass(metrics.qualityScore)}">
                ${metrics.qualityScore}/100
            </div>
            <div class="metric-description">
                Composite score based on complexity, maintainability, and technical debt
            </div>
        </div>

        <div class="metric-card">
            <div class="metric-title">Maintainability Index</div>
            <div class="metric-value">${metrics.maintainabilityIndex}</div>
            <div class="metric-description">
                Higher values indicate more maintainable code
            </div>
        </div>

        <div class="metric-card">
            <div class="metric-title">Average Complexity</div>
            <div class="metric-value">${metrics.averageComplexity}</div>
            <div class="metric-description">
                Cyclomatic complexity averaged across all functions
            </div>
        </div>

        <div class="metric-card">
            <div class="metric-title">Technical Debt Items</div>
            <div class="metric-value">${metrics.technicalDebtItems}</div>
            <div class="metric-description">
                Number of identified technical debt issues
            </div>
        </div>

        ${metrics.recommendations ? `
        <div class="metric-card">
            <div class="metric-title">💡 Recommendations</div>
            <ul>
                ${metrics.recommendations.map((rec) => `<li>${rec}</li>`).join('')}
            </ul>
        </div>
        ` : ''}
    </body>
    </html>
    `;
}
function getScoreClass(score) {
    if (score >= 90)
        return 'score-excellent';
    if (score >= 75)
        return 'score-good';
    if (score >= 50)
        return 'score-warning';
    return 'score-poor';
}
// Helper functions for BSL Index commands
function getConfigurationPath() {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    return config.get('configurationPath', '');
}
function getPlatformVersion() {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    return config.get('platformVersion', '8.3.25');
}
function getPlatformDocsArchive() {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    const userArchive = config.get('platformDocsArchive', '');
    // Используем только указанный пользователем архив
    // Документация должна быть указана явно для корректной работы
    if (userArchive && fs.existsSync(userArchive)) {
        outputChannel.appendLine(`📚 Using user-specified platform documentation: ${userArchive}`);
        return userArchive;
    }
    // Если документация не указана - показываем предупреждение
    if (!userArchive) {
        outputChannel.appendLine(`⚠️ Platform documentation not configured. Some features may be limited.`);
        outputChannel.appendLine(`💡 Specify path to rebuilt.shcntx_ru.zip or rebuilt.shlang_ru.zip in settings.`);
    }
    else {
        outputChannel.appendLine(`❌ Platform documentation not found at: ${userArchive}`);
    }
    return '';
}
function getBinaryPath(binaryName) {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    const useBundled = config.get('useBundledBinaries', true);
    // Если явно указано использовать встроенные бинарники
    if (useBundled) {
        // Сначала пробуем глобальный контекст (для development режима)
        if (globalExtensionContext) {
            const contextBinPath = path.join(globalExtensionContext.extensionPath, 'bin', `${binaryName}.exe`);
            if (fs.existsSync(contextBinPath)) {
                outputChannel.appendLine(`✅ Using bundled binary from context: ${contextBinPath}`);
                return contextBinPath;
            }
        }
        // Затем пробуем найти установленное расширение
        const extensionPath = vscode.extensions.getExtension('bsl-analyzer-team.bsl-analyzer')?.extensionPath;
        if (extensionPath) {
            const bundledBinPath = path.join(extensionPath, 'bin', `${binaryName}.exe`);
            if (fs.existsSync(bundledBinPath)) {
                outputChannel.appendLine(`✅ Using bundled binary: ${bundledBinPath}`);
                return bundledBinPath;
            }
        }
        // Fallback на vscode-extension/bin для development
        const workspacePath = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
        if (workspacePath) {
            const devBinPath = path.join(workspacePath, 'vscode-extension', 'bin', `${binaryName}.exe`);
            if (fs.existsSync(devBinPath)) {
                outputChannel.appendLine(`✅ Using development binary: ${devBinPath}`);
                return devBinPath;
            }
        }
        throw new Error(`BSL Analyzer: Binary '${binaryName}.exe' not found. Please check that binaries are in the 'bin' folder.`);
    }
    // Если отключены встроенные бинарники, используем внешние
    const serverPath = config.get('indexServerPath', '');
    if (serverPath) {
        const externalBinPath = path.join(serverPath, `${binaryName}.exe`);
        if (fs.existsSync(externalBinPath)) {
            outputChannel.appendLine(`⚠️ Using external binary: ${externalBinPath}`);
            return externalBinPath;
        }
    }
    // Fallback на workspace (только если явно отключены встроенные)
    const workspacePath = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
    if (workspacePath) {
        const workspaceBinPath = path.join(workspacePath, 'target', 'release', `${binaryName}.exe`);
        if (fs.existsSync(workspaceBinPath)) {
            outputChannel.appendLine(`⚠️ Using workspace binary: ${workspaceBinPath}`);
            return workspaceBinPath;
        }
    }
    throw new Error(`BSL Analyzer: Binary '${binaryName}.exe' not found. Enable useBundledBinaries or specify indexServerPath.`);
}
function executeBslCommand(command, args) {
    return new Promise((resolve, reject) => {
        const binaryPath = getBinaryPath(command);
        outputChannel.appendLine(`Executing: ${binaryPath} ${args.join(' ')}`);
        const process = (0, child_process_1.spawn)(binaryPath, args, {
            cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath
        });
        let stdout = '';
        let stderr = '';
        process.stdout?.on('data', (data) => {
            stdout += data.toString();
        });
        process.stderr?.on('data', (data) => {
            stderr += data.toString();
        });
        process.on('close', (code) => {
            outputChannel.appendLine(`Command completed with code: ${code}`);
            outputChannel.appendLine(`Output: ${stdout}`);
            if (stderr) {
                outputChannel.appendLine(`Error: ${stderr}`);
            }
            if (code === 0) {
                resolve(stdout);
            }
            else {
                reject(new Error(`Command failed with code ${code}: ${stderr}`));
            }
        });
        process.on('error', (error) => {
            outputChannel.appendLine(`Process error: ${error.message}`);
            reject(error);
        });
    });
}
function parseMethodCall(selectedText) {
    // Basic method call parsing - can be enhanced
    const methodCallRegex = /(\w+)\.(\w+)\s*\(([^)]*)\)/;
    const match = selectedText.match(methodCallRegex);
    if (match) {
        return {
            typeName: match[1],
            methodName: match[2],
            parameters: match[3].split(',').map(p => p.trim()).filter(p => p)
        };
    }
    return null;
}
function showTypeInfoWebview(context, typeName, result) {
    const panel = vscode.window.createWebviewPanel('bslTypeInfo', `BSL Type: ${typeName}`, vscode.ViewColumn.Two, {
        enableScripts: true,
        retainContextWhenHidden: true
    });
    panel.webview.html = getTypeInfoWebviewContent(typeName, result);
}
function showMethodInfoWebview(context, typeName, methodName, result) {
    const panel = vscode.window.createWebviewPanel('bslMethodInfo', `BSL Method: ${typeName}.${methodName}`, vscode.ViewColumn.Two, {
        enableScripts: true,
        retainContextWhenHidden: true
    });
    panel.webview.html = getMethodInfoWebviewContent(typeName, methodName, result);
}
function showTypeExplorerWebview(context, typeName, result) {
    const panel = vscode.window.createWebviewPanel('bslTypeExplorer', `BSL Type Explorer: ${typeName}`, vscode.ViewColumn.Two, {
        enableScripts: true,
        retainContextWhenHidden: true
    });
    panel.webview.html = getTypeExplorerWebviewContent(typeName, result);
}
function showIndexStatsWebview(context, result) {
    const panel = vscode.window.createWebviewPanel('bslIndexStats', 'BSL Index Statistics', vscode.ViewColumn.Two, {
        enableScripts: true,
        retainContextWhenHidden: true
    });
    panel.webview.html = getIndexStatsWebviewContent(result);
}
function showMethodValidationWebview(context, methodCall, result) {
    const panel = vscode.window.createWebviewPanel('bslMethodValidation', `Method Validation: ${methodCall.typeName}.${methodCall.methodName}`, vscode.ViewColumn.Two, {
        enableScripts: true,
        retainContextWhenHidden: true
    });
    panel.webview.html = getMethodValidationWebviewContent(methodCall, result);
}
function showTypeCompatibilityWebview(context, fromType, toType, result) {
    const panel = vscode.window.createWebviewPanel('bslTypeCompatibility', `Type Compatibility: ${fromType} → ${toType}`, vscode.ViewColumn.Two, {
        enableScripts: true,
        retainContextWhenHidden: true
    });
    panel.webview.html = getTypeCompatibilityWebviewContent(fromType, toType, result);
}
function getTypeInfoWebviewContent(typeName, result) {
    return `
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>BSL Type Information</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                margin: 20px;
                background-color: var(--vscode-editor-background);
                color: var(--vscode-editor-foreground);
            }
            .type-header {
                border-bottom: 2px solid var(--vscode-panel-border);
                padding-bottom: 16px;
                margin-bottom: 20px;
            }
            .type-name {
                font-size: 24px;
                font-weight: bold;
                color: var(--vscode-charts-blue);
            }
            .result-content {
                background: var(--vscode-editor-inactiveSelectionBackground);
                border: 1px solid var(--vscode-panel-border);
                border-radius: 6px;
                padding: 16px;
                white-space: pre-wrap;
                font-family: 'Consolas', 'Monaco', monospace;
                font-size: 14px;
                overflow-x: auto;
            }
        </style>
    </head>
    <body>
        <div class="type-header">
            <div class="type-name">🔍 ${typeName}</div>
        </div>
        <div class="result-content">${result}</div>
    </body>
    </html>
    `;
}
function getMethodInfoWebviewContent(typeName, methodName, result) {
    return `
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>BSL Method Information</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                margin: 20px;
                background-color: var(--vscode-editor-background);
                color: var(--vscode-editor-foreground);
            }
            .method-header {
                border-bottom: 2px solid var(--vscode-panel-border);
                padding-bottom: 16px;
                margin-bottom: 20px;
            }
            .method-name {
                font-size: 24px;
                font-weight: bold;
                color: var(--vscode-charts-green);
            }
            .result-content {
                background: var(--vscode-editor-inactiveSelectionBackground);
                border: 1px solid var(--vscode-panel-border);
                border-radius: 6px;
                padding: 16px;
                white-space: pre-wrap;
                font-family: 'Consolas', 'Monaco', monospace;
                font-size: 14px;
                overflow-x: auto;
            }
        </style>
    </head>
    <body>
        <div class="method-header">
            <div class="method-name">📋 ${typeName}.${methodName}</div>
        </div>
        <div class="result-content">${result}</div>
    </body>
    </html>
    `;
}
function getTypeExplorerWebviewContent(typeName, result) {
    return `
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>BSL Type Explorer</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                margin: 20px;
                background-color: var(--vscode-editor-background);
                color: var(--vscode-editor-foreground);
            }
            .explorer-header {
                border-bottom: 2px solid var(--vscode-panel-border);
                padding-bottom: 16px;
                margin-bottom: 20px;
            }
            .explorer-title {
                font-size: 24px;
                font-weight: bold;
                color: var(--vscode-charts-purple);
            }
            .result-content {
                background: var(--vscode-editor-inactiveSelectionBackground);
                border: 1px solid var(--vscode-panel-border);
                border-radius: 6px;
                padding: 16px;
                white-space: pre-wrap;
                font-family: 'Consolas', 'Monaco', monospace;
                font-size: 14px;
                overflow-x: auto;
            }
        </style>
    </head>
    <body>
        <div class="explorer-header">
            <div class="explorer-title">🧭 Type Explorer: ${typeName}</div>
        </div>
        <div class="result-content">${result}</div>
    </body>
    </html>
    `;
}
function getIndexStatsWebviewContent(result) {
    return `
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>BSL Index Statistics</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                margin: 20px;
                background-color: var(--vscode-editor-background);
                color: var(--vscode-editor-foreground);
            }
            .stats-header {
                border-bottom: 2px solid var(--vscode-panel-border);
                padding-bottom: 16px;
                margin-bottom: 20px;
            }
            .stats-title {
                font-size: 24px;
                font-weight: bold;
                color: var(--vscode-charts-orange);
            }
            .result-content {
                background: var(--vscode-editor-inactiveSelectionBackground);
                border: 1px solid var(--vscode-panel-border);
                border-radius: 6px;
                padding: 16px;
                white-space: pre-wrap;
                font-family: 'Consolas', 'Monaco', monospace;
                font-size: 14px;
                overflow-x: auto;
            }
        </style>
    </head>
    <body>
        <div class="stats-header">
            <div class="stats-title">📊 Index Statistics</div>
        </div>
        <div class="result-content">${result}</div>
    </body>
    </html>
    `;
}
function getMethodValidationWebviewContent(methodCall, result) {
    return `
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>BSL Method Validation</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                margin: 20px;
                background-color: var(--vscode-editor-background);
                color: var(--vscode-editor-foreground);
            }
            .validation-header {
                border-bottom: 2px solid var(--vscode-panel-border);
                padding-bottom: 16px;
                margin-bottom: 20px;
            }
            .validation-title {
                font-size: 24px;
                font-weight: bold;
                color: var(--vscode-charts-red);
            }
            .method-call-info {
                background: var(--vscode-badge-background);
                color: var(--vscode-badge-foreground);
                padding: 8px 12px;
                border-radius: 4px;
                margin: 8px 0;
                font-family: 'Consolas', 'Monaco', monospace;
            }
            .result-content {
                background: var(--vscode-editor-inactiveSelectionBackground);
                border: 1px solid var(--vscode-panel-border);
                border-radius: 6px;
                padding: 16px;
                white-space: pre-wrap;
                font-family: 'Consolas', 'Monaco', monospace;
                font-size: 14px;
                overflow-x: auto;
            }
        </style>
    </head>
    <body>
        <div class="validation-header">
            <div class="validation-title">✓ Method Validation</div>
            <div class="method-call-info">
                ${methodCall.typeName}.${methodCall.methodName}(${methodCall.parameters.join(', ')})
            </div>
        </div>
        <div class="result-content">${result}</div>
    </body>
    </html>
    `;
}
function getTypeCompatibilityWebviewContent(fromType, toType, result) {
    return `
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>BSL Type Compatibility</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                margin: 20px;
                background-color: var(--vscode-editor-background);
                color: var(--vscode-editor-foreground);
            }
            .compatibility-header {
                border-bottom: 2px solid var(--vscode-panel-border);
                padding-bottom: 16px;
                margin-bottom: 20px;
            }
            .compatibility-title {
                font-size: 24px;
                font-weight: bold;
                color: var(--vscode-charts-yellow);
            }
            .type-comparison {
                background: var(--vscode-badge-background);
                color: var(--vscode-badge-foreground);
                padding: 8px 12px;
                border-radius: 4px;
                margin: 8px 0;
                font-family: 'Consolas', 'Monaco', monospace;
                text-align: center;
            }
            .result-content {
                background: var(--vscode-editor-inactiveSelectionBackground);
                border: 1px solid var(--vscode-panel-border);
                border-radius: 6px;
                padding: 16px;
                white-space: pre-wrap;
                font-family: 'Consolas', 'Monaco', monospace;
                font-size: 14px;
                overflow-x: auto;
            }
        </style>
    </head>
    <body>
        <div class="compatibility-header">
            <div class="compatibility-title">↔️ Type Compatibility</div>
            <div class="type-comparison">
                ${fromType} → ${toType}
            </div>
        </div>
        <div class="result-content">${result}</div>
    </body>
    </html>
    `;
}
// Sidebar providers classes
class BslOverviewProvider {
    constructor() {
        this._onDidChangeTreeData = new vscode.EventEmitter();
        this.onDidChangeTreeData = this._onDidChangeTreeData.event;
        // Подписываемся на изменения прогресса индексации
        progressEmitter.event(() => {
            this.refresh();
        });
    }
    refresh() {
        this._onDidChangeTreeData.fire();
    }
    getTreeItem(element) {
        return element;
    }
    getChildren(element) {
        if (!element) {
            // Root items
            return Promise.resolve([
                new BslOverviewItem('Workspace Analysis', vscode.TreeItemCollapsibleState.Expanded, 'workspace'),
                new BslOverviewItem('LSP Server Status', vscode.TreeItemCollapsibleState.Expanded, 'server'),
                new BslOverviewItem('Configuration', vscode.TreeItemCollapsibleState.Expanded, 'config')
            ]);
        }
        else {
            switch (element.contextValue) {
                case 'workspace':
                    const workspaceItems = [
                        new BslOverviewItem('BSL Files: Scanning...', vscode.TreeItemCollapsibleState.None, 'file-count'),
                        new BslOverviewItem('Last Analysis: Never', vscode.TreeItemCollapsibleState.None, 'last-analysis'),
                        new BslOverviewItem('Issues Found: 0', vscode.TreeItemCollapsibleState.None, 'issues')
                    ];
                    // Добавляем информацию об индексации если она активна
                    if (globalIndexingProgress.isIndexing) {
                        const progressIcon = '$(loading~spin)';
                        const progressText = `${progressIcon} ${globalIndexingProgress.currentStep} (${globalIndexingProgress.progress}%)`;
                        const progressItem = new BslOverviewItem(progressText, vscode.TreeItemCollapsibleState.None, 'indexing-progress');
                        progressItem.tooltip = `Step ${globalIndexingProgress.currentStepNumber}/${globalIndexingProgress.totalSteps}${globalIndexingProgress.estimatedTimeRemaining ? `\nETA: ${globalIndexingProgress.estimatedTimeRemaining}` : ''}`;
                        workspaceItems.unshift(progressItem); // Добавляем в начало
                    }
                    return Promise.resolve(workspaceItems);
                case 'server':
                    // Более точная проверка статуса LSP сервера
                    let serverStatus = 'Stopped';
                    let debugInfo = '';
                    if (client) {
                        // Debug информация для диагностики
                        const hasIsRunning = client && typeof client.isRunning === 'function';
                        const isRunning = hasIsRunning && client ? client.isRunning() : false;
                        const clientState = client ? client.state : 'Not initialized';
                        debugInfo = `(isRunning: ${isRunning}, state: ${clientState})`;
                        outputChannel.appendLine(`LSP Status Debug: client exists, isRunning=${isRunning}, state=${clientState}`);
                        if (hasIsRunning && isRunning) {
                            serverStatus = 'Running';
                        }
                        else if (clientState !== undefined && clientState === 2) { // State.Running
                            serverStatus = 'Running';
                        }
                        else if (client && client.needsStart && !client.needsStart()) {
                            serverStatus = 'Running';
                        }
                    }
                    else {
                        outputChannel.appendLine('LSP Status Debug: client is null/undefined');
                        debugInfo = '(client not initialized)';
                    }
                    return Promise.resolve([
                        new BslOverviewItem(`Status: ${serverStatus}`, vscode.TreeItemCollapsibleState.None, 'status'),
                        new BslOverviewItem('UnifiedBslIndex: 13 types', vscode.TreeItemCollapsibleState.None, 'index-count'),
                        new BslOverviewItem('Platform: 8.3.25', vscode.TreeItemCollapsibleState.None, 'platform')
                    ]);
                case 'config':
                    const config = vscode.workspace.getConfiguration('bslAnalyzer');
                    const configPath = config.get('configurationPath', 'Not configured');
                    return Promise.resolve([
                        new BslOverviewItem(`Configuration: ${configPath}`, vscode.TreeItemCollapsibleState.None, 'config-path'),
                        new BslOverviewItem('Real-time Analysis: Enabled', vscode.TreeItemCollapsibleState.None, 'real-time'),
                        new BslOverviewItem('Metrics: Enabled', vscode.TreeItemCollapsibleState.None, 'metrics')
                    ]);
                default:
                    return Promise.resolve([]);
            }
        }
    }
}
class BslDiagnosticsProvider {
    constructor() {
        this._onDidChangeTreeData = new vscode.EventEmitter();
        this.onDidChangeTreeData = this._onDidChangeTreeData.event;
    }
    refresh() {
        this._onDidChangeTreeData.fire();
    }
    getTreeItem(element) {
        return element;
    }
    getChildren(element) {
        if (!element) {
            return Promise.resolve([
                new BslDiagnosticItem('Errors (0)', vscode.TreeItemCollapsibleState.Collapsed, 'errors'),
                new BslDiagnosticItem('Warnings (0)', vscode.TreeItemCollapsibleState.Collapsed, 'warnings'),
                new BslDiagnosticItem('Info (0)', vscode.TreeItemCollapsibleState.Collapsed, 'info')
            ]);
        }
        return Promise.resolve([]);
    }
}
class BslTypeIndexProvider {
    constructor() {
        this._onDidChangeTreeData = new vscode.EventEmitter();
        this.onDidChangeTreeData = this._onDidChangeTreeData.event;
    }
    refresh() {
        this._onDidChangeTreeData.fire();
    }
    getTreeItem(element) {
        return element;
    }
    getChildren(element) {
        if (!element) {
            return Promise.resolve([
                new BslTypeItem('Platform Types (8)', vscode.TreeItemCollapsibleState.Collapsed, 'platform'),
                new BslTypeItem('Configuration Types (5)', vscode.TreeItemCollapsibleState.Collapsed, 'configuration'),
                new BslTypeItem('Global Functions', vscode.TreeItemCollapsibleState.Collapsed, 'functions')
            ]);
        }
        else {
            switch (element.contextValue) {
                case 'platform':
                    return Promise.resolve([
                        new BslTypeItem('String', vscode.TreeItemCollapsibleState.None, 'type'),
                        new BslTypeItem('Number', vscode.TreeItemCollapsibleState.None, 'type'),
                        new BslTypeItem('Boolean', vscode.TreeItemCollapsibleState.None, 'type'),
                        new BslTypeItem('Date', vscode.TreeItemCollapsibleState.None, 'type'),
                        new BslTypeItem('Array', vscode.TreeItemCollapsibleState.None, 'type')
                    ]);
                case 'configuration':
                    return Promise.resolve([
                        new BslTypeItem('Catalogs.Контрагенты', vscode.TreeItemCollapsibleState.None, 'catalog'),
                        new BslTypeItem('Catalogs.Номенклатура', vscode.TreeItemCollapsibleState.None, 'catalog'),
                        new BslTypeItem('Documents.ЗаказНаряды', vscode.TreeItemCollapsibleState.None, 'document')
                    ]);
                case 'functions':
                    return Promise.resolve([
                        new BslTypeItem('Сообщить', vscode.TreeItemCollapsibleState.None, 'function'),
                        new BslTypeItem('СокрЛП', vscode.TreeItemCollapsibleState.None, 'function'),
                        new BslTypeItem('НачалоГода', vscode.TreeItemCollapsibleState.None, 'function')
                    ]);
                default:
                    return Promise.resolve([]);
            }
        }
    }
}
class BslPlatformDocsProvider {
    constructor() {
        this._onDidChangeTreeData = new vscode.EventEmitter();
        this.onDidChangeTreeData = this._onDidChangeTreeData.event;
    }
    refresh() {
        this._onDidChangeTreeData.fire();
    }
    getTreeItem(element) {
        return element;
    }
    getChildren(element) {
        if (!element) {
            // Показываем доступные версии платформы из кеша
            return this.getAvailablePlatformVersions();
        }
        else {
            // Показываем детали для конкретной версии
            const details = [];
            // Показываем количество типов
            details.push(new PlatformDocItem(`ℹ️ Types: ${element.typesCount || 'Unknown'}`, vscode.TreeItemCollapsibleState.None, element.version, 'info'));
            // Показываем информацию об архивах
            if (element.archiveName === 'Both archives') {
                details.push(new PlatformDocItem(`📂 Archive: shcntx_ru.zip`, vscode.TreeItemCollapsibleState.None, element.version, 'info'));
                details.push(new PlatformDocItem(`📂 Archive: shlang_ru.zip`, vscode.TreeItemCollapsibleState.None, element.version, 'info'));
                details.push(new PlatformDocItem(`✅ Status: Complete`, vscode.TreeItemCollapsibleState.None, element.version, 'info'));
            }
            else if (element.archiveName === 'shcntx_ru.zip') {
                details.push(new PlatformDocItem(`📂 Archive: ${element.archiveName}`, vscode.TreeItemCollapsibleState.None, element.version, 'info'));
                details.push(new PlatformDocItem(`⚠️ Missing: shlang_ru.zip`, vscode.TreeItemCollapsibleState.None, element.version, 'info'));
            }
            else if (element.archiveName === 'shlang_ru.zip') {
                details.push(new PlatformDocItem(`📂 Archive: ${element.archiveName}`, vscode.TreeItemCollapsibleState.None, element.version, 'info'));
                details.push(new PlatformDocItem(`⚠️ Missing: shcntx_ru.zip`, vscode.TreeItemCollapsibleState.None, element.version, 'info'));
            }
            else {
                details.push(new PlatformDocItem(`📦 Archive: ${element.archiveName || 'N/A'}`, vscode.TreeItemCollapsibleState.None, element.version, 'info'));
            }
            // Показываем дату парсинга
            details.push(new PlatformDocItem(`🕒 Parsed: ${element.lastParsed || 'Never'}`, vscode.TreeItemCollapsibleState.None, element.version, 'info'));
            return Promise.resolve(details);
        }
    }
    async getAvailablePlatformVersions() {
        const items = [];
        // Проверяем наличие кеша платформенной документации
        const homedir = require('os').homedir();
        const cacheDir = path.join(homedir, '.bsl_analyzer', 'platform_cache');
        if (fs.existsSync(cacheDir)) {
            // Читаем список версий из кеша
            const files = fs.readdirSync(cacheDir);
            const versionFiles = files.filter(f => f.match(/^v[\d.]+\.jsonl$/));
            for (const versionFile of versionFiles) {
                const version = versionFile.replace('v', '').replace('.jsonl', '');
                // Пытаемся прочитать количество типов из файла
                let typesCount = '?';
                let archiveInfo = 'Unknown';
                try {
                    const filePath = path.join(cacheDir, versionFile);
                    const content = fs.readFileSync(filePath, 'utf-8');
                    const lines = content.trim().split('\n');
                    typesCount = lines.length.toLocaleString();
                    // Анализируем содержимое для определения типа архивов
                    let hasObjectTypes = false;
                    let hasPrimitiveTypes = false;
                    for (const line of lines.slice(0, 100)) { // Проверяем первые 100 строк
                        try {
                            const entity = JSON.parse(line);
                            if (entity.name) {
                                // Проверка на объектные типы (из shcntx)
                                if (entity.name.includes('Массив') || entity.name.includes('Array') ||
                                    entity.name.includes('ТаблицаЗначений') || entity.name.includes('ValueTable')) {
                                    hasObjectTypes = true;
                                }
                                // Проверка на примитивные типы (из shlang)
                                if (entity.name === 'Число' || entity.name === 'Number' ||
                                    entity.name === 'Строка' || entity.name === 'String' ||
                                    entity.name === 'Булево' || entity.name === 'Boolean') {
                                    hasPrimitiveTypes = true;
                                }
                            }
                        }
                        catch (e) {
                            // Игнорируем ошибки парсинга
                        }
                    }
                    if (hasObjectTypes && hasPrimitiveTypes) {
                        archiveInfo = 'Both archives';
                    }
                    else if (hasObjectTypes) {
                        archiveInfo = 'shcntx_ru.zip';
                    }
                    else if (hasPrimitiveTypes) {
                        archiveInfo = 'shlang_ru.zip';
                    }
                }
                catch (e) {
                    outputChannel.appendLine(`Error reading platform cache: ${e}`);
                }
                const lastModified = fs.statSync(path.join(cacheDir, versionFile)).mtime.toLocaleDateString();
                items.push(new PlatformDocItem(`📋 Platform ${version}`, vscode.TreeItemCollapsibleState.Expanded, version, 'version', typesCount, archiveInfo, lastModified));
            }
        }
        // Всегда добавляем кнопку для добавления документации
        items.push(new PlatformDocItem('➕ Add Platform Documentation...', vscode.TreeItemCollapsibleState.None, '', 'add-docs'));
        return items;
    }
}
class PlatformDocItem extends vscode.TreeItem {
    constructor(label, collapsibleState, version, contextValue, typesCount, archiveName, lastParsed) {
        super(label, collapsibleState);
        this.label = label;
        this.collapsibleState = collapsibleState;
        this.version = version;
        this.contextValue = contextValue;
        this.typesCount = typesCount;
        this.archiveName = archiveName;
        this.lastParsed = lastParsed;
        this.contextValue = contextValue;
        if (contextValue === 'version') {
            this.tooltip = `Platform ${version} - ${typesCount} types`;
            this.iconPath = new vscode.ThemeIcon('database');
        }
        else if (contextValue === 'add-docs') {
            this.command = {
                command: 'bslAnalyzer.addPlatformDocs',
                title: 'Add Platform Documentation'
            };
            this.iconPath = new vscode.ThemeIcon('add');
        }
        else if (contextValue === 'info') {
            this.iconPath = new vscode.ThemeIcon('info');
        }
    }
}
class BslOverviewItem extends vscode.TreeItem {
    constructor(label, collapsibleState, contextValue) {
        super(label, collapsibleState);
        this.label = label;
        this.collapsibleState = collapsibleState;
        this.contextValue = contextValue;
        this.tooltip = `${this.label}`;
        // Add icons based on context
        switch (contextValue) {
            case 'workspace':
                this.iconPath = new vscode.ThemeIcon('folder');
                break;
            case 'server':
                this.iconPath = new vscode.ThemeIcon('server');
                break;
            case 'config':
                this.iconPath = new vscode.ThemeIcon('gear');
                break;
            default:
                this.iconPath = new vscode.ThemeIcon('info');
        }
    }
}
class BslDiagnosticItem extends vscode.TreeItem {
    constructor(label, collapsibleState, contextValue) {
        super(label, collapsibleState);
        this.label = label;
        this.collapsibleState = collapsibleState;
        this.contextValue = contextValue;
        this.tooltip = `${this.label}`;
        // Add icons based on type
        switch (contextValue) {
            case 'errors':
                this.iconPath = new vscode.ThemeIcon('error');
                break;
            case 'warnings':
                this.iconPath = new vscode.ThemeIcon('warning');
                break;
            case 'info':
                this.iconPath = new vscode.ThemeIcon('info');
                break;
        }
    }
}
class BslTypeItem extends vscode.TreeItem {
    constructor(label, collapsibleState, contextValue) {
        super(label, collapsibleState);
        this.label = label;
        this.collapsibleState = collapsibleState;
        this.contextValue = contextValue;
        this.tooltip = `${this.label}`;
        // Add icons based on type
        switch (contextValue) {
            case 'platform':
                this.iconPath = new vscode.ThemeIcon('library');
                break;
            case 'configuration':
                this.iconPath = new vscode.ThemeIcon('database');
                break;
            case 'functions':
                this.iconPath = new vscode.ThemeIcon('symbol-function');
                break;
            case 'type':
                this.iconPath = new vscode.ThemeIcon('symbol-class');
                break;
            case 'catalog':
                this.iconPath = new vscode.ThemeIcon('symbol-object');
                break;
            case 'document':
                this.iconPath = new vscode.ThemeIcon('symbol-file');
                break;
            case 'function':
                this.iconPath = new vscode.ThemeIcon('symbol-method');
                break;
        }
    }
}
function registerSidebarProviders(context) {
    outputChannel.appendLine('📋 Registering BSL Analyzer sidebar providers...');
    console.log('Registering sidebar providers');
    try {
        // Overview provider
        outputChannel.appendLine('📋 Creating Overview provider...');
        const overviewProvider = new BslOverviewProvider();
        const overviewTreeView = vscode.window.createTreeView('bslAnalyzer.overview', {
            treeDataProvider: overviewProvider,
            showCollapseAll: true
        });
        context.subscriptions.push(overviewTreeView);
        outputChannel.appendLine('✅ Overview provider registered');
        // Diagnostics provider  
        outputChannel.appendLine('📋 Creating Diagnostics provider...');
        const diagnosticsProvider = new BslDiagnosticsProvider();
        const diagnosticsTreeView = vscode.window.createTreeView('bslAnalyzer.diagnostics', {
            treeDataProvider: diagnosticsProvider,
            showCollapseAll: true
        });
        context.subscriptions.push(diagnosticsTreeView);
        outputChannel.appendLine('✅ Diagnostics provider registered');
        // Type Index provider
        outputChannel.appendLine('📋 Creating Type Index provider...');
        const typeIndexProvider = new BslTypeIndexProvider();
        const typeIndexTreeView = vscode.window.createTreeView('bslAnalyzer.typeIndex', {
            treeDataProvider: typeIndexProvider,
            showCollapseAll: true
        });
        context.subscriptions.push(typeIndexTreeView);
        outputChannel.appendLine('✅ Type Index provider registered');
        // Platform Documentation provider
        outputChannel.appendLine('📋 Creating Platform Documentation provider...');
        const platformDocsProvider = new BslPlatformDocsProvider();
        const platformDocsTreeView = vscode.window.createTreeView('bslAnalyzer.platformDocs', {
            treeDataProvider: platformDocsProvider,
            showCollapseAll: true
        });
        context.subscriptions.push(platformDocsTreeView);
        outputChannel.appendLine('✅ Platform Documentation provider registered');
        // Quick Actions webview provider
        outputChannel.appendLine('📋 Creating Quick Actions webview provider...');
        const actionsProvider = new BslActionsWebviewProvider(context.extensionUri);
        const webviewProvider = vscode.window.registerWebviewViewProvider('bslAnalyzer.actions', actionsProvider);
        context.subscriptions.push(webviewProvider);
        outputChannel.appendLine('✅ Quick Actions webview provider registered');
        // Register refresh commands
        context.subscriptions.push(vscode.commands.registerCommand('bslAnalyzer.refreshOverview', () => {
            outputChannel.appendLine('🔄 Refreshing Overview panel');
            overviewProvider.refresh();
        }));
        context.subscriptions.push(vscode.commands.registerCommand('bslAnalyzer.refreshDiagnostics', () => {
            outputChannel.appendLine('🔄 Refreshing Diagnostics panel');
            diagnosticsProvider.refresh();
        }));
        context.subscriptions.push(vscode.commands.registerCommand('bslAnalyzer.refreshTypeIndex', () => {
            outputChannel.appendLine('🔄 Refreshing Type Index panel');
            typeIndexProvider.refresh();
        }));
        context.subscriptions.push(vscode.commands.registerCommand('bslAnalyzer.refreshPlatformDocs', () => {
            outputChannel.appendLine('🔄 Refreshing Platform Docs panel');
            platformDocsProvider.refresh();
        }));
        context.subscriptions.push(vscode.commands.registerCommand('bslAnalyzer.addPlatformDocs', async () => {
            outputChannel.appendLine('📁 Adding platform documentation...');
            await addPlatformDocumentation(platformDocsProvider);
        }));
        context.subscriptions.push(vscode.commands.registerCommand('bslAnalyzer.removePlatformDocs', async (item) => {
            if (item && item.version) {
                outputChannel.appendLine(`🗑️ Removing platform docs for version: ${item.version}`);
                await removePlatformDocumentation(item.version, platformDocsProvider);
            }
        }));
        context.subscriptions.push(vscode.commands.registerCommand('bslAnalyzer.parsePlatformDocs', async (item) => {
            if (item && item.version) {
                outputChannel.appendLine(`⚙️ Parsing platform docs for version: ${item.version}`);
                await parsePlatformDocumentation(item.version);
            }
        }));
        outputChannel.appendLine('✅ All BSL Analyzer sidebar providers registered successfully');
        // Показываем уведомление об успешной регистрации
        vscode.window.showInformationMessage('BSL Analyzer sidebar activated! Check the Activity Bar for the BSL Analyzer icon.');
    }
    catch (error) {
        outputChannel.appendLine(`❌ Error registering sidebar providers: ${error}`);
        console.error('Error registering sidebar providers:', error);
        vscode.window.showErrorMessage(`Failed to register BSL Analyzer sidebar: ${error}`);
    }
}
class BslActionsWebviewProvider {
    constructor(extensionUri) {
        this.extensionUri = extensionUri;
    }
    resolveWebviewView(webviewView) {
        webviewView.webview.options = {
            enableScripts: true,
            localResourceRoots: [this.extensionUri]
        };
        webviewView.webview.html = this.getWebviewContent();
        // Handle messages from webview
        webviewView.webview.onDidReceiveMessage(async (message) => {
            switch (message.command) {
                case 'analyzeCurrentFile':
                    vscode.commands.executeCommand('bslAnalyzer.analyzeCurrentFile');
                    break;
                case 'buildIndex':
                    vscode.commands.executeCommand('bslAnalyzer.buildIndex');
                    break;
                case 'showMetrics':
                    vscode.commands.executeCommand('bslAnalyzer.showMetrics');
                    break;
                case 'openSettings':
                    vscode.commands.executeCommand('workbench.action.openSettings', 'bslAnalyzer');
                    break;
            }
        });
    }
    getWebviewContent() {
        return `
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>BSL Analyzer Actions</title>
            <style>
                body {
                    font-family: var(--vscode-font-family);
                    font-size: var(--vscode-font-size);
                    color: var(--vscode-foreground);
                    background-color: var(--vscode-editor-background);
                    margin: 0;
                    padding: 16px;
                }
                .action-button {
                    display: block;
                    width: 100%;
                    padding: 8px 12px;
                    margin: 8px 0;
                    background-color: var(--vscode-button-background);
                    color: var(--vscode-button-foreground);
                    border: none;
                    border-radius: 4px;
                    cursor: pointer;
                    text-align: left;
                    font-size: 13px;
                }
                .action-button:hover {
                    background-color: var(--vscode-button-hoverBackground);
                }
                .action-button:active {
                    transform: translateY(1px);
                }
                .section-title {
                    font-weight: bold;
                    margin: 16px 0 8px 0;
                    color: var(--vscode-descriptionForeground);
                    font-size: 11px;
                    text-transform: uppercase;
                }
                .icon {
                    margin-right: 8px;
                }
            </style>
        </head>
        <body>
            <div class="section-title">Analysis</div>
            <button class="action-button" onclick="sendMessage('analyzeCurrentFile')">
                <span class="icon">🔍</span>Analyze Current File
            </button>
            <button class="action-button" onclick="sendMessage('buildIndex')">
                <span class="icon">📋</span>Build Type Index
            </button>
            <button class="action-button" onclick="sendMessage('showMetrics')">
                <span class="icon">📊</span>Show Metrics
            </button>
            
            <div class="section-title">Configuration</div>
            <button class="action-button" onclick="sendMessage('openSettings')">
                <span class="icon">⚙️</span>Open Settings
            </button>

            <script>
                const vscode = acquireVsCodeApi();
                
                function sendMessage(command) {
                    vscode.postMessage({ command: command });
                }
            </script>
        </body>
        </html>
        `;
    }
}
// Функции для работы с платформенной документацией
async function addPlatformDocumentation(provider) {
    try {
        // 1. Спросим у пользователя версию платформы
        const version = await vscode.window.showInputBox({
            prompt: 'Enter platform version (e.g., 8.3.25)',
            placeHolder: '8.3.25',
            value: '8.3.25'
        });
        if (!version) {
            return;
        }
        // 2. Выберем архив с документацией
        const archiveFiles = await vscode.window.showOpenDialog({
            canSelectFiles: true,
            canSelectMany: false,
            filters: {
                'Help Archives': ['zip']
            },
            openLabel: 'Select Platform Documentation Archive (shcntx or shlang)'
        });
        if (!archiveFiles || archiveFiles.length === 0) {
            return;
        }
        const archivePath = archiveFiles[0].fsPath;
        const archiveDir = path.dirname(archivePath);
        const archiveName = path.basename(archivePath);
        // Определяем тип архива и ищем companion архив
        let shcntxPath;
        let shlangPath;
        let totalTypesCount = 0;
        if (archiveName.includes('shcntx')) {
            shcntxPath = archivePath;
            // Ищем shlang архив в той же папке
            const possibleShlangFiles = [
                'rebuilt.shlang_ru.zip',
                'shlang_ru.zip',
                archiveName.replace('shcntx', 'shlang')
            ];
            for (const shlangFile of possibleShlangFiles) {
                const shlangFullPath = path.join(archiveDir, shlangFile);
                if (fs.existsSync(shlangFullPath)) {
                    shlangPath = shlangFullPath;
                    outputChannel.appendLine(`📂 Found companion archive: ${shlangFile}`);
                    break;
                }
            }
        }
        else if (archiveName.includes('shlang')) {
            shlangPath = archivePath;
            // Ищем shcntx архив в той же папке
            const possibleShcntxFiles = [
                'rebuilt.shcntx_ru.zip',
                'shcntx_ru.zip',
                archiveName.replace('shlang', 'shcntx')
            ];
            for (const shcntxFile of possibleShcntxFiles) {
                const shcntxFullPath = path.join(archiveDir, shcntxFile);
                if (fs.existsSync(shcntxFullPath)) {
                    shcntxPath = shcntxFullPath;
                    outputChannel.appendLine(`📂 Found companion archive: ${shcntxFile}`);
                    break;
                }
            }
        }
        // 3. Выполним парсинг через бинарь с прогрессом
        const stepsCount = (shcntxPath && shlangPath) ? 5 : 3; // Больше шагов если есть оба архива
        startIndexing(stepsCount);
        vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: `Adding platform documentation for version ${version}...`,
            cancellable: false
        }, async (progress) => {
            try {
                let currentStep = 1;
                // Обрабатываем shcntx архив (основные типы и методы)
                if (shcntxPath) {
                    updateIndexingProgress(currentStep++, 'Processing shcntx archive (types & methods)...', 25);
                    progress.report({ increment: 25, message: 'Processing main types archive...' });
                    const shcntxResult = await executeBslCommand('extract_platform_docs', [
                        '--archive', `"${shcntxPath}"`,
                        '--platform-version', version
                    ]);
                    const shcntxTypesMatch = shcntxResult.match(/(\d+)\s+types/i) || shcntxResult.match(/(\d+)\s+entities/i);
                    if (shcntxTypesMatch) {
                        totalTypesCount += parseInt(shcntxTypesMatch[1]);
                    }
                    outputChannel.appendLine(`✅ shcntx processed: ${shcntxTypesMatch ? shcntxTypesMatch[1] : '?'} types`);
                }
                // Обрабатываем shlang архив (примитивные типы)
                if (shlangPath) {
                    updateIndexingProgress(currentStep++, 'Processing shlang archive (primitive types)...', 50);
                    progress.report({ increment: 25, message: 'Processing primitive types archive...' });
                    const shlangResult = await executeBslCommand('extract_platform_docs', [
                        '--archive', `"${shlangPath}"`,
                        '--platform-version', version,
                        '--force' // Форсируем обновление для добавления примитивных типов
                    ]);
                    const shlangTypesMatch = shlangResult.match(/(\d+)\s+types/i) || shlangResult.match(/(\d+)\s+entities/i);
                    if (shlangTypesMatch) {
                        totalTypesCount += parseInt(shlangTypesMatch[1]);
                    }
                    outputChannel.appendLine(`✅ shlang processed: ${shlangTypesMatch ? shlangTypesMatch[1] : '?'} types`);
                }
                // Финализация
                updateIndexingProgress(currentStep++, 'Finalizing...', 95);
                progress.report({ increment: 20, message: 'Finalizing...' });
                finishIndexing(true);
                // Формируем сообщение о результате
                let message = `✅ Platform documentation added for version ${version}`;
                if (shcntxPath && shlangPath) {
                    message += ` (${totalTypesCount} total types from both archives)`;
                }
                else if (shcntxPath) {
                    message += ` (${totalTypesCount} types from shcntx)`;
                    if (!shlangPath) {
                        message += '\n⚠️ Note: shlang archive not found - primitive types may be incomplete';
                    }
                }
                else if (shlangPath) {
                    message += ` (${totalTypesCount} primitive types from shlang)`;
                    if (!shcntxPath) {
                        message += '\n⚠️ Note: shcntx archive not found - object types may be incomplete';
                    }
                }
                vscode.window.showInformationMessage(message);
                outputChannel.appendLine(message);
                // Обновляем панель
                provider.refresh();
            }
            catch (error) {
                finishIndexing(false);
                vscode.window.showErrorMessage(`Failed to add platform documentation: ${error}`);
                outputChannel.appendLine(`Error adding platform docs: ${error}`);
            }
        });
    }
    catch (error) {
        vscode.window.showErrorMessage(`Failed to add platform documentation: ${error}`);
        outputChannel.appendLine(`Error: ${error}`);
    }
}
async function removePlatformDocumentation(version, provider) {
    const choice = await vscode.window.showWarningMessage(`Are you sure you want to remove platform documentation for version ${version}?`, { modal: true }, 'Remove');
    if (choice === 'Remove') {
        try {
            // TODO: Реализовать удаление кеша платформенных типов
            // Пока что просто показываем уведомление
            vscode.window.showInformationMessage(`Platform documentation removal for version ${version} - feature coming soon!`);
            outputChannel.appendLine(`Platform docs removal requested for ${version} - not implemented yet`);
            // Обновляем панель
            provider.refresh();
        }
        catch (error) {
            vscode.window.showErrorMessage(`Failed to remove platform documentation: ${error}`);
            outputChannel.appendLine(`Error removing platform docs: ${error}`);
        }
    }
}
async function parsePlatformDocumentation(version) {
    startIndexing(3); // 3 этапа для ре-парсинга
    vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: `Re-parsing platform documentation for version ${version}...`,
        cancellable: false
    }, async (progress) => {
        try {
            // Этап 1: Инициализация
            updateIndexingProgress(1, 'Initializing re-parse...', 15);
            progress.report({ increment: 30, message: 'Initializing re-parse...' });
            // Этап 2: Построение индекса
            updateIndexingProgress(2, 'Building unified index...', 70);
            progress.report({ increment: 55, message: 'Building unified index...' });
            const args = [
                '--platform-version', version,
                '--force-rebuild'
            ];
            const platformDocsArchive = getPlatformDocsArchive();
            if (platformDocsArchive) {
                args.push('--platform-docs-archive', platformDocsArchive);
            }
            const result = await executeBslCommand('build_unified_index', args);
            // Этап 3: Завершение
            updateIndexingProgress(3, 'Finalizing...', 95);
            progress.report({ increment: 15, message: 'Finalizing...' });
            finishIndexing(true);
            vscode.window.showInformationMessage(`✅ Platform documentation re-parsed successfully for version ${version}`);
            outputChannel.appendLine(`Re-parse result: ${result}`);
        }
        catch (error) {
            finishIndexing(false);
            vscode.window.showErrorMessage(`Failed to re-parse platform documentation: ${error}`);
            outputChannel.appendLine(`Error re-parsing platform docs: ${error}`);
        }
    });
}
async function deactivate() {
    if (!client) {
        return;
    }
    try {
        // Give the client time to shut down gracefully
        const timeoutPromise = new Promise((resolve) => {
            setTimeout(() => {
                outputChannel.appendLine('⚠️ LSP client shutdown timeout reached');
                resolve();
            }, 5000);
        });
        const stopPromise = client.stop().then(() => {
            outputChannel.appendLine('✅ LSP client stopped successfully');
        }).catch(error => {
            outputChannel.appendLine(`⚠️ Error stopping LSP client: ${error.message}`);
        });
        // Wait for either stop to complete or timeout
        await Promise.race([stopPromise, timeoutPromise]);
    }
    catch (error) {
        outputChannel.appendLine(`⚠️ Error during deactivation: ${error}`);
    }
    finally {
        client = null;
        outputChannel.appendLine('👋 BSL Analyzer extension deactivated');
        outputChannel.dispose();
    }
}
exports.deactivate = deactivate;
//# sourceMappingURL=extension.js.map