import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';

// Импорт из новых модулей
import { BslAnalyzerConfig, migrateLegacySettings } from './config';
import {
    initializeLspClient,
    startLanguageClient,
    stopLanguageClient,
    getLanguageClient
} from './lsp';
import {
    initializeProgress,
    startIndexing,
    updateIndexingProgress,
    finishIndexing,
    updateStatusBar
} from './lsp/progress';
import {
    executeBslCommand,
    getPlatformDocsArchive,
    initializeUtils,
    autoDetectConfiguration
} from './utils';
import {
    BslOverviewProvider,
    BslDiagnosticsProvider,
    BslPlatformDocsProvider,
    HierarchicalTypeIndexProvider,
    BslActionsWebviewProvider
} from './providers';
// Webview функции не используются напрямую в extension.ts
// Они используются в модуле commands
import { registerCommands as registerAllCommands, initializeCommands } from './commands';
import {
    initializePlatformDocs,
    addPlatformDocumentation,
    removePlatformDocumentation,
    parsePlatformDocumentation
} from './platformDocs';

// Глобальные переменные
let indexServerPath: string;
let outputChannel: vscode.OutputChannel;
let statusBarItem: vscode.StatusBarItem;
let extensionContext: vscode.ExtensionContext;

// Функции прогресса теперь импортируются из модуля lsp/progress

export async function activate(context: vscode.ExtensionContext) {
    extensionContext = context;

    try {
        // Get the current version from package.json
        const currentVersion = context.extension.packageJSON.version;

        // Context is passed directly to functions that need it

        // Initialize output channel
        outputChannel = vscode.window.createOutputChannel('BSL Analyzer');
        context.subscriptions.push(outputChannel);

        outputChannel.appendLine(`🚀 BSL Analyzer v${currentVersion} activation started (with modular architecture)`);
        outputChannel.appendLine(`Extension path: ${context.extensionPath}`);

        // Show immediate notification for debugging
        vscode.window.showInformationMessage(`BSL Analyzer v${currentVersion} is activating...`);
        outputChannel.show(); // Показываем Output канал для отладки

        // Create status bar item first
        statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
        statusBarItem.command = 'bslAnalyzer.analyzeFile';
        statusBarItem.text = 'BSL Analyzer: Starting...';
        statusBarItem.tooltip = 'Click to analyze current file (via LSP)';
        statusBarItem.show();
        context.subscriptions.push(statusBarItem);

        // Инициализируем модули
        initializeUtils(outputChannel);
        initializeProgress(outputChannel, statusBarItem);
        initializeLspClient(outputChannel);
        initializeCommands(outputChannel);
        initializePlatformDocs(outputChannel);

        // Migrate legacy settings if needed
        await migrateLegacySettings();

        // Initialize configuration
        initializeConfiguration();

        // Auto-detect configuration if not set
        await autoDetectConfigurationIfNeeded();

        // Start LSP client FIRST (it may register some commands)
        // Запускаем с задержкой для стабильности
        setTimeout(async () => {
            outputChannel.appendLine('🚀 Starting LSP server with delay...');
            await startLanguageClient(context);
            // Обновляем статус бар после успешного запуска
            updateStatusBar('$(database) BSL Analyzer: Ready');
        }, 1000);

        // Register sidebar providers
        registerSidebarProviders(context);

        // Register our custom commands AFTER LSP client
        await registerAllCommands(context);

        // Auto-initialize index if configured
        initializeIndexIfNeeded();

        // Show welcome message
        showWelcomeMessage();

        outputChannel.appendLine(`✅ BSL Analyzer v${currentVersion} activated successfully with auto-indexing support`);

    } catch (error) {
        outputChannel?.appendLine(`❌ Activation failed: ${error}`);
        vscode.window.showErrorMessage(`BSL Analyzer activation failed: ${error}`);
    }
}


