//! BSL Grammar для tree-sitter
//! 
//! Этот модуль содержит определение грамматики BSL для парсера.
//! В будущем будет заменен на внешнюю tree-sitter грамматику.

use tree_sitter::{Language, Node};

/// Временная заглушка для BSL языка
/// TODO: Заменить на реальную tree-sitter-bsl грамматику
pub fn language() -> Language {
    // Когда будет готова грамматика, здесь будет:
    // unsafe { tree_sitter_bsl::language() }
    unimplemented!("BSL grammar not yet implemented")
}

/// Типы узлов в BSL грамматике
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    // Модуль
    Module,
    
    // Директивы
    AtClient,
    AtServer,
    AtServerNoContext,
    AtClientAtServer,
    AtClientAtServerNoContext,
    
    // Объявления
    ProcedureDeclaration,
    FunctionDeclaration,
    VariableDeclaration,
    
    // Параметры
    ParameterList,
    Parameter,
    DefaultValue,
    
    // Операторы
    ExpressionStatement,
    AssignmentStatement,
    IfStatement,
    ElseIfClause,
    ElseClause,
    WhileStatement,
    ForStatement,
    ForEachStatement,
    ReturnStatement,
    BreakStatement,
    ContinueStatement,
    TryStatement,
    ExceptClause,
    
    // Выражения
    Identifier,
    Number,
    String,
    Boolean,
    Date,
    Undefined,
    Null,
    
    // Составные выражения
    MethodCall,
    PropertyAccess,
    NewExpression,
    BinaryExpression,
    UnaryExpression,
    TernaryExpression,
    IndexAccess,
    
    // Операторы
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessOrEqual,
    GreaterOrEqual,
    And,
    Or,
    Not,
    
    // Ключевые слова
    Procedure,
    EndProcedure,
    Function,
    EndFunction,
    Var,
    Export,
    Val,
    If,
    Then,
    ElseIf,
    Else,
    EndIf,
    While,
    Do,
    EndDo,
    For,
    To,
    Each,
    In,
    Return,
    Break,
    Continue,
    Try,
    Except,
    EndTry,
    New,
    True,
    False,
    
    // Специальные
    Comment,
    Error,
}

impl NodeKind {
    /// Преобразует строку в тип узла
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "module" => Some(Self::Module),
            "procedure_declaration" => Some(Self::ProcedureDeclaration),
            "function_declaration" => Some(Self::FunctionDeclaration),
            "variable_declaration" => Some(Self::VariableDeclaration),
            "identifier" => Some(Self::Identifier),
            "number" => Some(Self::Number),
            "string" => Some(Self::String),
            "method_call" => Some(Self::MethodCall),
            "property_access" => Some(Self::PropertyAccess),
            "new_expression" => Some(Self::NewExpression),
            _ => None,
        }
    }
}

/// Проверяет, является ли узел определенного типа
pub fn is_node_kind(node: &Node, kind: NodeKind) -> bool {
    NodeKind::from_str(node.kind()) == Some(kind)
}

