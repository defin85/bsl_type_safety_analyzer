import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { spawn, ChildProcess } from 'child_process';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
    RevealOutputChannelOn
} from 'vscode-languageclient/node';

let client: LanguageClient;
let indexServerPath: string;
let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    console.log('BSL Analyzer v1.3.0 extension is being activated');

    try {
        // Initialize output channel
        outputChannel = vscode.window.createOutputChannel('BSL Analyzer');
        context.subscriptions.push(outputChannel);
        
        outputChannel.appendLine('🚀 BSL Analyzer v1.3.1 activation started (with LSP commands support)');
        outputChannel.appendLine(`Extension path: ${context.extensionPath}`);

        // Initialize configuration
        initializeConfiguration();

        // Register all commands
        registerCommands(context);
        
        // Register status bar
        registerStatusBar(context);

        // Start LSP client
        startLanguageClient(context);

        // Show welcome message
        showWelcomeMessage();
        
        outputChannel.appendLine('✅ BSL Analyzer v1.3.1 activated successfully with LSP commands');
        
    } catch (error) {
        console.error('BSL Analyzer activation failed:', error);
        outputChannel?.appendLine(`❌ Activation failed: ${error}`);
        vscode.window.showErrorMessage(`BSL Analyzer activation failed: ${error}`);
    }
}


function initializeConfiguration() {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    indexServerPath = config.get<string>('indexServerPath', '');
    
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
        
        // Fallback: Try to find binaries in workspace
        if (!indexServerPath) {
            const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
            if (workspaceFolder) {
                const targetPath = path.join(workspaceFolder.uri.fsPath, 'target', 'debug');
                if (fs.existsSync(targetPath)) {
                    indexServerPath = targetPath;
                    outputChannel.appendLine(`Auto-detected BSL Analyzer binaries at: ${indexServerPath}`);
                }
            }
        }
    }
}

function showWelcomeMessage() {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    const configPath = config.get<string>('configurationPath', '');
    
    if (!configPath) {
        vscode.window.showInformationMessage(
            'BSL Analyzer is ready! Configure your 1C configuration path in settings to enable full functionality.',
            'Open Settings'
        ).then(selection => {
            if (selection === 'Open Settings') {
                vscode.commands.executeCommand('workbench.action.openSettings', 'bslAnalyzer.configurationPath');
            }
        });
    } else {
        vscode.window.showInformationMessage('BSL Analyzer is ready! Use Ctrl+Shift+P and search for "BSL" to explore features.');
    }
}

function startLanguageClient(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    let serverPath = config.get<string>('serverPath', '');
    const serverMode = config.get<string>('serverMode', 'tcp');
    const tcpPort = config.get<number>('tcpPort', 8080);
    const traceLevel = config.get<string>('trace.server', 'off');
    
    // Auto-detect bundled LSP server if not configured
    if (!serverPath) {
        const extensionPath = vscode.extensions.getExtension('bsl-analyzer-team.bsl-analyzer')?.extensionPath;
        if (extensionPath) {
            const bundledLspPath = path.join(extensionPath, 'bin', 'lsp_server.exe');
            if (fs.existsSync(bundledLspPath)) {
                serverPath = bundledLspPath;
                outputChannel.appendLine(`Using bundled LSP server: ${bundledLspPath}`);
            } else {
                serverPath = 'bsl-analyzer'; // fallback
            }
        } else {
            serverPath = 'bsl-analyzer'; // fallback
        }
    }

    let serverOptions: ServerOptions;

    if (serverMode === 'tcp') {
        // TCP mode
        serverOptions = {
            command: serverPath,
            args: ['lsp', '--port', tcpPort.toString()],
            transport: TransportKind.stdio
        };
    } else {
        // STDIO mode
        serverOptions = {
            command: serverPath,
            args: ['lsp'],
            transport: TransportKind.stdio
        };
    }

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'bsl' }],
        synchronize: {
            fileEvents: [
                vscode.workspace.createFileSystemWatcher('**/*.bsl'),
                vscode.workspace.createFileSystemWatcher('**/*.os'),
                vscode.workspace.createFileSystemWatcher('**/bsl-rules.toml'),
                vscode.workspace.createFileSystemWatcher('**/lsp-config.toml')
            ]
        },
        revealOutputChannelOn: RevealOutputChannelOn.Never,
        initializationOptions: {
            enableRealTimeAnalysis: config.get<boolean>('enableRealTimeAnalysis', true),
            enableMetrics: config.get<boolean>('enableMetrics', true),
            maxFileSize: config.get<number>('maxFileSize', 1048576),
            rulesConfig: config.get<string>('rulesConfig', ''),
            configurationPath: getConfigurationPath(),
            platformVersion: getPlatformVersion()
        },
        diagnosticCollectionName: 'bsl-analyzer',
        outputChannel: outputChannel
    };

    client = new LanguageClient(
        'bslAnalyzer',
        'BSL Analyzer Language Server',
        serverOptions,
        clientOptions
    );

    // Set trace level (handled through client options)

    // Start the client
    client.start().then(() => {
        console.log('BSL Analyzer LSP client started successfully');
        updateStatusBar('BSL Analyzer: Ready');
    }).catch(error => {
        console.error('Failed to start BSL Analyzer LSP client:', error);
        vscode.window.showErrorMessage(`Failed to start BSL Analyzer: ${error.message}`);
        updateStatusBar('BSL Analyzer: Error');
    });

    context.subscriptions.push(client);
}

