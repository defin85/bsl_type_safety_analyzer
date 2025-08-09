import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { BslAnalyzerConfig } from '../config/configHelper';

/**
 * Элемент иерархического дерева типов
 */
export class HierarchicalTypeItem extends vscode.TreeItem {
    constructor(
        public readonly label: string,
        public readonly collapsibleState: vscode.TreeItemCollapsibleState,
        public readonly typeName: string,
        public readonly typeContext: string,
        public readonly itemData?: string
    ) {
        super(label, collapsibleState);
        this.contextValue = typeContext;
        this.tooltip = typeName;
    }
}

interface BslEntity {
    id: string;
    qualified_name: string;
    display_name: string;
    entity_type: 'Platform' | 'Configuration';
    entity_kind: string;
    interface?: {
        methods?: Record<string, any>;
        properties?: Record<string, any>;
        events?: Record<string, any>;
    };
    documentation?: string;
}

interface TypeCategory {
    name: string;
    icon: string;
    types: BslEntity[];
}

/**
 * Иерархический провайдер для отображения типов BSL с группировкой по категориям
 */
export class HierarchicalTypeIndexProvider implements vscode.TreeDataProvider<HierarchicalTypeItem> {
    private _onDidChangeTreeData: vscode.EventEmitter<HierarchicalTypeItem | undefined | null | void> = new vscode.EventEmitter<HierarchicalTypeItem | undefined | null | void>();
    readonly onDidChangeTreeData: vscode.Event<HierarchicalTypeItem | undefined | null | void> = this._onDidChangeTreeData.event;
    
    private outputChannel: vscode.OutputChannel | undefined;
    private platformTypes: Map<string, BslEntity> = new Map();
    private configTypes: Map<string, BslEntity> = new Map();
    private typeCategories: Map<string, TypeCategory> = new Map();

    constructor(outputChannel?: vscode.OutputChannel) {
        this.outputChannel = outputChannel;
        this.loadTypes();
    }

    refresh(): void {
        this.loadTypes();
        this._onDidChangeTreeData.fire();
    }

    getTreeItem(element: HierarchicalTypeItem): vscode.TreeItem {
        return element;
    }

    getChildren(element?: HierarchicalTypeItem): Thenable<HierarchicalTypeItem[]> {
        if (!element) {
            return this.getRootCategories();
        } else if (element.contextValue === 'category') {
            return this.getCategoryTypes(element);
        } else if (element.contextValue === 'type') {
            return this.getTypeMembers(element);
        } else if (element.contextValue === 'methods-folder') {
            return this.getTypeMethods(element);
        } else if (element.contextValue === 'properties-folder') {
            return this.getTypeProperties(element);
        }
        return Promise.resolve([]);
    }

    private async loadTypes(): Promise<void> {
        this.platformTypes.clear();
        this.configTypes.clear();
        this.typeCategories.clear();

        // Загружаем типы платформы
        await this.loadPlatformTypes();
        
        // Загружаем типы конфигурации
        await this.loadConfigurationTypes();

        // Группируем типы по категориям
        this.categorizeTypes();
    }

    private async loadPlatformTypes(): Promise<void> {
        try {
            const homedir = require('os').homedir();
            const platformVersion = BslAnalyzerConfig.platformVersion;
            const platformCachePath = path.join(homedir, '.bsl_analyzer', 'platform_cache', `${platformVersion}.jsonl`);
            
            if (fs.existsSync(platformCachePath)) {
                const content = fs.readFileSync(platformCachePath, 'utf-8');
                const lines = content.trim().split('\n');
                
                for (const line of lines) {
                    try {
                        const entity: BslEntity = JSON.parse(line);
                        if (entity.qualified_name) {
                            this.platformTypes.set(entity.qualified_name, entity);
                        }
                    } catch (e) {
                        // Игнорируем ошибки парсинга
                    }
                }
                
                this.outputChannel?.appendLine(`Loaded ${this.platformTypes.size} platform types`);
            }
        } catch (error) {
            this.outputChannel?.appendLine(`Error loading platform types: ${error}`);
        }
    }

