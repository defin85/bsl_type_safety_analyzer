import * as vscode from 'vscode';
import { BslTypeItem } from './items';

/**
 * Provider для отображения индекса типов BSL
 */
export class BslTypeIndexProvider implements vscode.TreeDataProvider<BslTypeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<BslTypeItem | undefined | null | void> = new vscode.EventEmitter<BslTypeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<BslTypeItem | undefined | null | void> = this._onDidChangeTreeData.event;

    refresh(): void {
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: BslTypeItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: BslTypeItem): Thenable<BslTypeItem[]> {
        if (!element) {
            return Promise.resolve([
                new BslTypeItem('Platform Types (8)', vscode.TreeItemCollapsibleState.Collapsed, 'Platform Types', 'platform'),
                new BslTypeItem('Configuration Types (5)', vscode.TreeItemCollapsibleState.Collapsed, 'Configuration Types', 'configuration'),
                new BslTypeItem('Global Functions', vscode.TreeItemCollapsibleState.Collapsed, 'Global Functions', 'module')
            ]);
        } else {
            switch (element.contextValue) {
                case 'platform':
                    return Promise.resolve([
                        new BslTypeItem('String', vscode.TreeItemCollapsibleState.None, 'String', 'platform', 'type'),
                        new BslTypeItem('Number', vscode.TreeItemCollapsibleState.None, 'Number', 'platform', 'type'),
                        new BslTypeItem('Boolean', vscode.TreeItemCollapsibleState.None, 'Boolean', 'platform', 'type'),
                        new BslTypeItem('Date', vscode.TreeItemCollapsibleState.None, 'Date', 'platform', 'type'),
                        new BslTypeItem('Array', vscode.TreeItemCollapsibleState.None, 'Array', 'platform', 'type')
                    ]);
                case 'configuration':
                    return Promise.resolve([
                        new BslTypeItem('Catalogs.Контрагенты', vscode.TreeItemCollapsibleState.None, 'Контрагенты', 'configuration', 'catalog'),
                        new BslTypeItem('Catalogs.Номенклатура', vscode.TreeItemCollapsibleState.None, 'Номенклатура', 'configuration', 'catalog'),
                        new BslTypeItem('Documents.ЗаказНаряды', vscode.TreeItemCollapsibleState.None, 'ЗаказНаряды', 'configuration', 'document')
                    ]);
                case 'module':
                    return Promise.resolve([
                        new BslTypeItem('Сообщить', vscode.TreeItemCollapsibleState.None, 'Сообщить', 'module', 'function'),
                        new BslTypeItem('СокрЛП', vscode.TreeItemCollapsibleState.None, 'СокрЛП', 'module', 'function'),
                        new BslTypeItem('НачалоГода', vscode.TreeItemCollapsibleState.None, 'НачалоГода', 'module', 'function')
                    ]);
                default:
                    return Promise.resolve([]);
            }
        }
    }
}