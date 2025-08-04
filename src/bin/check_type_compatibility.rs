/// Проверка совместимости BSL типов
/// 
/// Утилита для проверки совместимости типов в рамках системы типов BSL.
/// Использует UnifiedBslIndex для определения возможности присваивания одного типа другому.
/// 
/// Пример использования:
/// ```bash
/// cargo run --bin check_type_compatibility -- --from "Справочники.Номенклатура" --to "СправочникСсылка" --config "path/to/config"
/// ```

use anyhow::Result;
use clap::Parser;
use std::path::Path;
use bsl_analyzer::unified_index::UnifiedIndexBuilder;

#[derive(Parser)]
#[command(name = "check_type_compatibility")]
#[command(about = "Проверка совместимости BSL типов")]
#[command(long_about = "Утилита для проверки совместимости типов в системе типов BSL. \
Проверяет возможность присваивания значения одного типа переменной другого типа.")]
struct Args {
    /// Исходный тип (от какого типа преобразуем)
    #[arg(long, help = "Исходный тип для проверки")]
    from: String,

    /// Целевой тип (к какому типу преобразуем)
    #[arg(long, help = "Целевой тип для проверки")]
    to: String,

    /// Путь к конфигурации 1С
    #[arg(long, help = "Путь к конфигурации 1С")]
    config: String,

    /// Версия платформы 1С
    #[arg(long, default_value = "8.3.25", help = "Версия платформы 1С")]
    platform_version: String,

    /// Подробный вывод с объяснением совместимости
    #[arg(long, help = "Подробный анализ совместимости")]
    verbose: bool,

    /// Показать путь наследования
    #[arg(long, help = "Показать путь наследования между типами")]
    show_inheritance_path: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Инициализация логирования
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Проверка существования конфигурации
    if !Path::new(&args.config).exists() {
        anyhow::bail!("Configuration path does not exist: {}", args.config);
    }

    tracing::info!("Проверка совместимости типов: {} -> {}", args.from, args.to);

    // Загрузка индекса
    let mut builder = UnifiedIndexBuilder::new()?;
    let index = builder.build_index(&args.config, &args.platform_version)?;

    // Проверка совместимости
    let is_compatible = index.is_assignable(&args.from, &args.to);

    // Проверка существования типов
    let from_entity = index.find_entity(&args.from);
    let to_entity = index.find_entity(&args.to);

    // Вывод результатов
    if args.verbose {
        println!("🔍 Анализ совместимости типов");
        println!("================================");
        println!("Исходный тип: {}", args.from);
        
        match from_entity {
            Some(entity) => {
                println!("  ✓ Найден: {} ({:?})", entity.display_name, entity.entity_type);
            }
            None => {
                println!("  ❌ Не найден в индексе");
            }
        }

        println!("Целевой тип: {}", args.to);
        match to_entity {
            Some(entity) => {
                println!("  ✓ Найден: {} ({:?})", entity.display_name, entity.entity_type);
            }
            None => {
                println!("  ❌ Не найден в индексе");
            }
        }

        println!();
        println!("Результат совместимости:");
        if is_compatible {
            println!("  ✅ СОВМЕСТИМЫ");
            println!("  Тип '{}' может быть присвоен переменной типа '{}'", args.from, args.to);
        } else {
            println!("  ❌ НЕ СОВМЕСТИМЫ");
            println!("  Тип '{}' НЕ может быть присвоен переменной типа '{}'", args.from, args.to);
        }

        // Показать путь наследования если запрошено
        if args.show_inheritance_path && is_compatible && args.from != args.to {
            println!();
            println!("Путь совместимости:");
            if let (Some(from_entity), Some(to_entity)) = (from_entity, to_entity) {
                // Простая проверка через родительские типы
                if from_entity.constraints.parent_types.contains(&args.to) {
                    println!("  {} → наследует → {}", args.from, args.to);
                } else if from_entity.constraints.implements.contains(&args.to) {
                    println!("  {} → реализует → {}", args.from, args.to);
                } else if args.from == args.to {
                    println!("  {} ≡ {} (идентичные типы)", args.from, args.to);
                } else {
                    println!("  Совместимость через систему типов BSL");
                }
            }
        }

        // Дополнительная информация о типах
        if from_entity.is_some() || to_entity.is_some() {
            println!();
            println!("Дополнительная информация:");
        }

        if let Some(entity) = from_entity {
            if !entity.constraints.parent_types.is_empty() {
                println!("  {} наследует от: {}", args.from, entity.constraints.parent_types.join(", "));
            }
            if !entity.constraints.implements.is_empty() {
                println!("  {} реализует: {}", args.from, entity.constraints.implements.join(", "));
            }
        }

        if let Some(entity) = to_entity {
            if !entity.constraints.parent_types.is_empty() {
                println!("  {} наследует от: {}", args.to, entity.constraints.parent_types.join(", "));
            }
            if !entity.constraints.implements.is_empty() {
                println!("  {} реализует: {}", args.to, entity.constraints.implements.join(", "));
            }
        }

    } else {
        // Краткий вывод
        if is_compatible {
            println!("✅ СОВМЕСТИМЫ: {} -> {}", args.from, args.to);
        } else {
            println!("❌ НЕ СОВМЕСТИМЫ: {} -> {}", args.from, args.to);
        }
    }

    // Установка кода возврата для скриптов
    if !is_compatible {
        std::process::exit(1);
    }

    Ok(())
}