# 📦 Публикация и распространение

## 📋 Содержание раздела

- [PUBLISH_QUICK.md](PUBLISH_QUICK.md) - **Быстрая публикация за 5 минут**
  - Настройка за один раз
  - Команды для публикации
  - Готовые примеры

- [PUBLISHING_GUIDE.md](PUBLISHING_GUIDE.md) - **Подробное руководство**
  - Visual Studio Code Marketplace
  - GitHub Releases  
  - Внутреннее распространение
  - CI/CD автоматизация

- [NAMING_ALTERNATIVES.md](NAMING_ALTERNATIVES.md) - **Альтернативные названия**
  - Резервные варианты названий
  - Стратегии нейминга
  - Проверка доступности

## 🎯 Способы публикации

### 🏪 VS Code Marketplace (Рекомендуемый)
- **Аудитория**: Максимальная
- **Сложность**: Средняя (нужен Azure DevOps аккаунт)
- **Команда**: `npm run publish:marketplace`

### 📁 GitHub Releases (Простой старт)
- **Аудитория**: Разработчики и энтузиасты
- **Сложность**: Низкая
- **Команда**: `npm run publish:github`

### 🔧 Enterprise распространение
- **Аудитория**: Корпоративные пользователи
- **Сложность**: Низкая  
- **Способ**: Прямое распространение .vsix файлов

## ⚡ Быстрый старт

```bash
# 1. GitHub Releases (для начала)
npm run publish:github

# 2. VS Code Marketplace (после настройки)
npm run publish:marketplace  

# 3. Проверка пакета перед публикацией
npm run publish:check
```

## 📊 Текущий статус

- **Название расширения**: `bsl-type-safety-analyzer` ✅
- **Версия**: 1.4.2  
- **Размер пакета**: ~50 MB (включая все бинарники)  
- **Готовность**: Готово к публикации

## 🔗 Полезные ссылки

- [VS Code Extension Guidelines](https://code.visualstudio.com/api/references/extension-guidelines)
- [Publishing Extensions](https://code.visualstudio.com/api/working-with-extensions/publishing-extension)
- [GitHub Releases Documentation](https://docs.github.com/en/repositories/releasing-projects-on-github)