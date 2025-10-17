# Примеры использования inih

Этот каталог содержит примеры использования библиотеки inih на Rust.

## Примеры

### basic_example.rs
Базовый пример использования высокоуровневого API `IniReader` для простого чтения значений из INI файла.

```bash
cargo run --example basic_example
```

### callback_example.rs
Пример использования низкоуровневого callback API с трейтом `IniHandler` для кастомной логики парсинга.

```bash
cargo run --example callback_example
```

### file_example.rs
Пример чтения INI данных из реального файла с демонстрацией различных типов конфигурации.

```bash
cargo run --example file_example
```

### advanced_example.rs
Продвинутый пример, демонстрирующий использование кастомных опций парсинга и обработку различных ошибок.

```bash
cargo run --example advanced_example
```

## Запуск всех примеров

```bash
cargo run --example basic_example
cargo run --example callback_example
cargo run --example file_example
cargo run --example advanced_example
```
