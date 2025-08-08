//! Модуль для отображения прогресса выполнения

use colored::Colorize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Стиль отображения прогресса
#[derive(Debug, Clone, Copy)]
pub enum ProgressStyle {
    /// Простой счетчик
    Counter,
    /// Процентный индикатор
    Percentage,
    /// Прогресс-бар
    Bar,
    /// Спиннер
    Spinner,
}

/// Репортер прогресса
pub struct ProgressReporter {
    total: usize,
    current: Arc<AtomicUsize>,
    style: ProgressStyle,
    start_time: Instant,
    last_update: Instant,
    update_interval: Duration,
    message: String,
}

impl ProgressReporter {
    /// Создает новый репортер прогресса
    pub fn new(total: usize, message: impl Into<String>) -> Self {
        Self {
            total,
            current: Arc::new(AtomicUsize::new(0)),
            style: ProgressStyle::Percentage,
            start_time: Instant::now(),
            last_update: Instant::now(),
            update_interval: Duration::from_millis(100),
            message: message.into(),
        }
    }

    /// Устанавливает стиль отображения
    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    /// Устанавливает интервал обновления
    pub fn with_update_interval(mut self, interval: Duration) -> Self {
        self.update_interval = interval;
        self
    }

    /// Увеличивает счетчик на 1
    pub fn inc(&self) {
        self.add(1);
    }

    /// Увеличивает счетчик на указанное значение
    pub fn add(&self, delta: usize) {
        let new_value = self.current.fetch_add(delta, Ordering::Relaxed) + delta;

        // Проверяем, нужно ли обновить отображение
        if self.should_update() {
            self.display(new_value);
        }
    }

    /// Устанавливает текущее значение
    pub fn set(&self, value: usize) {
        self.current.store(value, Ordering::Relaxed);

        if self.should_update() {
            self.display(value);
        }
    }

    /// Обновляет прогресс (алиас для set)
    pub fn update(&self, value: usize) {
        self.set(value);
    }

    /// Завершает прогресс
    pub fn finish(&self) {
        let current = self.current.load(Ordering::Relaxed);
        self.display_final(current);
    }

    /// Завершает с сообщением
    pub fn finish_with_message(&self, message: &str) {
        let current = self.current.load(Ordering::Relaxed);
        let elapsed = self.start_time.elapsed();

        println!(
            "\r{} {} ({}/{}) in {:.2?}",
            "✅".green(),
            message.green(),
            current,
            self.total,
            elapsed
        );
    }

    /// Проверяет, нужно ли обновить отображение
    fn should_update(&self) -> bool {
        self.last_update.elapsed() >= self.update_interval
    }

    /// Отображает текущий прогресс
    fn display(&self, current: usize) {
        match self.style {
            ProgressStyle::Counter => {
                print!("\r{}: {}/{}", self.message, current, self.total);
            }
            ProgressStyle::Percentage => {
                let percentage = if self.total > 0 {
                    (current as f64 / self.total as f64 * 100.0) as u32
                } else {
                    0
                };
                print!(
                    "\r{}: {}% ({}/{})",
                    self.message, percentage, current, self.total
                );
            }
            ProgressStyle::Bar => {
                self.display_bar(current);
            }
            ProgressStyle::Spinner => {
                self.display_spinner(current);
            }
        }

        // Сбрасываем буфер для немедленного отображения
        use std::io::{self, Write};
        let _ = io::stdout().flush();
    }

    /// Отображает прогресс-бар
    fn display_bar(&self, current: usize) {
        const BAR_WIDTH: usize = 40;

        let progress = if self.total > 0 {
            current as f64 / self.total as f64
        } else {
            0.0
        };

        let filled = (progress * BAR_WIDTH as f64) as usize;
        let empty = BAR_WIDTH - filled;

        print!(
            "\r{}: [{}{}] {}/{}",
            self.message,
            "█".repeat(filled).green(),
            "░".repeat(empty).dimmed(),
            current,
            self.total
        );
    }

    /// Отображает спиннер
    fn display_spinner(&self, current: usize) {
        const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

        let frame_index = (current / 10) % SPINNER_FRAMES.len();
        let frame = SPINNER_FRAMES[frame_index];

        print!(
            "\r{} {}: {} items processed",
            frame.cyan(),
            self.message,
            current
        );
    }

    /// Финальное отображение
    fn display_final(&self, current: usize) {
        let elapsed = self.start_time.elapsed();
        let rate = if elapsed.as_secs() > 0 {
            current as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        println!(
            "\r{}: {} items in {:.2?} ({:.1} items/sec)",
            self.message.green(),
            current,
            elapsed,
            rate
        );
    }
}

/// Простой счетчик без отображения прогресса
pub struct SilentCounter {
    count: AtomicUsize,
}

impl SilentCounter {
    pub fn new() -> Self {
        Self {
            count: AtomicUsize::new(0),
        }
    }

    pub fn inc(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn add(&self, delta: usize) {
        self.count.fetch_add(delta, Ordering::Relaxed);
    }

    pub fn get(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
}

impl Default for SilentCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Макрос для условного вывода прогресса
#[macro_export]
macro_rules! progress {
    ($reporter:expr, $($arg:tt)*) => {
        if let Some(reporter) = $reporter {
            reporter.inc();
        }
    };
}
