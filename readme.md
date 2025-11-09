# Проектная работа модуля 1. Чтение, парсинг и анализ данных в Rust

Задание реализовано для опции 2 с форматами YPBankCsv, YPBankText, YPBankBin

Присутствует 2 запускаемых файла - converter и comparer

## Converter

Команда для запуска 
```
cargo run --bin converter -- --input <PATH_TO_FILE> --input-format <FORMAT> --output-format <FORMAT> > <OUTPUT_FILE>
```

## Comparer

Команда для запуска 
```
cargo run --bin comparer -- --file1 <PATH_TO_FILE> --format1 <FORMAT> --file2 <PATH_TO_FILE> --format2 <FORMAT>
```

### Доступные значения FORMAT

`binary`, `text`, `csv`
