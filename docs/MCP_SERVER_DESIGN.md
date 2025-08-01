# MCP Server Design for BSL Type Safety Analyzer

<mcp-server>
  <metadata>
    <name>BSL Type Safety MCP Server</name>
    <version>1.0.0</version>
    <description>Model Context Protocol server providing BSL type system access to LLMs</description>
    <protocol-version>0.1.0</protocol-version>
  </metadata>

  <architecture>
    <component name="Core">
      <description>UnifiedBslIndex with 24,055+ types</description>
      <responsibilities>
        <item>Type storage and indexing</item>
        <item>Method/property resolution</item>
        <item>Type compatibility checking</item>
      </responsibilities>
    </component>
    
    <component name="MCP Handler">
      <description>Protocol implementation using tokio</description>
      <responsibilities>
        <item>Request/response handling</item>
        <item>Tool registration</item>
        <item>Error management</item>
      </responsibilities>
    </component>
  </architecture>

  <tools>
    <tool name="find_type">
      <description>Поиск типа в индексе BSL</description>
      <purpose>
        Найти информацию о любом типе BSL: платформенном, конфигурационном или форме.
        Поддерживает поиск на русском и английском языках.
      </purpose>
      <parameters>
        <parameter name="type_name" type="string" required="true">
          <description>Имя типа для поиска</description>
          <examples>
            <example>Массив</example>
            <example>Array</example>
            <example>Справочники.Номенклатура</example>
            <example>РегистрыСведений.КурсыВалют</example>
          </examples>
        </parameter>
        <parameter name="language_preference" type="string" default="auto">
          <description>Языковое предпочтение: russian, english, auto</description>
        </parameter>
      </parameters>
      <returns>
        <structure>
{
  "found": true,
  "entity": {
    "id": "Справочники.Номенклатура",
    "display_name": "Номенклатура",
    "entity_type": "Configuration",
    "entity_kind": "Catalog",
    "methods_count": 45,
    "properties_count": 12,
    "parent_types": ["СправочникОбъект"],
    "implements": ["СправочникСсылка"]
  },
  "suggestions": []
}
        </structure>
      </returns>
      <examples>
        <example>
          <request>
{
  "tool": "find_type",
  "arguments": {
    "type_name": "Массив"
  }
}
          </request>
          <response>
{
  "found": true,
  "entity": {
    "id": "Array",
    "display_name": "Массив",
    "entity_type": "Platform",
    "entity_kind": "Array",
    "methods_count": 15,
    "properties_count": 1
  }
}
          </response>
        </example>
      </examples>
    </tool>

    <tool name="get_type_methods">
      <description>Получить все методы типа включая унаследованные</description>
      <purpose>
        Получить полный список методов типа с параметрами, типами возврата и документацией.
        Включает методы, унаследованные от родительских типов.
      </purpose>
      <parameters>
        <parameter name="type_name" type="string" required="true">
          <description>Имя типа</description>
        </parameter>
        <parameter name="include_inherited" type="boolean" default="true">
          <description>Включить унаследованные методы</description>
        </parameter>
        <parameter name="filter_context" type="string" optional="true">
          <description>Фильтр по контексту: Client, Server, All</description>
        </parameter>
      </parameters>
      <returns>
        <structure>
{
  "type_name": "Справочники.Номенклатура",
  "total_methods": 45,
  "own_methods": 3,
  "inherited_methods": 42,
  "methods": [
    {
      "name": "Записать",
      "english_name": "Write",
      "is_function": false,
      "parameters": [
        {
          "name": "РежимЗаписи",
          "type": "РежимЗаписиДокумента",
          "optional": true,
          "default": "РежимЗаписиДокумента.Запись"
        }
      ],
      "availability": ["Client", "Server"],
      "inherited_from": "СправочникОбъект"
    }
  ]
}
        </structure>
      </returns>
    </tool>

    <tool name="check_type_compatibility">
      <description>Проверить совместимость типов для присваивания</description>
      <purpose>
        Проверить, можно ли присвоить значение одного типа переменной другого типа.
        Учитывает наследование и реализацию интерфейсов.
      </purpose>
      <parameters>
        <parameter name="from_type" type="string" required="true">
          <description>Исходный тип (что присваиваем)</description>
        </parameter>
        <parameter name="to_type" type="string" required="true">
          <description>Целевой тип (куда присваиваем)</description>
        </parameter>
      </parameters>
      <returns>
        <structure>
{
  "compatible": true,
  "reason": "implements_interface",
  "path": ["Справочники.Номенклатура", "implements", "СправочникСсылка"]
}
        </structure>
      </returns>
      <examples>
        <example>
          <context>Проверка присваивания справочника к ссылке</context>
          <request>
{
  "tool": "check_type_compatibility",
  "arguments": {
    "from_type": "Справочники.Номенклатура",
    "to_type": "СправочникСсылка"
  }
}
          </request>
          <response>
{
  "compatible": true,
  "reason": "implements_interface",
  "path": ["Справочники.Номенклатура", "implements", "СправочникСсылка"]
}
          </response>
        </example>
      </examples>
    </tool>

    <tool name="validate_method_call">
      <description>Проверить корректность вызова метода</description>
      <purpose>
        Валидировать вызов метода: существование метода у типа, корректность параметров,
        доступность в текущем контексте выполнения.
      </purpose>
      <parameters>
        <parameter name="object_type" type="string" required="true">
          <description>Тип объекта, у которого вызывается метод</description>
        </parameter>
        <parameter name="method_name" type="string" required="true">
          <description>Имя вызываемого метода</description>
        </parameter>
        <parameter name="arguments" type="array" optional="true">
          <description>Список аргументов вызова</description>
          <structure>
