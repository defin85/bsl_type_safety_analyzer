/*!
# Integration Tests for Enhanced BSL Analyzer

Тестирует интеграцию новых парсеров с основным анализатором.
*/

use bsl_analyzer::{
    Configuration, MetadataReportParser, FormXmlParser,
    DocsIntegration, ObjectType, FormType
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_metadata_report_parser_integration() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path();
    
    // Создаем пример текстового отчета конфигурации
    let report_content = r#"Справочник.Номенклатура
Реквизиты:
  Код (Строка(9))
  Наименование (Строка(150))
  Артикул (Строка(50))
Табличные части:
  Характеристики:
    Свойство (Строка(100))
    Значение (Строка(200))

Документ.ПриходнаяНакладная
Реквизиты:
  Номер (Строка(11))
  Дата (Дата)
  Организация (СправочникСсылка.Организации)
Табличные части:
  Товары:
    Номенклатура (СправочникСсылка.Номенклатура)
    Количество (Число(15,3))
    Цена (Число(15,2))
"#;
    
    fs::write(config_dir.join("config_report.txt"), report_content).unwrap();
    
    // Тестируем парсер отчетов метаданных
    let parser = MetadataReportParser::new().unwrap();
    let contracts = parser.parse_report(config_dir.join("config_report.txt")).unwrap();
    
    assert_eq!(contracts.len(), 2);
    
    // Проверяем справочник
    let directory = contracts.iter().find(|c| c.name == "Номенклатура").unwrap();
    assert_eq!(directory.object_type, ObjectType::Directory);
    assert_eq!(directory.structure.attributes.len(), 3);
    assert_eq!(directory.structure.tabular_sections.len(), 1);
    
    // Проверяем документ
    let document = contracts.iter().find(|c| c.name == "ПриходнаяНакладная").unwrap();
    assert_eq!(document.object_type, ObjectType::Document);
    assert_eq!(document.structure.attributes.len(), 3);
    assert_eq!(document.structure.tabular_sections.len(), 1);
    
    println!("✅ Metadata report parser integration test passed");
}

#[test]
fn test_form_xml_parser_integration() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path();
    
    // Создаем структуру с XML формой
    let form_dir = config_dir.join("Catalogs").join("Items").join("Forms").join("ItemForm");
    fs::create_dir_all(&form_dir).unwrap();
    
    let form_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Form xmlns="http://v8.1c.ru/8.3/xcf/logform" xmlns:app="http://v8.1c.ru/8.2/managed-application/core">
    <Items>
        <InputField name="Code" Title="Код">
            <DataPath>Object.Code</DataPath>
        </InputField>
        <InputField name="Description" Title="Наименование">
            <DataPath>Object.Description</DataPath>
        </InputField>
        <Button name="OK" Title="ОК">
            <Action>OK</Action>
        </Button>
    </Items>
    <Attributes>
        <Attribute name="Object" Type="CatalogObject.Items" SaveData="true"/>
    </Attributes>
    <Commands>
        <Command name="Save" Title="Сохранить" Action="Write"/>
    </Commands>
</Form>"#;
    
    fs::write(form_dir.join("Form.xml"), form_xml).unwrap();
    
    // Тестируем парсер XML форм
    let parser = FormXmlParser::new();
    let contracts = parser.generate_all_contracts(config_dir).unwrap();
    
    assert_eq!(contracts.len(), 1);
    
    let form = &contracts[0];
    assert_eq!(form.name, "ItemForm");
    assert_eq!(form.structure.elements.len(), 3); // 2 InputField + 1 Button
    assert_eq!(form.structure.attributes.len(), 1);
    assert_eq!(form.structure.commands.len(), 1);
    assert!(matches!(form.form_type, FormType::ItemForm));
    
    println!("✅ Form XML parser integration test passed");
}

#[test]
fn test_configuration_with_integrated_parsers() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path();
    
    // Создаем минимальную конфигурацию для тестирования
    // Configuration.xml
    let metadata_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Configuration>
    <Name>TestConfiguration</Name>
    <Version>1.0.0</Version>
</Configuration>"#;
    
    fs::write(config_dir.join("Configuration.xml"), metadata_xml).unwrap();
    
    // BSL модуль
    let module_content = r#"
Процедура ТестоваяПроцедура() Экспорт
    Сообщить("Тест");