function initializeConfiguration() {
    indexServerPath = BslAnalyzerConfig.binaryPath;

    if (!indexServerPath) {
        // First, try bundled binaries from extension context
        // Use extensionContext which is available globally in this scope
        const extensionPath = extensionContext?.extensionPath;
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

async function autoDetectConfigurationIfNeeded() {
    const configPath = BslAnalyzerConfig.configurationPath;

    if (!configPath) {
        outputChannel.appendLine('📍 Configuration path not set, attempting auto-detection...');
        const detectedPath = await autoDetectConfiguration(outputChannel);

        if (detectedPath) {
            outputChannel.appendLine(`✅ Configuration auto-detected: ${detectedPath}`);
            // Refresh providers to use new configuration
            vscode.commands.executeCommand('bslAnalyzer.refreshTypeIndex');
        }
    } else {
        outputChannel.appendLine(`📍 Using configured path: ${configPath}`);
    }
}

async function initializeIndexIfNeeded() {
    const autoIndexBuild = BslAnalyzerConfig.autoIndexBuild;
    const configPath = BslAnalyzerConfig.configurationPath;

    if (!autoIndexBuild) {
        outputChannel.appendLine('ℹ️ Auto-index build is disabled');
        return;
    }

    if (!configPath) {
        outputChannel.appendLine('⚠️ Configuration path not set - user must configure it');
        updateStatusBar('BSL Analyzer: No Config');
        return;
    }

    // Check if we have valid UUID for this configuration
    const projectId = extractUuidProjectId(configPath);
    if (!projectId) {
        outputChannel.appendLine('❌ Cannot find UUID in Configuration.xml - no fallback, index cannot be built');
        updateStatusBar('BSL Analyzer: Invalid Config');
        return;
    }

    // Check if index already exists in cache
    const platformVersion = BslAnalyzerConfig.platformVersion;
    const indexCachePath = path.join(
        require('os').homedir(),
        '.bsl_analyzer',
        'project_indices',
        projectId,
        platformVersion
    );

    if (fs.existsSync(path.join(indexCachePath, 'unified_index.json'))) {
        outputChannel.appendLine(`✅ Index found in cache: ${projectId}/${platformVersion}`);
        updateStatusBar('BSL Analyzer: Index Ready');
        return;
    }

    // Check if platform documentation is configured
    const platformDocsArchive = getPlatformDocsArchive();
    if (!platformDocsArchive) {
        outputChannel.appendLine('❌ Platform documentation not configured - cannot build full index');
        outputChannel.appendLine('💡 User must specify platform documentation archive in settings');
        updateStatusBar('BSL Analyzer: No Platform Docs');
        // Don't build index without platform docs - it would be incomplete
        return;
    }

    outputChannel.appendLine('🚀 Building BSL index with user-configured settings...');

    // Build index with user-provided configuration
    try {
        startIndexing(4);

        await vscode.window.withProgress({
            location: vscode.ProgressLocation.Notification,
            title: 'Building BSL Index',
            cancellable: false
        }, async (progress) => {
            updateIndexingProgress(1, 'Loading platform documentation...', 10);
            progress.report({ increment: 25, message: 'Loading platform documentation...' });

            updateIndexingProgress(2, 'Parsing configuration...', 35);
            progress.report({ increment: 25, message: 'Parsing configuration...' });

            updateIndexingProgress(3, 'Building unified index...', 70);
            progress.report({ increment: 35, message: 'Building unified index...' });

            outputChannel.appendLine(`📁 Configuration: ${configPath}`);
            outputChannel.appendLine(`📚 Platform docs: ${platformDocsArchive}`);
            outputChannel.appendLine(`🔢 Platform version: ${platformVersion}`);
            
            const args = [
                '--config', configPath,
                '--platform-version', platformVersion,
                '--platform-docs-archive', platformDocsArchive
            ];

            await executeBslCommand('build_unified_index', args);

            updateIndexingProgress(4, 'Finalizing index...', 90);
            progress.report({ increment: 15, message: 'Finalizing...' });

            finishIndexing(true);

            outputChannel.appendLine('✅ Index build completed successfully');
            updateStatusBar('BSL Analyzer: Index Ready');
        });
    } catch (error) {
        finishIndexing(false);
        outputChannel.appendLine(`❌ Index build failed: ${error}`);
        updateStatusBar('BSL Analyzer: Build Failed');
    }
}

function showWelcomeMessage() {
    const configPath = BslAnalyzerConfig.configurationPath;
    const platformDocs = BslAnalyzerConfig.platformDocsArchive;

    if (!configPath && !platformDocs) {
        vscode.window.showInformationMessage(
            'BSL Analyzer: No configuration. Please configure 1C path and platform documentation.',
            'Open Settings'
        ).then(selection => {
            if (selection === 'Open Settings') {
                vscode.commands.executeCommand('workbench.action.openSettings', 'bslAnalyzer');
            }
        });
    } else if (!configPath) {
        vscode.window.showInformationMessage(
            'BSL Analyzer: Please configure your 1C configuration path.',
            'Open Settings'
        ).then(selection => {
            if (selection === 'Open Settings') {
                vscode.commands.executeCommand('workbench.action.openSettings', 'bslAnalyzer.configurationPath');
            }
        });
    } else if (!platformDocs) {
        vscode.window.showInformationMessage(
            'BSL Analyzer: Please configure platform documentation archive for full functionality.',
            'Open Settings'
        ).then(selection => {
            if (selection === 'Open Settings') {
                vscode.commands.executeCommand('workbench.action.openSettings', 'bslAnalyzer.platformDocsArchive');
            }
        });
    } else {
        // Everything is configured
        const homedir = require('os').homedir();
        const platformVersion = BslAnalyzerConfig.platformVersion;
        const projectId = extractUuidProjectId(configPath);
        
        if (projectId) {
            const indexPath = path.join(homedir, '.bsl_analyzer', 'project_indices', projectId, platformVersion, 'unified_index.json');
            if (fs.existsSync(indexPath)) {
                vscode.window.showInformationMessage('BSL Analyzer: Index loaded from cache. Ready to use!');
            } else {
                vscode.window.showInformationMessage('BSL Analyzer: Configuration detected. Building index...');
            }
        } else {
            vscode.window.showWarningMessage('BSL Analyzer: Invalid configuration (no UUID). Please check your Configuration.xml');
        }
    }
}

// UUID-based project identifier (must match Rust naming scheme; no fallback)
function extractUuidProjectId(configPath: string): string | null {
    try {
        const cfgXml = path.join(configPath, 'Configuration.xml');
        if (!fs.existsSync(cfgXml)) return null;
        const content = fs.readFileSync(cfgXml, 'utf-8');
        const m = content.match(/<Configuration[^>]*uuid="([^"]+)"/i);
        if (m && m[1]) {
            const uuid = m[1].replace(/-/g, '');
            return `${path.basename(configPath)}_${uuid}`;
        }
    } catch (e) {
        outputChannel.appendLine(`Failed to extract UUID: ${e}`);
    }
    return null;
}

// Все функции организованы в модули:
// - LSP клиент в модуле lsp/
// - Webview функции в модуле webviews/
// - Провайдеры в модуле providers/
// - Команды в модуле commands/
// - Утилиты в модуле utils/

function registerSidebarProviders(context: vscode.ExtensionContext) {
    outputChannel.appendLine('📋 Registering BSL Analyzer sidebar providers...');

    try {
        // Overview provider
        outputChannel.appendLine('📋 Creating Overview provider...');
        const overviewProvider = new BslOverviewProvider(outputChannel);
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

        // Type Index provider - используем новый иерархический провайдер
        outputChannel.appendLine('📋 Creating Hierarchical Type Index provider...');
        const typeIndexProvider = new HierarchicalTypeIndexProvider(outputChannel);
        const typeIndexTreeView = vscode.window.createTreeView('bslAnalyzer.typeIndex', {
            treeDataProvider: typeIndexProvider,
            showCollapseAll: true
        });
        context.subscriptions.push(typeIndexTreeView);
        outputChannel.appendLine('✅ Hierarchical Type Index provider registered');

        // Platform Documentation provider
        outputChannel.appendLine('📋 Creating Platform Documentation provider...');
        const platformDocsProvider = new BslPlatformDocsProvider(outputChannel);
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
        context.subscriptions.push(
            vscode.commands.registerCommand('bslAnalyzer.refreshOverview', () => {
                outputChannel.appendLine('🔄 Refreshing Overview panel');
                overviewProvider.refresh();
            })
        );

        context.subscriptions.push(
            vscode.commands.registerCommand('bslAnalyzer.refreshDiagnostics', () => {
                outputChannel.appendLine('🔄 Refreshing Diagnostics panel');
                diagnosticsProvider.refresh();
            })
        );

        context.subscriptions.push(
            vscode.commands.registerCommand('bslAnalyzer.refreshTypeIndex', () => {
                outputChannel.appendLine('🔄 Refreshing Type Index panel');
                typeIndexProvider.refresh();
            })
        );

        context.subscriptions.push(
            vscode.commands.registerCommand('bslAnalyzer.refreshPlatformDocs', () => {
                outputChannel.appendLine('🔄 Refreshing Platform Docs panel');
                platformDocsProvider.refresh();
            })
        );

        // Регистрируем команду добавления документации
        outputChannel.appendLine('Registering bslAnalyzer.addPlatformDocs command...');
        try {
            const addDocsDisposable = vscode.commands.registerCommand('bslAnalyzer.addPlatformDocs', async () => {
                outputChannel.appendLine('📁 Command executed: Adding platform documentation...');
                await addPlatformDocumentation(platformDocsProvider);
            });
            context.subscriptions.push(addDocsDisposable);
            outputChannel.appendLine('✅ Successfully registered bslAnalyzer.addPlatformDocs');
        } catch (error) {
            outputChannel.appendLine(`❌ Failed to register bslAnalyzer.addPlatformDocs: ${error}`);
        }

        context.subscriptions.push(
            vscode.commands.registerCommand('bslAnalyzer.removePlatformDocs', async (item) => {
                if (item && item.version) {
                    outputChannel.appendLine(`🗑️ Removing platform docs for version: ${item.version}`);
                    await removePlatformDocumentation(item.version, platformDocsProvider);
                }
            })
        );

        context.subscriptions.push(
            vscode.commands.registerCommand('bslAnalyzer.parsePlatformDocs', async (item) => {
                if (item && item.version) {
                    outputChannel.appendLine(`⚙️ Parsing platform docs for version: ${item.version}`);
                    await parsePlatformDocumentation(item.version);
                }
            })
        );

        outputChannel.appendLine('✅ All BSL Analyzer sidebar providers registered successfully');

        // Показываем уведомление об успешной регистрации
        vscode.window.showInformationMessage('BSL Analyzer sidebar activated! Check the Activity Bar for the BSL Analyzer icon.');

    } catch (error) {
        outputChannel.appendLine(`❌ Error registering sidebar providers: ${error}`);
        vscode.window.showErrorMessage(`Failed to register BSL Analyzer sidebar: ${error}`);
    }
}



// Функции платформенной документации перенесены в модуль platformDocs

export async function deactivate(): Promise<void> {
    const client = getLanguageClient();
    if (!client) {
        return;
    }

    try {
        // Give the client time to shut down gracefully
        const timeoutPromise = new Promise<void>((resolve) => {
            setTimeout(() => {
                outputChannel.appendLine('⚠️ LSP client shutdown timeout reached');
                resolve();
            }, 5000);
        });

        const stopPromise = stopLanguageClient().then(() => {
            outputChannel.appendLine('✅ LSP client stopped successfully');
        }).catch(error => {
            outputChannel.appendLine(`⚠️ Error stopping LSP client: ${error}`);
        });

        // Wait for either stop to complete or timeout
        await Promise.race([stopPromise, timeoutPromise]);

    } catch (error) {
        outputChannel.appendLine(`⚠️ Error during deactivation: ${error}`);
    } finally {
        outputChannel.appendLine('👋 BSL Analyzer extension deactivated');
        outputChannel.dispose();
    }
}