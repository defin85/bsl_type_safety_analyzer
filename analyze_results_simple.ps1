# Анализ результатов парсинга
$metadata_path = "test_output\metadata"
$forms_path = "test_output\forms"

Write-Host "=== Analysis Results ===" -ForegroundColor Cyan

# Подсчет метаданных
$metadata_files = Get-ChildItem "$metadata_path\*.json" -File
Write-Host "`nMetadata:" -ForegroundColor Green
Write-Host "Total objects: $($metadata_files.Count)"

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

Write-Host "`nDistribution by types:"
$types.GetEnumerator() | Sort-Object Value -Descending | ForEach-Object {
    Write-Host "  $($_.Key): $($_.Value)"
}

# Подсчет форм
$form_files = Get-ChildItem "$forms_path\*.json" -File
Write-Host "`nForms:" -ForegroundColor Green
Write-Host "Total forms: $($form_files.Count)"

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

Write-Host "`nForm types distribution (first 100):"
$form_types.GetEnumerator() | Sort-Object Value -Descending | ForEach-Object {
    Write-Host "  $($_.Key): $($_.Value)"
}

Write-Host "`n=== jq usage examples ===" -ForegroundColor Cyan
Write-Host "# Get all Directory names:"
Write-Host "find test_output/metadata -name '*.json' -exec jq -r 'select(.object_type == ""Directory"") | .name' {} \;"

Write-Host "`n# Count attributes in an object:"
Write-Host "jq '.structure.attributes | length' test_output/metadata/*.json"

Write-Host "`n# Get tabular sections:"
Write-Host "jq -r '.structure.tabular_sections[].name' test_output/metadata/*.json"