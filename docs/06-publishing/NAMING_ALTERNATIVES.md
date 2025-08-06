# 📝 Альтернативные названия для BSL Analyzer

## 🎯 Текущее название
- **Имя:** `bsl-type-safety-analyzer`
- **Отображаемое название:** `BSL Analyzer`
- **Publisher:** `bsl-analyzer-team`

## 📋 Резервные варианты (если текущий занят)

### 🔢 Numbered Variants
- `bsl-type-safety-analyzer-v2`
- `bsl-analyzer-advanced`
- `bsl-analyzer-enterprise`
- `bsl-analyzer-plus`

### 🏷️ Team/Company Variants  
- `bsl-analyzer-by-team`
- `team-bsl-analyzer`
- `advanced-bsl-analyzer`
- `professional-bsl-analyzer`

### 🔧 Feature-focused Names
- `bsl-type-checker`
- `bsl-static-analyzer`
- `onec-bsl-analyzer`
- `enterprise-bsl-analyzer`
- `unified-bsl-analyzer`

### 🌟 Creative Names
- `bsl-sentinel` (страж BSL кода)
- `bsl-guardian` (защитник BSL)
- `onec-code-analyzer`
- `bsl-quality-checker`
- `smart-bsl-analyzer`

## 🔍 Проверка доступности

Перед публикацией можно проверить доступность:

```bash
# Поиск существующих расширений
# https://marketplace.visualstudio.com/search?term=bsl&target=VSCode

# Проверить конкретное название через API
curl "https://marketplace.visualstudio.com/items?itemName=publisher.extension-name"
```

## 📦 Смена названия (если потребуется)

1. **Изменить в package.json:**
   ```json
   {
     "name": "новое-название",
     "displayName": "Отображаемое название"
   }
   ```

2. **Обновить версию:**
   ```bash
   npm run version:patch
   ```

3. **Пересобрать:**
   ```bash
   npm run rebuild:extension
   ```

4. **Опубликовать:**
   ```bash
   npm run publish:marketplace
   ```

## 🎯 Рекомендации

1. **Первый выбор:** `bsl-type-safety-analyzer` ✅ (уже выбран)
2. **Если занят:** `bsl-analyzer-advanced`
3. **Enterprise версия:** `bsl-analyzer-enterprise`
4. **Creative вариант:** `bsl-sentinel`

## 📊 Текущий статус

- ✅ Название изменено на `bsl-type-safety-analyzer`
- ✅ Версия обновлена до 1.4.2
- 🔄 Расширение пересобирается
- ⏳ Готовится к публикации