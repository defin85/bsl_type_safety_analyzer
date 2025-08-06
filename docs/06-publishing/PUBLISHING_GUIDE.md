# 📦 BSL Analyzer - Руководство по публикации расширений

## 🎯 Способы публикации VSCode расширений

### 1. 🏪 Visual Studio Code Marketplace (Официальный магазин)

#### Подготовка к публикации
```bash
# 1. Создать релизную версию
npm run git:release minor

# 2. Проверить готовность расширения
ls vscode-extension/dist/
# Должно быть: bsl-analyzer-X.X.X.vsix
```

#### Создание Publisher аккаунта
1. **Зарегистрироваться в Azure DevOps:**
   - Перейти на https://dev.azure.com
   - Создать аккаунт Microsoft (если нет)
   - Создать новую организацию

2. **Создать Personal Access Token:**
   ```
   Azure DevOps → User Settings → Personal Access Tokens
   - Name: VSCode Extension Publishing
   - Expiration: 1 year (или Custom)
   - Scopes: Custom defined
   - Selected scopes: Marketplace (Manage)
   ```

3. **Установить vsce CLI:**
   ```bash
   npm install -g @vscode/vsce
   ```

4. **Создать Publisher:**
   ```bash
   vsce create-publisher bsl-analyzer-team
   # Ввести: Personal Access Token
   ```

#### Публикация в Marketplace
```bash
# Перейти в директорию расширения
cd vscode-extension

# Опубликовать расширение
vsce publish

# Или опубликовать конкретный .vsix файл
vsce publish dist/bsl-analyzer-1.4.1.vsix
```

#### Обновление версии в Marketplace
```bash
# Автоматическое увеличение версии и публикация
vsce publish patch   # 1.4.1 → 1.4.2
vsce publish minor   # 1.4.1 → 1.5.0  
vsce publish major   # 1.4.1 → 2.0.0

# Или использовать нашу систему
npm run git:release patch
cd vscode-extension && vsce publish
```

---

### 2. 📁 GitHub Releases (Рекомендуемый способ)

#### Подготовка релиза
```bash
# 1. Создать релиз с нашей системой
npm run git:release minor

# 2. Запушить изменения и теги
git push origin main --follow-tags
```

#### Создание GitHub Release
1. **Перейти на GitHub:**
   - Открыть репозиторий: https://github.com/your-org/bsl-analyzer
   - Перейти в "Releases" → "Create a new release"

2. **Заполнить информацию о релизе:**
   ```
   Tag version: v1.4.1 (выбрать созданный тег)
   Release title: BSL Analyzer v1.4.1
   
   Description:
   ## 🚀 BSL Analyzer v1.4.1
   
   ### ✨ Новые возможности
   - Единая система сборки и версионирования
   - Автоматическое управление версиями
   - Git workflow интеграция
   
   ### 🔧 Улучшения
   - Оптимизированная сборка расширения
   - Синхронизация версий между компонентами
   
   ### 📦 Установка
   1. Скачать `bsl-analyzer-1.4.1.vsix`
   2. VS Code → Ctrl+Shift+P → "Extensions: Install from VSIX"
   3. Выбрать скачанный файл
   
   ### 🎯 Системные требования
   - VS Code 1.75.0+
   - Windows 10+ (включены бинарные файлы)
   ```

3. **Прикрепить файлы:**
   - Добавить `vscode-extension/dist/bsl-analyzer-1.4.1.vsix`
   - Добавить `README.md` (опционально)

4. **Опубликовать:**
   - Нажать "Publish release"

---

### 3. 🔧 Внутреннее распространение (Enterprise)

#### Корпоративные пользователи
```bash
# 1. Создать архив для распространения
npm run git:release patch
zip -r bsl-analyzer-enterprise-v1.4.1.zip vscode-extension/dist/ docs/ examples/

# 2. Инструкции для пользователей
```

**Инструкция для конечных пользователей Enterprise:**
1. Распаковать архив
2. VS Code → Ctrl+Shift+P 
3. "Extensions: Install from VSIX"
4. Выбрать `bsl-analyzer-X.X.X.vsix`

---

### 4. 🤖 Автоматизация публикации (CI/CD)

#### GitHub Actions для автоматической публикации
```yaml
# .github/workflows/publish.yml
name: Publish Extension

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        
    - name: Install dependencies
      run: npm install
      
    - name: Build extension
      run: npm run build:release
      
    - name: Publish to Marketplace
      run: |
        cd vscode-extension
        npx @vscode/vsce publish
      env:
        VSCE_PAT: ${{ secrets.VSCE_PAT }}
        
    - name: Create GitHub Release
      uses: softprops/action-gh-release@v1
      with:
        files: vscode-extension/dist/*.vsix
        generate_release_notes: true
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

## 📋 Чек-лист перед публикацией

### ✅ Обязательные проверки
- [ ] Версии синхронизированы (`npm run version:sync`)
- [ ] Расширение собрано (`npm run build:release`)
- [ ] .vsix файл создан в `vscode-extension/dist/`
- [ ] Размер файла разумный (< 50MB)
- [ ] Лицензия указана в package.json
- [ ] README.md содержит инструкции по установке

### ✅ Проверка расширения
```bash
# Установить локально для тестирования
code --install-extension vscode-extension/dist/bsl-analyzer-1.4.1.vsix

# Проверить активацию
# Открыть .bsl файл и проверить работу расширения
```

### ✅ Метаданные расширения
```json
// vscode-extension/package.json
{
  "publisher": "bsl-analyzer-team",           // ✅ Указан publisher
  "license": "MIT",                           // ✅ Указана лицензия  
  "repository": "https://github.com/...",     // ✅ Указан репозиторий
  "keywords": ["bsl", "1c", "analyzer"],     // ✅ Ключевые слова
  "categories": ["Programming Languages"],    // ✅ Категории
  "icon": "images/bsl-analyzer-logo.png"     // ✅ Иконка
}
```

---

## 💡 Рекомендации по публикации

### 🎯 Стратегия распространения
1. **Начать с GitHub Releases** - проще, нет модерации
2. **Набрать пользователей и отзывы** 
3. **Перейти на VS Code Marketplace** - для широкой аудитории
4. **Поддерживать оба канала** - максимальное покрытие

### 📊 Версионирование для публикации
```bash
# Стабильные релизы
npm run git:release major    # Большие обновления (2.0.0)
npm run git:release minor    # Новые функции (1.5.0)
npm run git:release patch    # Исправления (1.4.2)

# Beta версии (добавить в vscode-extension/package.json)
"version": "1.5.0-beta.1"
```

### 🔒 Безопасность
- ❌ **НЕ** включать секретные ключи в .vsix
- ❌ **НЕ** включать приватные зависимости
- ✅ Проверить все включенные файлы: `vsce ls`
- ✅ Указать только необходимые файлы в package.json "files"

### 📈 Мониторинг
После публикации отслеживать:
- Количество установок
- Рейтинг и отзывы пользователей  
- Issues в GitHub репозитории
- Статистика использования

---

## 🚀 Команды для быстрой публикации

```bash
# Полный цикл публикации одной командой
npm run git:release minor && \
cd vscode-extension && \
vsce publish && \
cd .. && \
echo "✅ Расширение опубликовано!"

# Или создать npm script
npm run publish:marketplace
```

**Добавить в package.json:**
```json
{
  "scripts": {
    "publish:marketplace": "npm run build:release && cd vscode-extension && vsce publish",
    "publish:github": "npm run git:release minor && git push origin main --follow-tags"
  }
}
```

Теперь можно публиковать расширения максимально просто! 🎉