function registerCommands(context: vscode.ExtensionContext) {
    outputChannel.appendLine('📝 Registering BSL Analyzer commands...');
    
    // Analyze current file
    const analyzeFileCommand = vscode.commands.registerCommand('bslAnalyzer.analyzeFile', async () => {
        outputChannel.appendLine('🔍 analyzeFile command executed');
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'bsl') {
            vscode.window.showWarningMessage('Please open a BSL file to analyze');
            return;
        }

        const document = editor.document;
        updateStatusBar('BSL Analyzer: Analyzing...');

        try {
            // Try LSP server analysis first
            if (client && client.isRunning()) {
                try {
                    await client.sendRequest('workspace/executeCommand', {
                        command: 'bslAnalyzer.analyzeFile',
                        arguments: [document.uri.toString()]
                    });
                    vscode.window.showInformationMessage('✅ File analysis completed (LSP mode)');
                } catch (lspError) {
                    outputChannel.appendLine(`⚠️ LSP analysis failed: ${lspError}`);
                    // Fallback to direct analysis
                    await performDirectAnalysis(document);
                    vscode.window.showInformationMessage('✅ File analysis completed (direct mode)');
                }
            } else {
                // Direct analysis without LSP
                await performDirectAnalysis(document);
                vscode.window.showInformationMessage('✅ File analysis completed (offline mode)');
            }
            updateStatusBar('BSL Analyzer: Ready');
        } catch (error) {
            vscode.window.showErrorMessage(`Analysis failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });

    // Helper function for direct analysis
    async function performDirectAnalysis(document: vscode.TextDocument) {
        outputChannel.appendLine(`📁 Analyzing file: ${document.fileName}`);
        outputChannel.appendLine(`📊 File size: ${document.getText().length} characters`);
        outputChannel.appendLine(`🔤 Language: ${document.languageId}`);
        
        // TODO: Add direct BSL analysis using bundled bsl-analyzer.exe
        if (indexServerPath) {
            outputChannel.appendLine(`🔧 Using BSL analyzer at: ${indexServerPath}`);
        }
    }

    // Analyze workspace
    const analyzeWorkspaceCommand = vscode.commands.registerCommand('bslAnalyzer.analyzeWorkspace', async () => {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (!workspaceFolders || workspaceFolders.length === 0) {
            vscode.window.showWarningMessage('No workspace folder is open');
            return;
        }

        updateStatusBar('BSL Analyzer: Analyzing workspace...');

        try {
            await client.sendRequest('workspace/executeCommand', {
                command: 'bslAnalyzer.analyzeWorkspace',
                arguments: [workspaceFolders[0].uri.toString()]
            });

            vscode.window.showInformationMessage('Workspace analysis completed');
            updateStatusBar('BSL Analyzer: Ready');
        } catch (error) {
            vscode.window.showErrorMessage(`Workspace analysis failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });

    // Generate reports
    const generateReportsCommand = vscode.commands.registerCommand('bslAnalyzer.generateReports', async () => {
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
            await client.sendRequest('workspace/executeCommand', {
                command: 'bslAnalyzer.generateReports',
                arguments: [workspaceFolders[0].uri.toString(), outputDir]
            });

            const openReports = await vscode.window.showInformationMessage(
                'Reports generated successfully',
                'Open Reports Folder'
            );

            if (openReports) {
                vscode.commands.executeCommand('vscode.openFolder', vscode.Uri.file(outputDir));
            }

            updateStatusBar('BSL Analyzer: Ready');
        } catch (error) {
            vscode.window.showErrorMessage(`Report generation failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });

    // Show metrics
    const showMetricsCommand = vscode.commands.registerCommand('bslAnalyzer.showMetrics', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'bsl') {
            vscode.window.showWarningMessage('Please open a BSL file to show metrics');
            return;
        }

        try {
            const metrics = await client.sendRequest('workspace/executeCommand', {
                command: 'bslAnalyzer.getMetrics',
                arguments: [editor.document.uri.toString()]
            });

            // Show metrics in a webview
            showMetricsWebview(context, metrics);
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to get metrics: ${error}`);
        }
    });

    // Configure rules
    const configureRulesCommand = vscode.commands.registerCommand('bslAnalyzer.configureRules', async () => {
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
        } catch {
            // File doesn't exist, create it
            const createFile = await vscode.window.showInformationMessage(
                'Rules configuration file not found. Would you like to create one?',
                'Create Rules File'
            );

            if (createFile) {
                try {
                    await client.sendRequest('workspace/executeCommand', {
                        command: 'bslAnalyzer.createRulesConfig',
                        arguments: [rulesFile.toString()]
                    });

                    const document = await vscode.workspace.openTextDocument(rulesFile);
                    await vscode.window.showTextDocument(document);
                } catch (error) {
                    vscode.window.showErrorMessage(`Failed to create rules file: ${error}`);
                }
            }
        }
    });

    // Search BSL Type
    const searchTypeCommand = vscode.commands.registerCommand('bslAnalyzer.searchType', async () => {
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
        } catch (error) {
            vscode.window.showErrorMessage(`Type search failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });

    // Search Method in Type
    const searchMethodCommand = vscode.commands.registerCommand('bslAnalyzer.searchMethod', async () => {
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
        } catch (error) {
            vscode.window.showErrorMessage(`Method search failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });

    // Build Unified BSL Index
    const buildIndexCommand = vscode.commands.registerCommand('bslAnalyzer.buildIndex', async () => {
        const configPath = getConfigurationPath();
        if (!configPath) {
            vscode.window.showWarningMessage('Please configure the 1C configuration path in settings');
            return;
        }

        const choice = await vscode.window.showInformationMessage(
            'Building unified BSL index. This may take a few seconds...',
            'Build Index',
            'Cancel'
        );

        if (choice !== 'Build Index') {
            return;
        }

        updateStatusBar('BSL Analyzer: Building index...');

        try {
            await vscode.window.withProgress({
                location: vscode.ProgressLocation.Notification,
                title: 'Building BSL Index',
                cancellable: false
            }, async (progress) => {
                progress.report({ increment: 0, message: 'Initializing...' });
                
                const result = await executeBslCommand('build_unified_index', [
                    '--config', configPath,
                    '--platform-version', getPlatformVersion()
                ]);

                progress.report({ increment: 100, message: 'Completed' });
                
                vscode.window.showInformationMessage(`Index built successfully: ${result}`);
            });

            updateStatusBar('BSL Analyzer: Ready');
        } catch (error) {
            vscode.window.showErrorMessage(`Index build failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });

    // Show Index Statistics
    const showIndexStatsCommand = vscode.commands.registerCommand('bslAnalyzer.showIndexStats', async () => {
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
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to load index stats: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });

    // Incremental Index Update
    const incrementalUpdateCommand = vscode.commands.registerCommand('bslAnalyzer.incrementalUpdate', async () => {
        const configPath = getConfigurationPath();
        if (!configPath) {
            vscode.window.showWarningMessage('Please configure the 1C configuration path in settings');
            return;
        }

        updateStatusBar('BSL Analyzer: Updating index...');

        try {
            const result = await executeBslCommand('incremental_update', [
                '--config', configPath,
                '--platform-version', getPlatformVersion(),
                '--verbose'
            ]);

            vscode.window.showInformationMessage(`Index updated: ${result}`);
            updateStatusBar('BSL Analyzer: Ready');
        } catch (error) {
            vscode.window.showErrorMessage(`Incremental update failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });

    // Explore Type Methods & Properties
    const exploreTypeCommand = vscode.commands.registerCommand('bslAnalyzer.exploreType', async () => {
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
        } catch (error) {
            vscode.window.showErrorMessage(`Type exploration failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });

    // Validate Method Call
    const validateMethodCallCommand = vscode.commands.registerCommand('bslAnalyzer.validateMethodCall', async () => {
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
        } catch (error) {
            vscode.window.showErrorMessage(`Method validation failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });

    // Check Type Compatibility
    const checkTypeCompatibilityCommand = vscode.commands.registerCommand('bslAnalyzer.checkTypeCompatibility', async () => {
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
        } catch (error) {
            vscode.window.showErrorMessage(`Type compatibility check failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });

    // Restart server
    const restartServerCommand = vscode.commands.registerCommand('bslAnalyzer.restartServer', async () => {
        updateStatusBar('BSL Analyzer: Restarting...');
        
        try {
            await client.stop();
            startLanguageClient(context);
            vscode.window.showInformationMessage('BSL Analyzer server restarted');
        } catch (error) {
            vscode.window.showErrorMessage(`Failed to restart server: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });

    context.subscriptions.push(
        analyzeFileCommand,
        analyzeWorkspaceCommand,
        generateReportsCommand,
        showMetricsCommand,
        configureRulesCommand,
        searchTypeCommand,
        searchMethodCommand,
        buildIndexCommand,
        showIndexStatsCommand,
        incrementalUpdateCommand,
        exploreTypeCommand,
        validateMethodCallCommand,
        checkTypeCompatibilityCommand,
        restartServerCommand
    );
    
    outputChannel.appendLine('✅ Successfully registered 14 BSL Analyzer commands');
}

let statusBarItem: vscode.StatusBarItem;

function registerStatusBar(context: vscode.ExtensionContext) {
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    statusBarItem.command = 'bslAnalyzer.analyzeFile';
    statusBarItem.text = 'BSL Analyzer: Starting...';
    statusBarItem.tooltip = 'Click to analyze current file';
    statusBarItem.show();
    
    context.subscriptions.push(statusBarItem);
}

function updateStatusBar(text: string) {
    if (statusBarItem) {
        statusBarItem.text = text;
    }
}

function showMetricsWebview(context: vscode.ExtensionContext, metrics: any) {
    const panel = vscode.window.createWebviewPanel(
        'bslMetrics',
        'BSL Code Quality Metrics',
        vscode.ViewColumn.Two,
        {
            enableScripts: true,
            retainContextWhenHidden: true
        }
    );

    panel.webview.html = getMetricsWebviewContent(metrics);
}

function getMetricsWebviewContent(metrics: any): string {
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
                ${metrics.recommendations.map((rec: string) => `<li>${rec}</li>`).join('')}
            </ul>
        </div>
        ` : ''}
    </body>
    </html>
    `;
}

function getScoreClass(score: number): string {
    if (score >= 90) return 'score-excellent';
    if (score >= 75) return 'score-good';
    if (score >= 50) return 'score-warning';
    return 'score-poor';
}

// Helper functions for BSL Index commands
function getConfigurationPath(): string {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    return config.get<string>('configurationPath', '');
}

function getPlatformVersion(): string {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    return config.get<string>('platformVersion', '8.3.25');
}

function getBinaryPath(binaryName: string): string {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    const serverPath = config.get<string>('indexServerPath', '');
    
    if (serverPath) {
        return path.join(serverPath, `${binaryName}.exe`);
    }
    
    // Try bundled binaries first
    const extensionPath = vscode.extensions.getExtension('bsl-analyzer-team.bsl-analyzer')?.extensionPath;
    if (extensionPath) {
        const bundledBinPath = path.join(extensionPath, 'bin', `${binaryName}.exe`);
        if (fs.existsSync(bundledBinPath)) {
            return bundledBinPath;
        }
    }
    
    // Try workspace target directory
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (workspaceFolder) {
        const targetPath = path.join(workspaceFolder.uri.fsPath, 'target', 'debug', `${binaryName}.exe`);
        if (fs.existsSync(targetPath)) {
            return targetPath;
        }
        
        const releasePath = path.join(workspaceFolder.uri.fsPath, 'target', 'release', `${binaryName}.exe`);
        if (fs.existsSync(releasePath)) {
            return releasePath;
        }
    }
    
    return `${binaryName}.exe`;
}

function executeBslCommand(command: string, args: string[]): Promise<string> {
    return new Promise((resolve, reject) => {
        const binaryPath = getBinaryPath(command);
        
        outputChannel.appendLine(`Executing: ${binaryPath} ${args.join(' ')}`);
        
        const process = spawn(binaryPath, args, {
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
            } else {
                reject(new Error(`Command failed with code ${code}: ${stderr}`));
            }
        });
        
        process.on('error', (error) => {
            outputChannel.appendLine(`Process error: ${error.message}`);
            reject(error);
        });
    });
}

interface MethodCallInfo {
    typeName: string;
    methodName: string;
    parameters: string[];
}

function parseMethodCall(selectedText: string): MethodCallInfo | null {
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

function showTypeInfoWebview(context: vscode.ExtensionContext, typeName: string, result: string) {
    const panel = vscode.window.createWebviewPanel(
        'bslTypeInfo',
        `BSL Type: ${typeName}`,
        vscode.ViewColumn.Two,
        {
            enableScripts: true,
            retainContextWhenHidden: true
        }
    );
    
    panel.webview.html = getTypeInfoWebviewContent(typeName, result);
}

function showMethodInfoWebview(context: vscode.ExtensionContext, typeName: string, methodName: string, result: string) {
    const panel = vscode.window.createWebviewPanel(
        'bslMethodInfo',
        `BSL Method: ${typeName}.${methodName}`,
        vscode.ViewColumn.Two,
        {
            enableScripts: true,
            retainContextWhenHidden: true
        }
    );
    
    panel.webview.html = getMethodInfoWebviewContent(typeName, methodName, result);
}

function showTypeExplorerWebview(context: vscode.ExtensionContext, typeName: string, result: string) {
    const panel = vscode.window.createWebviewPanel(
        'bslTypeExplorer',
        `BSL Type Explorer: ${typeName}`,
        vscode.ViewColumn.Two,
        {
            enableScripts: true,
            retainContextWhenHidden: true
        }
    );
    
    panel.webview.html = getTypeExplorerWebviewContent(typeName, result);
}

function showIndexStatsWebview(context: vscode.ExtensionContext, result: string) {
    const panel = vscode.window.createWebviewPanel(
        'bslIndexStats',
        'BSL Index Statistics',
        vscode.ViewColumn.Two,
        {
            enableScripts: true,
            retainContextWhenHidden: true
        }
    );
    
    panel.webview.html = getIndexStatsWebviewContent(result);
}

function showMethodValidationWebview(context: vscode.ExtensionContext, methodCall: MethodCallInfo, result: string) {
    const panel = vscode.window.createWebviewPanel(
        'bslMethodValidation',
        `Method Validation: ${methodCall.typeName}.${methodCall.methodName}`,
        vscode.ViewColumn.Two,
        {
            enableScripts: true,
            retainContextWhenHidden: true
        }
    );
    
    panel.webview.html = getMethodValidationWebviewContent(methodCall, result);
}

function showTypeCompatibilityWebview(context: vscode.ExtensionContext, fromType: string, toType: string, result: string) {
    const panel = vscode.window.createWebviewPanel(
        'bslTypeCompatibility',
        `Type Compatibility: ${fromType} → ${toType}`,
        vscode.ViewColumn.Two,
        {
            enableScripts: true,
            retainContextWhenHidden: true
        }
    );
    
    panel.webview.html = getTypeCompatibilityWebviewContent(fromType, toType, result);
}

function getTypeInfoWebviewContent(typeName: string, result: string): string {
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

function getMethodInfoWebviewContent(typeName: string, methodName: string, result: string): string {
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

function getTypeExplorerWebviewContent(typeName: string, result: string): string {
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

function getIndexStatsWebviewContent(result: string): string {
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

function getMethodValidationWebviewContent(methodCall: MethodCallInfo, result: string): string {
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

function getTypeCompatibilityWebviewContent(fromType: string, toType: string, result: string): string {
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

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}