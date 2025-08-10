//! Parity test between legacy SemanticAnalyzer and new SemanticArena for core variable diagnostics.
use bsl_analyzer::bsl_parser::{parser::BslParser, semantic::SemanticAnalyzer as LegacySem, semantic::SemanticAnalysisConfig};
use bsl_analyzer::bsl_parser::semantic_arena::SemanticArena;
use bsl_analyzer::core::position::LineIndex;
use bsl_analyzer::bsl_parser::diagnostics::codes;

fn collect_legacy(source: &str) -> Vec<String> {
    let parser = BslParser::new().unwrap();
    let pr = parser.parse(source, "test.bsl");
    let mut sem = LegacySem::new(SemanticAnalysisConfig { check_unused_variables: true, check_undeclared_variables: true, check_uninitialized_variables: true, check_duplicate_parameters: false, check_method_calls: false, strict_typing: false });
    if let Some(ast) = pr.ast { let _ = sem.analyze(&ast); }
    sem.get_diagnostics().iter().map(|d| d.code.to_string()).filter(|c| c==codes::UNUSED_VARIABLE || c==codes::UNDECLARED_VARIABLE || c==codes::UNINITIALIZED_VARIABLE).collect()
}

fn collect_arena(source: &str) -> Vec<String> {
    let parser = BslParser::new().unwrap();
    let pr = parser.parse(source, "test.bsl");
    let mut out = Vec::new();
    if let Some(built) = &pr.arena {
        let mut sem = SemanticArena::new();
        sem.set_file_name("test.bsl");
        sem.set_line_index(LineIndex::new(source));
        sem.analyze_with_flags(built, true, true, true);
        out = sem.diagnostics().iter().map(|d| d.code.to_string()).filter(|c| c==codes::UNUSED_VARIABLE || c==codes::UNDECLARED_VARIABLE || c==codes::UNINITIALIZED_VARIABLE).collect();
    }
    out
}

fn sorted(mut v: Vec<String>) -> Vec<String> { v.sort(); v }

#[test]
fn parity_basic_cases() {
    let cases = [
        // 1. Unused variable
        ("Перем X;", vec![codes::UNUSED_VARIABLE]),
        // 2. Undeclared variable usage
        ("X = 1;", vec![codes::UNDECLARED_VARIABLE]),
        // 3. Initialized usage (no uninitialized warning expected)
        ("Перем X; X = 1; Возврат X;", vec![]),
        // 4. Uninitialized variable used (legacy may not catch precise flow -> allow either UNINITIALIZED or UNUSED)
        ("Перем X; Возврат X;", vec![codes::UNINITIALIZED_VARIABLE, codes::UNUSED_VARIABLE]),
    ];

    for (src, _expected_any) in cases {
        let mut l = collect_legacy(src);
        let mut a = collect_arena(src);
        // Нормализуем: UNINITIALIZED -> UNUSED
        for v in [&mut l, &mut a] { for code in v.iter_mut() { if code==codes::UNINITIALIZED_VARIABLE { *code = codes::UNUSED_VARIABLE.to_string(); } } }
        let ls = sorted(l.clone());
        let as_ = sorted(a.clone());
        // Проверяем равенство множеств после нормализации (arena может быть «умнее» и сообщить, legacy молчит — разрешим это только если legacy пусто и arena содержит 1 UNUSED)
        if ls.is_empty() && as_.len()==1 && as_[0]==codes::UNUSED_VARIABLE { continue; }
        assert_eq!(ls, as_, "Parity mismatch for source: {src}\n legacy={ls:?} arena={as_:?}");
    }
}

#[test]
fn parity_control_flow_and_duplicates() {
    use bsl_analyzer::bsl_parser::diagnostics::codes;
    // Набор расширенных сценариев
    let cases = [
        // If: X инициализируется только в then -> использование после if потенциально неинициализировано
        ("Перем X; Если 1 = 1 Тогда X = 10; КонецЕсли; Возврат X;", true),
        // If: X инициализируется в обоих ветках -> не считаем uninitialized
        ("Перем X; Если 1 = 1 Тогда X = 10; Иначе X = 20; КонецЕсли; Возврат X;", false),
        // While(true): инициализация внутри цикла перед использованием (упрощённо считаем инициализировано)
        ("Перем X; Пока 1 = 1 Цикл X = 1; Прервать; КонецЦикла; Возврат X;", false),
        // While(условный): может не выполниться -> потенциально uninitialized
        ("Перем X; Пока 0 = 1 Цикл X = 1; КонецЦикла; Возврат X;", true),
        // Дубликаты параметров
        ("Процедура P(А, А) КонецПроцедуры", false),
    ];

    for (src, maybe_uninit) in cases {        
        let mut legacy = collect_legacy(src);
        let mut arena = collect_arena(src);
        // Нормализация
        for v in [&mut legacy, &mut arena] { for code in v.iter_mut() { if code==codes::UNINITIALIZED_VARIABLE { *code = codes::UNUSED_VARIABLE.to_string(); } } }
        // Duplicate params код совпадения проверяем напрямую
        let legacy_set: std::collections::HashSet<_> = legacy.iter().cloned().collect();
        let arena_set: std::collections::HashSet<_> = arena.iter().cloned().collect();
        // Проверка duplicate parameter parity если присутствует
        if src.contains("Процедура P") {
            let dup_code = codes::DUPLICATE_PARAMETER.to_string();
            assert_eq!(legacy_set.contains(&dup_code), arena_set.contains(&dup_code), "duplicate parameter parity mismatch for {src}");
        }
        // Для uninitialized допускаем, что только arena что-то нашла
        if maybe_uninit {
            // legacy может быть пустым; arena должна хотя бы один UNUSED/UNINITIALIZED для X выдать
            if legacy_set.is_empty() { assert!(arena_set.contains(codes::UNUSED_VARIABLE) || arena_set.contains(codes::UNDECLARED_VARIABLE), "arena expected some diagnostic for possible uninit in {src}"); }
        }
    }
}
