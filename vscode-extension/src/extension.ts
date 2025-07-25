import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
    RevealOutputChannelOn
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    console.log('BSL Analyzer extension is being activated');

    // Start LSP client
    startLanguageClient(context);

    // Register commands
    registerCommands(context);

    // Register status bar
    registerStatusBar(context);
}

function startLanguageClient(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('bslAnalyzer');
    const serverPath = config.get<string>('serverPath', 'bsl-analyzer');
    const serverMode = config.get<string>('serverMode', 'tcp');
    const tcpPort = config.get<number>('tcpPort', 8080);
    const traceLevel = config.get<string>('trace.server', 'off');

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
            rulesConfig: config.get<string>('rulesConfig', '')
        }
    };

    client = new LanguageClient(
        'bslAnalyzer',
        'BSL Analyzer Language Server',
        serverOptions,
        clientOptions
    );

    // Set trace level
    client.trace = traceLevel as any;

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
    // Analyze current file
    const analyzeFileCommand = vscode.commands.registerCommand('bslAnalyzer.analyzeFile', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'bsl') {
            vscode.window.showWarningMessage('Please open a BSL file to analyze');
            return;
        }

        const document = editor.document;
        updateStatusBar('BSL Analyzer: Analyzing...');

        try {
            // Request analysis from LSP server
            await client.sendRequest('workspace/executeCommand', {
                command: 'bslAnalyzer.analyzeFile',
                arguments: [document.uri.toString()]
            });

            vscode.window.showInformationMessage('File analysis completed');
            updateStatusBar('BSL Analyzer: Ready');
        } catch (error) {
            vscode.window.showErrorMessage(`Analysis failed: ${error}`);
            updateStatusBar('BSL Analyzer: Error');
        }
    });

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
        restartServerCommand
    );
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

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}