# inih (INI Not Invented Here) - Rust Edition

[![Tests](https://github.com/benhoyt/inih/actions/workflows/tests.yml/badge.svg)](https://github.com/benhoyt/inih/actions/workflows/tests.yml)

**inih (INI Not Invented Here)** - это простой парсер файлов .INI, написанный на Rust. Это порт популярной C библиотеки inih, разработанный для того, чтобы быть _маленьким и простым_, что делает его подходящим для встраиваемых систем. Он также более или менее совместим со стилем .INI файлов Python [ConfigParser](http://docs.python.org/library/configparser.html), включая синтаксис многострочных записей в стиле RFC 822 и записи `name: value`.

## Особенности

- Парсинг INI файлов с секциями, парами name=value и комментариями
- Поддержка многострочных записей (как в ConfigParser Python)
- Поддержка UTF-8 BOM
- Встроенные и начальные комментарии
- Настраиваемые опции парсинга
- Как callback-based, так и reader-based API
- Эффективное использование памяти (без ненужных выделений)
- Полная совместимость с оригинальной C библиотекой

## Быстрый старт

### Высокоуровневый API

```rust
use inih::IniReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reader = IniReader::from_file("config.ini")?;
    
    let version = reader.get_integer("protocol", "version", -1);
    let name = reader.get_string("user", "name", "UNKNOWN");
    let email = reader.get_string("user", "email", "UNKNOWN");
    
    println!("Config: version={}, name={}, email={}", version, name, email);
    Ok(())
}
```

### Низкоуровневый API

```rust
use inih::{ini_parse_string, IniHandler};

struct Config {
    version: i32,
    name: String,
}

impl IniHandler for Config {
    fn handle(&mut self, section: &str, name: &str, value: &str) -> Result<(), String> {
        match (section, name) {
            ("protocol", "version") => {
                self.version = value.parse().map_err(|_| "Invalid version".to_string())?;
            }
            ("user", "name") => {
                self.name = value.to_string();
            }
            _ => return Err(format!("Unknown key: {}.{}", section, name)),
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config { version: 0, name: String::new() };
    ini_parse_string("[protocol]\nversion=6\n[user]\nname=Bob", &mut config)?;
    println!("Version: {}, Name: {}", config.version, config.name);
    Ok(())
}
```

## Установка

Добавьте в ваш `Cargo.toml`:

```toml
[dependencies]
inih = "0.1.0"
```

## Использование

### Чтение из файла

```rust
use inih::IniReader;

let reader = IniReader::from_file("config.ini")?;
let value = reader.get_string("section", "key", "default");
```

### Чтение из строки

```rust
use inih::IniReader;

let data = r#"
[section]
key = value
"#;
let reader = IniReader::from_string(data)?;
let value = reader.get_string("section", "key", "default");
```

### Чтение из потока

```rust
use inih::IniReader;
use std::fs::File;

let file = File::open("config.ini")?;
let reader = IniReader::from_reader(file)?;
```

### Типы данных

```rust
let reader = IniReader::from_string(data)?;

// Строки
let name = reader.get_string("user", "name", "UNKNOWN");
let email = reader.get_string("user", "email", "");

// Целые числа
let port = reader.get_integer("server", "port", 8080);
let big_number = reader.get_integer64("data", "trillion", 0);

// Беззнаковые целые числа
let max_connections = reader.get_unsigned("server", "max_connections", 100);

// Числа с плавающей точкой
let pi = reader.get_real("math", "pi", 3.14159);

// Логические значения
let debug = reader.get_boolean("server", "debug", false);
// Поддерживаемые значения: true/false, yes/no, on/off, 1/0
```

### Работа с секциями

```rust
let reader = IniReader::from_string(data)?;

// Получить все секции
let sections = reader.sections();
for section in sections {
    println!("Section: {}", section);
}

// Получить все ключи в секции
let keys = reader.keys("user");
for key in keys {
    println!("Key: {}", key);
}

// Проверить существование
if reader.has_section("user") {
    println!("User section exists");
}

if reader.has_value("user", "name") {
    println!("Name key exists in user section");
}
```

### Настройка парсинга

```rust
use inih::{ini_parse_string_with_options, ParseOptions};

let mut options = ParseOptions::default();
options.allow_multiline = true;
options.allow_inline_comments = true;
options.inline_comment_prefixes = ";#".to_string();
options.start_comment_prefixes = ";#".to_string();
options.allow_no_value = true;
options.stop_on_first_error = false;
options.max_line = 1000;

let mut handler = MyHandler::new();
ini_parse_string_with_options(data, &mut handler, &options)?;
```

## Опции компиляции

Вы можете контролировать различные аспекты inih с помощью опций в `ParseOptions`:

### Опции синтаксиса

- **Многострочные записи:** По умолчанию inih поддерживает многострочные записи в стиле ConfigParser Python. Установите `allow_multiline = false` для отключения.
- **UTF-8 BOM:** По умолчанию inih позволяет последовательность UTF-8 BOM (0xEF 0xBB 0xBF) в начале INI файлов. Установите `allow_bom = false` для отключения.
- **Встроенные комментарии:** По умолчанию inih позволяет встроенные комментарии с символом `;`. Установите `allow_inline_comments = false` для отключения.
- **Комментарии в начале строки:** По умолчанию inih позволяет как `;`, так и `#` для начала комментария в начале строки. Настройте `start_comment_prefixes`.
- **Разрешить отсутствие значения:** По умолчанию inih обрабатывает имя без значения (без `=` или `:` в строке) как ошибку. Установите `allow_no_value = true` для разрешения.

### Опции парсинга

- **Остановка на первой ошибке:** По умолчанию inih продолжает парсинг остальной части файла после ошибки. Установите `stop_on_first_error = true` для остановки на первой ошибке.
- **Вызов обработчика на новой секции:** По умолчанию inih вызывает обработчик только для каждой пары `name=value`. Установите `call_handler_on_new_section = true` для вызова при обнаружении новой секции.

### Опции памяти

- **Максимальная длина строки:** По умолчанию максимальная длина строки составляет 200 байт. Настройте `max_line` для изменения.

## Примеры

Смотрите папку `examples/` для дополнительных примеров:

- `basic_example.rs` - Базовое использование высокоуровневого API
- `callback_example.rs` - Использование низкоуровневого callback API
- `file_example.rs` - Чтение из файла
- `advanced_example.rs` - Продвинутые возможности и настройки

## Тестирование

```bash
cargo test
```

## Лицензия

BSD-3-Clause (см. файл LICENSE.txt)

## Связанные ссылки

- [Оригинальная C библиотека inih](https://github.com/benhoyt/inih)
- [Conan пакет для inih](https://github.com/conan-io/conan-center-index/tree/master/recipes/inih) (Conan - это менеджер пакетов C/C++)

## Различия с ConfigParser

Некоторые различия между inih и модулем [ConfigParser](http://docs.python.org/library/configparser.html) стандартной библиотеки Python:

* Пары INI name=value, указанные выше любых заголовков секций, обрабатываются как действительные элементы без секции (имя секции - пустая строка). В ConfigParser отсутствие секции является ошибкой.
* Продолжения строк обрабатываются с ведущими пробелами на продолженных строках (как в ConfigParser). Однако вместо объединения продолженных строк вместе, они обрабатываются как отдельные значения для того же ключа (в отличие от ConfigParser).