    private async loadConfigurationTypes(): Promise<void> {
        try {
            const configPath = BslAnalyzerConfig.configurationPath;
            if (!configPath) return;
            
            const homedir = require('os').homedir();
            const platformVersion = BslAnalyzerConfig.platformVersion;
            const projectHash = require('crypto').createHash('md5').update(configPath).digest('hex').slice(0, 8);
            const projectCachePath = path.join(
                homedir, 
                '.bsl_analyzer', 
                'project_indices',
                `${path.basename(configPath)}_${projectHash}`,
                platformVersion,
                'config_entities.jsonl'
            );
            
            if (fs.existsSync(projectCachePath)) {
                const content = fs.readFileSync(projectCachePath, 'utf-8');
                const lines = content.trim().split('\n');
                
                for (const line of lines) {
                    try {
                        const entity: BslEntity = JSON.parse(line);
                        if (entity.qualified_name) {
                            this.configTypes.set(entity.qualified_name, entity);
                        }
                    } catch (e) {
                        // Игнорируем ошибки парсинга
                    }
                }
                
                this.outputChannel?.appendLine(`Loaded ${this.configTypes.size} configuration types`);
            }
        } catch (error) {
            this.outputChannel?.appendLine(`Error loading configuration types: ${error}`);
        }
    }

    private categorizeTypes(): void {
        // Категории для платформенных типов
        const platformCategories = {
            'Примитивные типы': ['Число', 'Строка', 'Булево', 'Дата', 'Неопределено', 'Null', 'Тип'],
            'Коллекции': ['Массив', 'Структура', 'Соответствие', 'СписокЗначений', 'ТаблицаЗначений', 'ДеревоЗначений'],
            'Работа с данными': ['Запрос', 'ПостроительЗапроса', 'СхемаЗапроса', 'РезультатЗапроса', 'ВыборкаИзРезультатаЗапроса'],
            'Работа с XML': ['ЧтениеXML', 'ЗаписьXML', 'ФабрикаXDTO', 'СериализаторXDTO'],
            'Работа с JSON': ['ЧтениеJSON', 'ЗаписьJSON'],
            'Файловая система': ['Файл', 'ДиалогВыбораФайла', 'ЧтениеТекста', 'ЗаписьТекста'],
            'Интерфейс': ['Форма', 'ТабличныйДокумент', 'Диаграмма', 'ПолеHTMLДокумента'],
            'Менеджеры': ['Справочники', 'Документы', 'РегистрыСведений', 'РегистрыНакопления', 'ПланыВидовХарактеристик'],
            'Глобальные функции': ['Сообщить', 'СокрЛП', 'НачалоГода', 'СтрНайти', 'Формат', 'XMLСтрока']
        };

        // Создаем категории для платформенных типов
        for (const [categoryName, typePatterns] of Object.entries(platformCategories)) {
            const category: TypeCategory = {
                name: categoryName,
                icon: this.getCategoryIcon(categoryName),
                types: []
            };

            for (const [typeName, entity] of this.platformTypes) {
                if (this.matchesCategory(typeName, entity.display_name, typePatterns)) {
                    category.types.push(entity);
                }
            }

            if (category.types.length > 0) {
                this.typeCategories.set(`platform:${categoryName}`, category);
            }
        }

        // Категории для типов конфигурации
        if (this.configTypes.size > 0) {
            const configCategories: Map<string, TypeCategory> = new Map();
            
            for (const [typeName, entity] of this.configTypes) {
                const categoryName = this.getConfigCategory(entity);
                
                if (!configCategories.has(categoryName)) {
                    configCategories.set(categoryName, {
                        name: categoryName,
                        icon: this.getCategoryIcon(categoryName),
                        types: []
                    });
                }
                
                configCategories.get(categoryName)!.types.push(entity);
            }

            for (const [categoryName, category] of configCategories) {
                this.typeCategories.set(`config:${categoryName}`, category);
            }
        }

        // Добавляем категорию "Все остальные" для неклассифицированных типов платформы
        const uncategorized: BslEntity[] = [];
        for (const [typeName, entity] of this.platformTypes) {
            let found = false;
            for (const category of this.typeCategories.values()) {
                if (category.types.includes(entity)) {
                    found = true;
                    break;
                }
            }
            if (!found) {
                uncategorized.push(entity);
            }
        }

        if (uncategorized.length > 0) {
            this.typeCategories.set('platform:Другие', {
                name: 'Другие типы платформы',
                icon: '📦',
                types: uncategorized
            });
        }
    }

