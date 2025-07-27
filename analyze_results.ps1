# Анализ результатов парсинга
$metadata_path = "test_output\metadata"
$forms_path = "test_output\forms"

Write-Host "=== Анализ результатов парсинга ===" -ForegroundColor Cyan

# Подсчет метаданных
$metadata_files = Get-ChildItem "$metadata_path\*.json" -File
Write-Host "`nМетаданные:" -ForegroundColor Green
Write-Host "Всего объектов: $($metadata_files.Count)"

# Группировка по типам объектов
$types = @{}
foreach ($file in $metadata_files) {
    $content = Get-Content $file.FullName -Raw | ConvertFrom-Json
    $type = $content.object_type
    if ($types.ContainsKey($type)) {
        $types[$type]++
    } else {
        $types[$type] = 1
    }
}

Write-Host "`nРаспределение по типам:"
$types.GetEnumerator() | Sort-Object Value -Descending | ForEach-Object {
    Write-Host "  $($_.Key): $($_.Value)"
}

# Подсчет форм
$form_files = Get-ChildItem "$forms_path\*.json" -File
Write-Host "`nФормы:" -ForegroundColor Green
Write-Host "Всего форм: $($form_files.Count)"

# Группировка по типам форм
$form_types = @{}
foreach ($file in $form_files | Select-Object -First 100) {
    $content = Get-Content $file.FullName -Raw | ConvertFrom-Json
    $type = $content.form_type
    if ($form_types.ContainsKey($type)) {
        $form_types[$type]++
    } else {
        $form_types[$type] = 1
    }
}

Write-Host "`nРаспределение по типам форм (первые 100):"
$form_types.GetEnumerator() | Sort-Object Value -Descending | ForEach-Object {
    Write-Host "  $($_.Key): $($_.Value)"
}

# Примеры с jq
Write-Host "`n=== Примеры использования jq ===" -ForegroundColor Cyan
Write-Host "# Получить имена всех справочников:"
Write-Host "jq -r 'select(.object_type == `"Directory`") | .name' test_output/metadata/*.json"

Write-Host "`n# Подсчитать количество реквизитов в объекте:"
Write-Host "jq '.structure.attributes | length' `"test_output/metadata/Документ.АвансовыйОтчет.json`""

Write-Host "`n# Получить все табличные части документа:"
Write-Host "jq -r '.structure.tabular_sections[].name' `"test_output/metadata/Документ.АвансовыйОтчет.json`""