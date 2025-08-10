# AST Redesign Summary Plan

Концентрированный итоговый план эволюции AST / семантического слоя. Синтезирует идеи из AST_DESIGN_PROPOSAL, AST_IMPROVEMENT_PLAN и дополнительного анализа.

## Цели
- Производительность: минимальные аллокации, инкрементальные обновления, кэшируемость.
- Расширяемость: лёгкое добавление правил, метрик, refactor-инструментов.
- Устойчивость: корректная работа на частично неверном коде (LSP/live editing).
- Семантическая точность: чистое разделение синтаксики и семантики.
- Подготовка к форматтеру, рефакторингам и LLM-контексту.

## Архитектурные слои
1. Source Layer: FileId, текст, line_offsets.
2. CST (tree-sitter) как "green" источник (immut.).
3. Red Wrappers: типизированный доступ (ленивый, без копий).
4. AST Arena: минимальные узлы (NodeId, kind, children, packed_span, payload idx).
5. Side-Tables: types, symbols, method/property resolution, diagnostics, fingerprints.
6. Semantic Passes: name resolve → type inference → rule engine.
7. Incremental Engine: dirty ranges → affected NodeIds → selective recompute.
8. Presentation: LSP адаптеры, отчёты, сериализация.

## Ключевые концепты
- NodeId(u32) + Arena<Vec<Node>>
- PackedSpan { start_offset: u32, len: u32 }
- Интернирование идентификаторов и строк (SymbolId)
- ErrorNode для recovery и стабильного анализа
- Fingerprint (u64) на узел (kind + children + interned payload)
- Side-tables (dense Vec<Option<...>>) вместо хранения в AST
- Trivia (leading/trailing comments) — позже

## Фазы (минимально рискованная последовательность)

### Phase 0: Инфраструктура
- FileId + line index + PackedSpan.
- Метрики времени (parse, ast_build, sem_analysis).

### Phase 1: NodeId & Arena
- Ввести `AstKind` (минимальный набор: Module, Procedure, Function, Param, Block, Identifier, Literal, Call, Member, Assignment, New, Error).
- Builder: CST → Arena AST (без типов). Bridge удалить после адаптации анализаторов.

### Phase 2: Error Recovery & Visitor
- ErrorNode + интеграция с парсером.
- Генерик `AstVisitor` + утилиты обхода (preorder, children_of_kind).

### Phase 3: Semantic Side-Tables & Precise Spans (ACTUAL STATUS)
Оригинальный план предполагал полный переход на side-tables. Фактически на текущем этапе реализовано подмножество + расширения:

Выполнено:
- Arena AST (Phase 1) интегрирован в семантику, legacy путь заморожен.
- Точное позиционирование: `PackedSpan` заполнен для большинства узлов; блоки агрегируют span детей.
- `LineIndex` подключён: диагностики содержат корректные file name / line / column (по offset+len).
- Heуристические spans для Identifier / Method / Property / New / Binary / Unary выражений в конвертере.
- Диагностики перенесены на более точные места (напр., TYPE_MISMATCH теперь указывает на идентификатор цели обновления).
- Паритетные тесты (legacy vs arena) для переменных, control-flow (If/While), дубликатов — предотвращают регресс.
- Snapshot harness (`tests/fixtures/arena/*.bsl` + `_snapshots/*.json`) фиксирует базовый набор диагностик; обновление через `UPDATE_SNAPSHOTS=1`.
- Прототип side-table типов (`expr_types: Vec<Option<SimpleType>>`) внедрён в `SemanticArena`.
- Legacy semantic путь помечен DEPRECATED (будет удалён после стабильного релиза).

Частично / в очереди:
- Расширенные side-tables (symbols, методы/свойства с разрешениями) — впереди.
- Method/Property resolution сейчас минималистична (каталог для Array). Будет вынесено в отдельные таблицы.
- Interning строк и разделение payload по векторам — сдвинулось в будущие фазы (см. Phase 4).

Дельты к исходному плану:
- Добавлены parity и snapshot механизмы раньше интернинга, чтобы безопасно ускорить удаление legacy.
- Приоритет точных spans поднят в Phase 3 (изначально планировались базовые side-tables без детальной точности позиций).

Критерий завершения обновлён:
- Точные spans для ≥90% синтаксических конструкций (осталось уточнить offset после ключевого слова `Новый` и операторы в Binary).
- Паритетные тесты без расхождений.
- Snapshot базовый набор стабилен в CI.
- Отсутствуют предупреждения компилятора в новом семантическом пути.