КонецПроцедуры
    "#;
    
    fs::create_dir_all(config_dir.join("CommonModules").join("ОбщийМодуль")).unwrap();
    fs::write(
        config_dir.join("CommonModules").join("ОбщийМодуль").join("Module.bsl"),
        module_content
    ).unwrap();
    
    // Отчет конфигурации
    let report_content = r#"Справочник.Тест
Реквизиты:
  Код (Строка(9))
  Наименование (Строка(150))
"#;
    
    fs::write(config_dir.join("config_report.txt"), report_content).unwrap();
    
    // Загружаем конфигурацию с интегрированными парсерами
    let config = Configuration::load_from_directory(config_dir).unwrap();
    
    // Проверяем базовую функциональность
    assert_eq!(config.metadata.name, "TestConfiguration");
    assert!(!config.modules.is_empty());
    
    // Проверяем новые возможности
    assert_eq!(config.metadata_contracts.len(), 1);
    assert_eq!(config.forms.len(), 0); // Нет XML форм в тесте
    
    // Проверяем метаданные
    let contract = &config.metadata_contracts[0];
    assert_eq!(contract.name, "Тест");
    assert_eq!(contract.object_type, ObjectType::Directory);
    assert_eq!(contract.structure.attributes.len(), 2);
    
    // Проверяем статистику
    let stats = config.statistics();
    assert_eq!(stats.metadata_contracts_count, 1);
    assert_eq!(stats.forms_count, 0);
    
    println!("✅ Configuration with integrated parsers test passed");
}

#[test]
fn test_docs_integration_creation() {
    // Тестируем создание интеграции документации
    let docs = DocsIntegration::new();
    assert!(!docs.is_loaded());
    assert!(docs.get_statistics().is_none());
    
    // Тестируем методы без загруженной документации
    assert!(docs.get_method_info("Сообщить").is_none());
    assert!(docs.get_completions("Сооб").is_empty());
    assert!(docs.search_methods("Сообщить").is_empty());
    
    println!("✅ Documentation integration creation test passed");
}

#[test]
fn test_metadata_contract_search() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path();
    
    // Создаем отчет с несколькими объектами
    let report_content = r#"Справочник.Номенклатура
Реквизиты:
  Код (Строка(9))

Справочник.Контрагенты  
Реквизиты:
  Наименование (Строка(150))

Документ.Заказ
Реквизиты:
  Номер (Строка(11))
  Дата (Дата)
"#;
    
    fs::write(config_dir.join("config_report.txt"), report_content).unwrap();
    fs::write(config_dir.join("Configuration.xml"), "<Configuration><Name>Test</Name></Configuration>").unwrap();
    
    let config = Configuration::load_from_directory(config_dir).unwrap();
    
    // Тестируем поиск контрактов по типу
    let directories = config.get_metadata_contracts_by_type(&ObjectType::Directory);
    assert_eq!(directories.len(), 2);
    
    let documents = config.get_metadata_contracts_by_type(&ObjectType::Document);
    assert_eq!(documents.len(), 1);
    
    // Тестируем поиск по имени
    assert!(config.get_metadata_contract("Номенклатура").is_some());
    assert!(config.get_metadata_contract("НесуществующийОбъект").is_none());
    
    println!("✅ Metadata contract search test passed");
}

#[test]
fn test_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path();
    
    // Тестируем обработку отсутствующего отчета конфигурации
    fs::write(config_dir.join("Configuration.xml"), "<Configuration><Name>Test</Name></Configuration>").unwrap();
    
    let config = Configuration::load_from_directory(config_dir).unwrap();
    
    // Должен загрузиться без ошибок, но с пустыми контрактами
    assert_eq!(config.metadata_contracts.len(), 0);
    assert_eq!(config.forms.len(), 0);
    
    // Тестируем некорректный XML форм
    let form_dir = config_dir.join("Forms").join("BadForm");
    fs::create_dir_all(&form_dir).unwrap();
    fs::write(form_dir.join("Form.xml"), "<?xml version='1.0'?><InvalidXml>").unwrap();
    
    // Парсер должен пропустить некорректный XML без падения
    let parser = FormXmlParser::new();
    let contracts = parser.generate_all_contracts(config_dir).unwrap();
    assert_eq!(contracts.len(), 0); // Некорректный XML пропущен
    
    println!("✅ Error handling test passed");
}