#[cfg(test)]
mod tests {
    use crate::unified_index::{
        ConfigurationXmlParser,
        BslEntity, BslEntityType, BslEntityKind, BslContext,
    };
    use std::fs;
    use tempfile::TempDir;

    fn create_test_form_xml() -> String {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Form xmlns="http://v8.1c.ru/8.3/xcf/logform" xmlns:app="http://v8.1c.ru/8.2/managed-application/core" xmlns:cfg="http://v8.1c.ru/8.1/data/enterprise/current-config" xmlns:dcscor="http://v8.1c.ru/8.1/data-composition-system/core" xmlns:dcsset="http://v8.1c.ru/8.1/data-composition-system/settings" xmlns:ent="http://v8.1c.ru/8.1/data/enterprise" xmlns:lf="http://v8.1c.ru/8.2/managed-application/logform" xmlns:style="http://v8.1c.ru/8.1/data/ui/style" xmlns:sys="http://v8.1c.ru/8.1/data/ui/fonts/system" xmlns:v8="http://v8.1c.ru/8.1/data/core" xmlns:v8ui="http://v8.1c.ru/8.1/data/ui" xmlns:web="http://v8.1c.ru/8.1/data/ui/colors/web" xmlns:win="http://v8.1c.ru/8.1/data/ui/colors/windows" xmlns:xen="http://v8.1c.ru/8.3/xcf/enums" xmlns:xpr="http://v8.1c.ru/8.3/xcf/predef" xmlns:xr="http://v8.1c.ru/8.3/xcf/readable" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" version="2.15">
    <AutoCommandBar name="ФормаКоманднаяПанель" id="1"/>
    <Attributes>
        <Attribute name="Объект" id="1">
            <Type>
                <v8:Type>cfg:CatalogObject.Номенклатура</v8:Type>
            </Type>
            <MainAttribute>true</MainAttribute>
            <SavedData>true</SavedData>
        </Attribute>
    </Attributes>
    <Commands>
        <Command name="Записать" id="1">
            <Title>
                <v8:item>
                    <v8:lang>ru</v8:lang>
                    <v8:content>Записать</v8:content>
                </v8:item>
            </Title>
            <ToolTip>
                <v8:item>
                    <v8:lang>ru</v8:lang>
                    <v8:content>Записать объект</v8:content>
                </v8:item>
            </ToolTip>
            <Action>Записать</Action>
        </Command>
        <Command name="ЗаписатьИЗакрыть" id="2">
            <Title>
                <v8:item>
                    <v8:lang>ru</v8:lang>
                    <v8:content>Записать и закрыть</v8:content>
                </v8:item>
            </Title>
            <Action>ЗаписатьИЗакрыть</Action>
        </Command>
    </Commands>
    <Items>
        <UsualGroup name="ГруппаОсновная" id="3">
            <Title>
                <v8:item>
                    <v8:lang>ru</v8:lang>
                    <v8:content>Основная</v8:content>
                </v8:item>
            </Title>
            <Group>Vertical</Group>
            <Representation>None</Representation>
            <ExtendedTooltip name="ГруппаОсновнаяРасширеннаяПодсказка" id="4"/>
            <ChildItems>
                <InputField name="Код" id="5">
                    <DataPath>Объект.Код</DataPath>
                    <EditMode>EnterOnInput</EditMode>
                    <ExtendedTooltip name="КодРасширеннаяПодсказка" id="6"/>
                    <ContextMenu name="КодКонтекстноеМеню" id="7"/>
                </InputField>
                <InputField name="Наименование" id="8">
                    <DataPath>Объект.Наименование</DataPath>
                    <EditMode>EnterOnInput</EditMode>
                    <ExtendedTooltip name="НаименованиеРасширеннаяПодсказка" id="9"/>
                    <ContextMenu name="НаименованиеКонтекстноеМеню" id="10"/>
                </InputField>
            </ChildItems>
        </UsualGroup>
        <Table name="ТаблицаХарактеристики" id="11">
            <DataPath>Объект.Характеристики</DataPath>
            <AutoInsertNewRow>true</AutoInsertNewRow>
            <EnableStartDrag>true</EnableStartDrag>
            <EnableDrag>true</EnableDrag>
            <ExtendedTooltip name="ТаблицаХарактеристикиРасширеннаяПодсказка" id="12"/>
            <ContextMenu name="ТаблицаХарактеристикиКонтекстноеМеню" id="13"/>
            <AutoCommandBar name="ТаблицаХарактеристикиКоманднаяПанель" id="14"/>
            <SearchStringAddition name="ТаблицаХарактеристикиСтрокаПоиска" id="15"/>
            <ViewStatusAddition name="ТаблицаХарактеристикиСостояниеПросмотра" id="16"/>
            <SearchControlAddition name="ТаблицаХарактеристикиУправлениеПоиском" id="17"/>
        </Table>
    </Items>
</Form>"#.to_string()
    }

