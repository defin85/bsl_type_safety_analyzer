# 🚀 Быстрая публикация BSL Analyzer

## ⚡ За 5 минут до публикации

### 1. 📦 Подготовка к публикации
```bash
# Проверить готовность расширения
npm run publish:check

# Создать релизную версию
npm run git:release minor
```

### 2. 🏪 VS Code Marketplace (Рекомендуемый способ)

#### Настройка (один раз)
1. **Azure DevOps аккаунт:** https://dev.azure.com
2. **Personal Access Token:** User Settings → Personal Access Tokens
   - Scopes: **Marketplace (Manage)**
3. **Установить vsce:** `npm install -g @vscode/vsce`
4. **Создать publisher:** `vsce create-publisher bsl-analyzer-team`

#### Публикация (каждый релиз)
```bash
# ⚠️ ВАЖНО: команды выполнять из КОРНЯ проекта
cd C:\1CProject\bsl_type_safety_analyzer\

# Опубликовать в Marketplace одной командой
npm run publish:marketplace

# Или вручную
cd vscode-extension
npx @vscode/vsce publish
cd ..  # Вернуться в корень
```

### 3. 📁 GitHub Releases (Альтернативный способ)

```bash
# ⚠️ ВАЖНО: команды выполнять из КОРНЯ проекта
cd C:\1CProject\bsl_type_safety_analyzer\

# Создать GitHub Release одной командой  
npm run publish:github

# Затем на GitHub:
# 1. Releases → Create new release
# 2. Выбрать созданный тег v1.X.X
# 3. Прикрепить .vsix файл из vscode-extension/dist/
# 4. Publish release
```

## 📊 Статус расширения

**Текущая версия:** 1.4.1  
**Размер пакета:** ~20.5 MB (47 MB бинарников + интерфейс)  
**Файлов в пакете:** 47 файлов  
**Готовность к публикации:** ✅

## 💡 Рекомендации

1. **Начать с GitHub Releases** - проще, без модерации
2. **Собрать отзывы пользователей**  
3. **Перейти на VS Code Marketplace** - для широкой аудитории

## 🔍 Проверка перед публикацией

```bash
# Содержимое пакета
npm run publish:check

# Локальная установка для тестирования
code --install-extension vscode-extension/dist/bsl-analyzer-1.4.1.vsix
```

## 📞 Поддержка

- 📚 Подробности: `PUBLISHING_GUIDE.md`
- 🚀 Быстрый старт: `QUICK_START.md`
- 🔧 Сборка: `BUILD_SYSTEM.md`