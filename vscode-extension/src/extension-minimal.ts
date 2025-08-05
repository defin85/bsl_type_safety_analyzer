import * as vscode from 'vscode';

export function activate(context: vscode.ExtensionContext) {
    console.log('Ultra-minimal BSL extension activating...');
    
    const disposable = vscode.commands.registerCommand('bslAnalyzer.hello', () => {
        vscode.window.showInformationMessage('Hello from BSL Analyzer!');
    });

    context.subscriptions.push(disposable);
    
    console.log('Ultra-minimal BSL extension activated!');
}

export function deactivate() {}