    fn create_test_config(dir: &TempDir) -> std::path::PathBuf {
        // Создаем структуру конфигурации
        let config_path = dir.path().join("test_config");
        
        // Configuration.xml
        fs::create_dir_all(&config_path).unwrap();
        fs::write(config_path.join("Configuration.xml"), r#"<?xml version="1.0" encoding="UTF-8"?>
<Configuration><Properties><Name>ТестоваяКонфигурация</Name></Properties></Configuration>"#).unwrap();
        
        // Справочник с формой
        let catalog_path = config_path.join("Catalogs").join("Номенклатура");
        fs::create_dir_all(&catalog_path).unwrap();
        fs::write(catalog_path.join("Номенклатура.xml"), r#"<?xml version="1.0" encoding="UTF-8"?>
<MetaDataObject xmlns="http://v8.1c.ru/8.3/MDClasses" name="Номенклатура">
    <Properties><Name>Номенклатура</Name></Properties>
</MetaDataObject>"#).unwrap();
        
        // Форма элемента
        let form_path = catalog_path.join("Forms").join("ФормаЭлемента").join("Ext");
        fs::create_dir_all(&form_path).unwrap();
        fs::write(form_path.join("Form.xml"), create_test_form_xml()).unwrap();
        
        // Общая форма
        let common_form_path = config_path.join("CommonForms").join("ВыборПериода").join("Ext");
        fs::create_dir_all(&common_form_path).unwrap();
        fs::write(common_form_path.join("Form.xml"), r#"<?xml version="1.0" encoding="UTF-8"?>
<Form xmlns="http://v8.1c.ru/8.3/xcf/logform" version="2.15">
    <Commands>
        <Command name="Выбрать" id="1">
            <Title>
                <v8:item>
                    <v8:lang>ru</v8:lang>
                    <v8:content>Выбрать</v8:content>
                </v8:item>
            </Title>
        </Command>
        <Command name="Отмена" id="2">
            <Title>
                <v8:item>
                    <v8:lang>ru</v8:lang>
                    <v8:content>Отмена</v8:content>
                </v8:item>
            </Title>
        </Command>
    </Commands>
    <Items>
        <Button name="КнопкаВыбрать" id="3">
            <Type>CommandBarButton</Type>
            <CommandName>Выбрать</CommandName>
        </Button>
    </Items>
</Form>"#).unwrap();
        
        config_path
    }

    #[test]
    fn test_parse_forms_integration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(&temp_dir);
        
        let parser = ConfigurationXmlParser::new(&config_path);
        let entities = parser.parse_configuration().unwrap();
        
        // Проверяем, что формы были найдены
        let form_entities: Vec<_> = entities.iter()
            .filter(|e| e.entity_type == BslEntityType::Form)
            .collect();
        
        assert_eq!(form_entities.len(), 2, "Должны быть найдены 2 формы");
        
        // Проверяем форму справочника
        let catalog_form = form_entities.iter()
            .find(|e| e.qualified_name.contains("Номенклатура.Form.ФормаЭлемента"))
            .expect("Форма справочника должна быть найдена");
        
        assert_eq!(catalog_form.display_name, "ФормаЭлемента");
        assert_eq!(catalog_form.entity_kind, BslEntityKind::ManagedForm);
        
        // Проверяем команды формы
        assert!(catalog_form.interface.methods.contains_key("Записать"));
        assert!(catalog_form.interface.methods.contains_key("ЗаписатьИЗакрыть"));
        
        // Проверяем элементы формы в extended_data
        let form_items = catalog_form.extended_data.get("form_items")
            .expect("Должны быть сохранены элементы формы");
        
        if let serde_json::Value::Array(items) = form_items {
            assert!(items.len() > 0, "Должны быть элементы формы");
            
            // Проверяем наличие конкретных элементов
            let items_str: Vec<String> = items.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            
            assert!(items_str.iter().any(|s| s.contains("Код")));
            assert!(items_str.iter().any(|s| s.contains("Наименование")));
            assert!(items_str.iter().any(|s| s.contains("ТаблицаХарактеристики")));
        }
        
        // Проверяем общую форму
        let common_form = form_entities.iter()
            .find(|e| e.qualified_name.contains("ОбщиеФормы.Form.ВыборПериода"))
            .expect("Общая форма должна быть найдена");
        
        assert!(common_form.interface.methods.contains_key("Выбрать"));
        assert!(common_form.interface.methods.contains_key("Отмена"));
    }
    
    #[test]
    fn test_form_xml_parsing_details() {
        let temp_dir = TempDir::new().unwrap();
        let form_xml_path = temp_dir.path().join("Form.xml");
        fs::write(&form_xml_path, create_test_form_xml()).unwrap();
        
        let parser = ConfigurationXmlParser::new(temp_dir.path());
        let form_entity = parser.parse_form_xml(&form_xml_path, "Справочники.Номенклатура", "ФормаЭлемента").unwrap();
        
        // Проверяем базовые свойства
        assert_eq!(form_entity.display_name, "ФормаЭлемента");
        assert_eq!(form_entity.qualified_name, "Справочники.Номенклатура.Form.ФормаЭлемента");
        assert_eq!(form_entity.entity_type, BslEntityType::Form);
        
        // Проверяем owner relationship
        assert_eq!(form_entity.relationships.owner, Some("Справочники.Номенклатура".to_string()));
        
        // Проверяем команды
        let commands: Vec<_> = form_entity.interface.methods.keys().collect();
        assert_eq!(commands.len(), 2);
        assert!(commands.contains(&&"Записать".to_string()));
        assert!(commands.contains(&&"ЗаписатьИЗакрыть".to_string()));
        
        // Проверяем методы-команды
        let save_method = &form_entity.interface.methods["Записать"];
        assert_eq!(save_method.name, "Записать");
        assert_eq!(save_method.availability, vec![BslContext::Client]);
        assert!(!save_method.is_function);
        
        // Проверяем элементы формы
        let form_items = form_entity.extended_data.get("form_items").unwrap();
        if let serde_json::Value::Array(items) = form_items {
            assert!(items.len() >= 4); // Группа, 2 поля, таблица
        }
    }
}