[
  {
    "value": "значение или выражение",
    "type": "предполагаемый тип"
  }
]
          </structure>
        </parameter>
        <parameter name="context" type="string" default="Server">
          <description>Контекст выполнения: Client, Server</description>
        </parameter>
      </parameters>
      <returns>
        <structure>
{
  "valid": true,
  "method": {
    "name": "Добавить",
    "parameters": [...],
    "return_type": "void"
  },
  "errors": [],
  "warnings": []
}
        </structure>
      </returns>
    </tool>
  </tools>

  <usage-guidelines>
    <guideline>
      <title>Оптимальное использование для LLM</title>
      <recommendations>
        <item>Всегда начинайте с find_type для проверки существования типа</item>
        <item>Используйте get_type_methods для понимания API объекта</item>
        <item>Проверяйте совместимость типов перед генерацией присваиваний</item>
        <item>Валидируйте вызовы методов для избежания ошибок runtime</item>
      </recommendations>
    </guideline>
    
    <guideline>
      <title>Обработка ошибок</title>
      <recommendations>
        <item>Тип не найден - предложите похожие из suggestions</item>
        <item>Метод недоступен в контексте - укажите правильный контекст</item>
        <item>Неверные параметры - покажите правильную сигнатуру</item>
      </recommendations>
    </guideline>
  </usage-guidelines>

  <implementation-notes>
    <note priority="high">
      MCP Server должен кешировать UnifiedBslIndex в памяти для производительности
    </note>
    <note priority="high">
      Поддержка streaming для больших результатов (список всех типов)
    </note>
    <note priority="medium">
      Логирование запросов для анализа использования LLM
    </note>
  </implementation-notes>
</mcp-server>

## Пример конфигурации для Claude

```json
{
  "mcpServers": {
    "bsl-types": {
      "command": "bsl-analyzer",
      "args": ["mcp-server", "--port", "7777"],
      "env": {
        "BSL_INDEX_PATH": "~/.bsl_analyzer/indices/current"
      }
    }
  }
}
```

## Rust Implementation Skeleton

```rust
/// <mcp-handler>
///   <purpose>Основной обработчик MCP запросов</purpose>
///   <responsibilities>
///     <item>Регистрация инструментов</item>
///     <item>Маршрутизация запросов</item>
///     <item>Сериализация ответов</item>
///   </responsibilities>
/// </mcp-handler>
pub struct McpHandler {
    index: Arc<UnifiedBslIndex>,
    tools: HashMap<String, Box<dyn Tool>>,
}

/// <tool-trait>
///   <purpose>Интерфейс для всех MCP инструментов</purpose>
/// </tool-trait>
#[async_trait]
trait Tool: Send + Sync {
    /// <method>
    ///   <name>name</name>
    ///   <returns>Уникальное имя инструмента</returns>
    /// </method>
    fn name(&self) -> &str;
    
    /// <method>
    ///   <name>execute</name>
    ///   <parameters>
    ///     <param name="args">JSON аргументы от LLM</param>
    ///   </parameters>
    ///   <returns>JSON результат для LLM</returns>
    /// </method>
    async fn execute(&self, args: Value) -> Result<Value>;
}
```