/// Грамматика BSL в формате tree-sitter (упрощенная версия)
/// 
/// ```javascript
/// module.exports = grammar({
///   name: 'bsl',
///   
///   rules: {
///     module: $ => repeat($._declaration),
///     
///     _declaration: $ => choice(
///       $.procedure_declaration,
///       $.function_declaration,
///       $.variable_declaration
///     ),
///     
///     procedure_declaration: $ => seq(
///       optional($.compiler_directive),
///       choice('Процедура', 'Procedure'),
///       $.identifier,
///       '(',
///       optional($.parameter_list),
///       ')',
///       optional('Экспорт', 'Export'),
///       repeat($._statement),
///       choice('КонецПроцедуры', 'EndProcedure')
///     ),
///     
///     function_declaration: $ => seq(
///       optional($.compiler_directive),
///       choice('Функция', 'Function'),
///       $.identifier,
///       '(',
///       optional($.parameter_list),
///       ')',
///       optional('Экспорт', 'Export'),
///       repeat($._statement),
///       choice('КонецФункции', 'EndFunction')
///     ),
///     
///     method_call: $ => prec.left(seq(
///       $._expression,
///       '.',
///       $.identifier,
///       '(',
///       optional($.argument_list),
///       ')'
///     )),
///     
///     property_access: $ => prec.left(seq(
///       $._expression,
///       '.',
///       $.identifier
///     )),
///     
///     new_expression: $ => seq(
///       choice('Новый', 'New'),
///       $.identifier,
///       '(',
///       optional($.argument_list),
///       ')'
///     ),
///     
///     identifier: $ => /[а-яА-Яa-zA-Z_][а-яА-Яa-zA-Z0-9_]*/,
///     
///     number: $ => /\d+(\.\d+)?/,
///     
///     string: $ => /"([^"\\]|\\.)*"/,
///     
///     compiler_directive: $ => choice(
///       '&НаКлиенте',
///       '&НаСервере',
///       '&НаСервереБезКонтекста',
///       '&НаКлиентеНаСервереБезКонтекста',
///       '&НаКлиентеНаСервере',
///       '&AtClient',
///       '&AtServer',
///       '&AtServerNoContext',
///       '&AtClientAtServerNoContext',
///       '&AtClientAtServer'
///     )
///   }
/// });
/// ```
pub const BSL_GRAMMAR_JS: &str = r#"
module.exports = grammar({
  name: 'bsl',
  
  // Дополнительные настройки
  extras: $ => [
    /\s/,
    $.comment
  ],
  
  // Приоритеты операторов
  precedences: $ => [
    ['unary', 'binary', 'ternary'],
    ['multiply', 'add'],
    ['compare', 'logical']
  ],
  
  rules: {
    // Основное правило - модуль
    module: $ => repeat($._top_level_item),
    
    _top_level_item: $ => choice(
      $.compiler_directive,
      $._declaration,
      $._statement
    ),
    
    // Директивы компиляции
    compiler_directive: $ => choice(
      '&НаКлиенте',
      '&НаСервере',
      '&НаСервереБезКонтекста',
      '&НаКлиентеНаСервереБезКонтекста',
      '&НаКлиентеНаСервере',
      '&AtClient',
      '&AtServer',
      '&AtServerNoContext',
      '&AtClientAtServerNoContext',
      '&AtClientAtServer'
    ),
    
    // Объявления
    _declaration: $ => choice(
      $.procedure_declaration,
      $.function_declaration,
      $.variable_declaration
    ),
    
    // Процедура
    procedure_declaration: $ => seq(
      optional($.compiler_directive),
      choice('Процедура', 'Procedure'),
      field('name', $.identifier),
      '(',
      optional($.parameter_list),
      ')',
      optional(choice('Экспорт', 'Export')),
      repeat($._statement),
      choice('КонецПроцедуры', 'EndProcedure')
    ),
    
    // Функция
    function_declaration: $ => seq(
      optional($.compiler_directive),
      choice('Функция', 'Function'),
      field('name', $.identifier),
      '(',
      optional($.parameter_list),
      ')',
      optional(choice('Экспорт', 'Export')),
      repeat($._statement),
      choice('КонецФункции', 'EndFunction')
    ),
    
    // Список параметров
    parameter_list: $ => commaSep1($.parameter),
    
    parameter: $ => seq(
      optional(choice('Знач', 'Val')),
      field('name', $.identifier),
      optional(seq('=', field('default', $._expression)))
    ),
    
    // Объявление переменных
    variable_declaration: $ => seq(
      choice('Перем', 'Var'),
      commaSep1($.identifier),
      optional(choice('Экспорт', 'Export')),
      ';'
    ),
    
    // Операторы
    _statement: $ => choice(
      $.expression_statement,
      $.assignment_statement,
      $.if_statement,
      $.while_statement,
      $.for_statement,
      $.for_each_statement,
      $.return_statement,
      $.break_statement,
      $.continue_statement,
      $.try_statement,
      ';'
    ),
    
    expression_statement: $ => seq($._expression, ';'),
    
    assignment_statement: $ => seq(
      field('left', $._expression),
      '=',
      field('right', $._expression),
      ';'
    ),
    
    // Условный оператор
    if_statement: $ => seq(
      choice('Если', 'If'),
      field('condition', $._expression),
      choice('Тогда', 'Then'),
      repeat($._statement),
      repeat($.elseif_clause),
      optional($.else_clause),
      choice('КонецЕсли', 'EndIf')
    ),
    
    elseif_clause: $ => seq(
      choice('ИначеЕсли', 'ElseIf'),
      field('condition', $._expression),
      choice('Тогда', 'Then'),
      repeat($._statement)
    ),
    
    else_clause: $ => seq(
      choice('Иначе', 'Else'),
      repeat($._statement)
    ),
    
    // Циклы
    while_statement: $ => seq(
      choice('Пока', 'While'),
      field('condition', $._expression),
      choice('Цикл', 'Do'),
      repeat($._statement),
      choice('КонецЦикла', 'EndDo')
    ),
    
    for_statement: $ => seq(
      choice('Для', 'For'),
      field('variable', $.identifier),
      '=',
      field('from', $._expression),
      choice('По', 'To'),
      field('to', $._expression),
      choice('Цикл', 'Do'),
      repeat($._statement),
      choice('КонецЦикла', 'EndDo')
    ),
    
    for_each_statement: $ => seq(
      choice('Для', 'For'),
      choice('Каждого', 'Each'),
      field('variable', $.identifier),
      choice('Из', 'In'),
      field('collection', $._expression),
      choice('Цикл', 'Do'),
      repeat($._statement),
      choice('КонецЦикла', 'EndDo')
    ),
    
    // Управляющие операторы
    return_statement: $ => seq(
      choice('Возврат', 'Return'),
      optional($._expression),
      ';'
    ),
    
    break_statement: $ => seq(choice('Прервать', 'Break'), ';'),
    
    continue_statement: $ => seq(choice('Продолжить', 'Continue'), ';'),
    
    // Обработка исключений
    try_statement: $ => seq(
      choice('Попытка', 'Try'),
      repeat($._statement),
      choice('Исключение', 'Except'),
      repeat($._statement),
      choice('КонецПопытки', 'EndTry')
    ),
    
    // Выражения
    _expression: $ => choice(
      $.identifier,
      $.number,
      $.string,
      $.boolean,
      $.date,
      $.undefined,
      $.null,
      $.method_call,
      $.property_access,
      $.new_expression,
      $.binary_expression,
      $.unary_expression,
      $.ternary_expression,
      $.index_access,
      $.parenthesized_expression
    ),
    
    // Вызов метода
    method_call: $ => prec.left('call', seq(
      field('object', $._expression),
      '.',
      field('method', $.identifier),
      '(',
      optional($.argument_list),
      ')'
    )),
    
    // Обращение к свойству
    property_access: $ => prec.left('member', seq(
      field('object', $._expression),
      '.',
      field('property', $.identifier)
    )),
    
    // Создание объекта
    new_expression: $ => seq(
      choice('Новый', 'New'),
      field('type', $.identifier),
      '(',
      optional($.argument_list),
      ')'
    ),
    
    // Список аргументов
    argument_list: $ => commaSep1($._expression),
    
    // Бинарные операции
    binary_expression: $ => choice(
      prec.left('multiply', seq($._expression, choice('*', '/', '%'), $._expression)),
      prec.left('add', seq($._expression, choice('+', '-'), $._expression)),
      prec.left('compare', seq($._expression, choice('<', '>', '<=', '>=', '=', '<>'), $._expression)),
      prec.left('logical', seq($._expression, choice('И', 'ИЛИ', 'And', 'Or'), $._expression))
    ),
    
    // Унарные операции
    unary_expression: $ => prec('unary', choice(
      seq(choice('НЕ', 'Not'), $._expression),
      seq('-', $._expression)
    )),
    
    // Тернарная операция
    ternary_expression: $ => prec('ternary', seq(
      field('condition', $._expression),
      '?',
      field('then', $._expression),
      ':',
      field('else', $._expression)
    )),
    
    // Индексный доступ
    index_access: $ => prec.left('member', seq(
      field('object', $._expression),
      '[',
      field('index', $._expression),
      ']'
    )),
    
    // Выражение в скобках
    parenthesized_expression: $ => seq('(', $._expression, ')'),
    
    // Литералы
    identifier: $ => /[а-яА-Яa-zA-Z_][а-яА-Яa-zA-Z0-9_]*/,
    
    number: $ => /\d+(\.\d+)?/,
    
    string: $ => /"([^"\\]|\\.)*"/,
    
    boolean: $ => choice(
      'Истина', 'Ложь',
      'True', 'False'
    ),
    
    date: $ => /'(\d{8}|\d{14})'/,
    
    undefined: $ => choice('Неопределено', 'Undefined'),
    
    null: $ => 'Null',
    
    // Комментарии
    comment: $ => token(choice(
      seq('//', /.*/),
      seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')
    ))
  }
});

// Вспомогательные функции
function commaSep1(rule) {
  return seq(rule, repeat(seq(',', rule)));
}
"#;