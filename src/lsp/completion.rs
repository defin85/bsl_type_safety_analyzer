use tower_lsp::{
    lsp_types::{CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse},
    jsonrpc::Result,
};

#[allow(dead_code)]
pub struct CompletionProvider;

#[allow(dead_code)]
impl CompletionProvider {
    pub fn new() -> Self {
        Self
    }

    pub fn provide_completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // Базовые ключевые слова BSL
        let keywords = vec![
            "Процедура", "Функция", "КонецПроцедуры", "КонецФункции",
            "Если", "Тогда", "Иначе", "КонецЕсли",
            "Для", "Каждого", "Из", "По", "Цикл", "КонецЦикла",
            "Пока", "КонецЦикла",
            "Попытка", "Исключение", "КонецПопытки",
            "Истина", "Ложь", "Неопределено",
        ];

        let completion_items: Vec<CompletionItem> = keywords
            .into_iter()
            .map(|keyword| CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("BSL keyword".to_string()),
                ..Default::default()
            })
            .collect();

        Ok(Some(CompletionResponse::Array(completion_items)))
    }
}