    private matchesCategory(typeName: string, displayName: string, patterns: string[]): boolean {
        for (const pattern of patterns) {
            if (typeName.includes(pattern) || displayName?.includes(pattern)) {
                return true;
            }
        }
        return false;
    }

    private getConfigCategory(entity: BslEntity): string {
        const kind = entity.entity_kind || 'Other';
        const categoryMap: Record<string, string> = {
            'Catalog': 'Справочники',
            'Document': 'Документы',
            'InformationRegister': 'Регистры сведений',
            'AccumulationRegister': 'Регистры накопления',
            'AccountingRegister': 'Регистры бухгалтерии',
            'CalculationRegister': 'Регистры расчета',
            'ChartOfCharacteristicTypes': 'Планы видов характеристик',
            'ChartOfAccounts': 'Планы счетов',
            'ChartOfCalculationTypes': 'Планы видов расчета',
            'BusinessProcess': 'Бизнес-процессы',
            'Task': 'Задачи',
            'ExchangePlan': 'Планы обмена',
            'CommonModule': 'Общие модули',
            'Report': 'Отчеты',
            'DataProcessor': 'Обработки'
        };
        
        return categoryMap[kind] || 'Другие объекты';
    }

    private getCategoryIcon(categoryName: string): string {
        const iconMap: Record<string, string> = {
            'Примитивные типы': '🔤',
            'Коллекции': '📚',
            'Работа с данными': '🗃️',
            'Работа с XML': '📄',
            'Работа с JSON': '📋',
            'Файловая система': '📁',
            'Интерфейс': '🖼️',
            'Менеджеры': '👥',
            'Глобальные функции': '🔧',
            'Справочники': '📖',
            'Документы': '📃',
            'Регистры сведений': '📊',
            'Регистры накопления': '📈',
            'Регистры бухгалтерии': '💰',
            'Регистры расчета': '🧮',
            'Общие модули': '📦',
            'Отчеты': '📊',
            'Обработки': '⚙️'
        };
        
        return iconMap[categoryName] || '📂';
    }

    private async getRootCategories(): Promise<HierarchicalTypeItem[]> {
        const items: HierarchicalTypeItem[] = [];

        // Группируем категории: платформенные и конфигурационные
        const platformCategories: HierarchicalTypeItem[] = [];
        const configCategories: HierarchicalTypeItem[] = [];

        for (const [key, category] of this.typeCategories) {
            const categoryItem = new HierarchicalTypeItem(
                `${category.icon} ${category.name} (${category.types.length})`,
                vscode.TreeItemCollapsibleState.Collapsed,
                category.name,
                'category',
                key
            );
            
            if (key.startsWith('platform:')) {
                platformCategories.push(categoryItem);
            } else {
                configCategories.push(categoryItem);
            }
        }

        // Добавляем группы верхнего уровня
        if (platformCategories.length > 0) {
            const platformGroup = new HierarchicalTypeItem(
                `🏢 Платформа 1С (${this.platformTypes.size} типов)`,
                vscode.TreeItemCollapsibleState.Collapsed,
                'Платформа',
                'platform-group'
            );
            items.push(platformGroup);
        }

        if (configCategories.length > 0) {
            const configGroup = new HierarchicalTypeItem(
                `🏗️ Конфигурация (${this.configTypes.size} типов)`,
                vscode.TreeItemCollapsibleState.Collapsed,
                'Конфигурация',
                'config-group'
            );
            items.push(configGroup);
        }

        // Для упрощения пока возвращаем категории напрямую
        return [...platformCategories, ...configCategories];
    }

    private async getCategoryTypes(element: HierarchicalTypeItem): Promise<HierarchicalTypeItem[]> {
        const categoryKey = element.itemData;
        if (!categoryKey) return [];

        const category = this.typeCategories.get(categoryKey);
        if (!category) return [];

        return category.types.slice(0, 100).map(entity => {
            const hasMembers = this.hasMembers(entity);
            return new HierarchicalTypeItem(
                entity.display_name || entity.qualified_name,
                hasMembers ? vscode.TreeItemCollapsibleState.Collapsed : vscode.TreeItemCollapsibleState.None,
                entity.qualified_name,
                'type',
                entity.qualified_name
            );
        });
    }