Следующие шаги перед переходом к Phase 4:
1. Финализировать spans (операторы: окончательная политика позиционирования; NewExpression уже уточнён, при необходимости донастройка).
2. Добавить symbol table (идентификаторы → SymbolId) и подготовку к string interning.
3. Документировать процедуру полного удаления legacy (remove code + changelog заметка).

### Phase 4: Interning & Payload Split
- String interner (идентификаторы, строковые литералы) — ЧАСТИЧНО: внедрён `SymbolId` в `AstPayload::Ident` / `Literal` с дублированным текстом для обратной совместимости. Следующий шаг: убрать поле `text` после обновления диагностики/паритета.
- Вынести разные payload (LiteralData, CallData, etc.) в сегрегированные вектора (пока не начато).
- Добавить метрики: `interner.symbol_count`, `interner.total_bytes`.

### Phase 5: Fingerprints & Incremental
- Fingerprint pass (DFS) + корневой hash.
- Dirty range → CST diff → пересборка изменённых веток → сравнение fingerprint для раннего выхода.

### Phase 6: LSP Интеграция
- DocumentCache { ast_root_id, version, fingerprint, side-tables snapshot }.
- Быстрый completion контекст через восхождение по NodeId.

### Phase 7: Расширения
- Trivia capture (комментарии, doc-комменты) + привязка к ближайшему узлу.
- JSON экспорт AST (schema_version).
- Rule Engine адаптация под NodeId (паттерн-матчер поверх AstKind последовательностей).

### Phase 8: Форматтер и Рефакторинг (опционально)
- Rewrite API (builder pattern) с сохранением trivia.
- Batch edits: план изменений → применение к тексту через spans.

## Side-Table Структуры (черновик)
```rust
struct AstContext {
    arena: Arena<AstNode>,
    symbols: Vec<Option<SymbolId>>,
    types: Vec<Option<TypeId>>,
    method_res: Vec<Option<MethodId>>,
    prop_res: Vec<Option<PropertyId>>,
    diagnostics: Vec<Vec<Diagnostic>>, // per file or flattened
    fingerprints: Vec<u64>,
}
```

## Диагностики
- Diagnostic: { file: FileId, span: PackedSpan, code: u16, severity: u8, msg: String }
- Стабильность через FileId + PackedSpan.

## Метрики (минимум)
- ast.nodes_total
- ast.bytes_arena
- interner.symbol_count
- sem.types_assigned
- incr.reused_subtrees
- time.parse_ms / ast_ms / sem_ms
 - interner.total_bytes (Phase 4 добавление)

## Риски и Смягчение
Риск | Митигация
-----|---------
Срыв совместимости | Фазовый rollout + feature flags
Избыточная сложность | Документировать публичный API AST Context
Регресс производительности | Microbench до/после (criterion) + метрики
Сложность incremental | Сначала стабильный полно rebuild, потом dirty diff

## Критерии Готовности Фаз
Фаза | Done When
-----|-----------
0 | FileId + PackedSpan + метрики интегрированы
1 | Семантика компилируется на новом AST без bridge
2 | ErrorNode создаётся, Visitor покрывает ≥80% узлов
3 | Type & Symbol side-tables заменили in-node хранение
4 | >70% идентификаторов интернированы, память на AST ↓
5 | Повторный анализ без изменений <30% исходного времени
6 | LSP completion использует NodeId путь, latency снижен
7 | Комментарии доступны API, JSON экспорт стабилен
8 | Прототип форматтера корректно воспроизводит код без потерь

## Быстрый Start задач (labels пример)
1. feat(ast-core): add FileId + PackedSpan
2. feat(ast-core): implement Arena + NodeId + builder
3. feat(ast-core): add ErrorNode & recovery
4. feat(ast-api): visitor + traversal helpers
5. feat(sem-core): side-tables (types/symbols)
6. perf(ast): string interner integration
7. perf(incremental): node fingerprint hashing
8. feat(lsp): document cache with fingerprints
9. feat(ast-extra): trivia capture
10. feat(export): JSON AST schema v1

## Summary
План минимизирует риск: сперва фундамент (идентификаторы, арена, ошибки), затем семантическая декомпозиция, затем инкрементальность и удобства (trivia, экспорт, рефакторинг). Каждый этап измерим, поддерживает раннюю обратную связь и не блокирует текущий функционал.