    private hasMembers(entity: BslEntity): boolean {
        const hasMethod = entity.interface?.methods && Object.keys(entity.interface.methods).length > 0;
        const hasProps = entity.interface?.properties && Object.keys(entity.interface.properties).length > 0;
        const hasEvents = entity.interface?.events && Object.keys(entity.interface.events).length > 0;
        return !!(hasMethod || hasProps || hasEvents);
    }

    private async getTypeMembers(element: HierarchicalTypeItem): Promise<HierarchicalTypeItem[]> {
        const typeName = element.itemData;
        if (!typeName) return [];

        const entity = this.platformTypes.get(typeName) || this.configTypes.get(typeName);
        if (!entity) return [];

        const items: HierarchicalTypeItem[] = [];

        // Добавляем папку с методами
        const methodCount = entity.interface?.methods ? Object.keys(entity.interface.methods).length : 0;
        if (methodCount > 0) {
            const methodsFolder = new HierarchicalTypeItem(
                `📦 Методы (${methodCount})`,
                vscode.TreeItemCollapsibleState.Collapsed,
                'Методы',
                'methods-folder',
                typeName
            );
            items.push(methodsFolder);
        }

        // Добавляем папку со свойствами
        const propCount = entity.interface?.properties ? Object.keys(entity.interface.properties).length : 0;
        if (propCount > 0) {
            const propsFolder = new HierarchicalTypeItem(
                `📋 Свойства (${propCount})`,
                vscode.TreeItemCollapsibleState.Collapsed,
                'Свойства',
                'properties-folder',
                typeName
            );
            items.push(propsFolder);
        }

        // Добавляем описание, если есть
        if (entity.documentation) {
            const docItem = new HierarchicalTypeItem(
                `📄 ${entity.documentation.substring(0, 100)}...`,
                vscode.TreeItemCollapsibleState.None,
                'Описание',
                'documentation'
            );
            docItem.tooltip = entity.documentation;
            items.push(docItem);
        }

        return items;
    }

    private async getTypeMethods(element: HierarchicalTypeItem): Promise<HierarchicalTypeItem[]> {
        const typeName = element.itemData;
        if (!typeName) return [];

        const entity = this.platformTypes.get(typeName) || this.configTypes.get(typeName);
        if (!entity || !entity.interface?.methods) return [];

        return Object.entries(entity.interface.methods).slice(0, 50).map(([name, method]) => {
            const item = new HierarchicalTypeItem(
                `⚡ ${name}`,
                vscode.TreeItemCollapsibleState.None,
                name,
                'method'
            );
            item.tooltip = this.formatMethodTooltip(name, method);
            return item;
        });
    }

    private async getTypeProperties(element: HierarchicalTypeItem): Promise<HierarchicalTypeItem[]> {
        const typeName = element.itemData;
        if (!typeName) return [];

        const entity = this.platformTypes.get(typeName) || this.configTypes.get(typeName);
        if (!entity || !entity.interface?.properties) return [];

        return Object.entries(entity.interface.properties).slice(0, 50).map(([name, prop]) => {
            const item = new HierarchicalTypeItem(
                `📌 ${name}`,
                vscode.TreeItemCollapsibleState.None,
                name,
                'property'
            );
            item.tooltip = this.formatPropertyTooltip(name, prop);
            return item;
        });
    }

    private formatMethodTooltip(name: string, method: any): string {
        let tooltip = `Метод: ${name}`;
        if (method.parameters) {
            tooltip += '\nПараметры: ' + JSON.stringify(method.parameters);
        }
        if (method.returns) {
            tooltip += '\nВозвращает: ' + method.returns;
        }
        if (method.documentation) {
            tooltip += '\n\n' + method.documentation;
        }
        return tooltip;
    }

    private formatPropertyTooltip(name: string, prop: any): string {
        let tooltip = `Свойство: ${name}`;
        if (prop.type) {
            tooltip += '\nТип: ' + prop.type;
        }
        if (prop.readonly) {
            tooltip += '\n(Только чтение)';
        }
        if (prop.documentation) {
            tooltip += '\n\n' + prop.documentation;
        }
        return tooltip;
    }
}