# image_ffi_project
CLI-приложение для обработки изображения с поддержкой динамически подключаемых плагинов через ffi. 

## !!! Название плагинов !!!
При компилировании библеотеки на линукс, компиятор добавляет в название lib. В итоге плагины собираются с названием:  `libmirror_plugin` и `libblur_plugin`.

## Сборка проекта: 
```bash
# Сборка всех артифактов:
cargo build

# Сборка основного приложения
cargo build --bin image_processor

# Сборка плагина для блюра
cargo build --bin blur_plugin

# Сборка плагина для отзеркаливания
cargo build --bin mirror_plugin

```

## Параметры для плагинов:
### Blur_plugin
```json
{
    "radius": 14.0,
    "iterations": 7
}

```

### mirror_plugin
```json
{
    "horizontal": true,
    "vertical": true
}

```
## Пример запуска программы:
```bash
.target/debug/image_processor [OPTIONS] --input <INPUT> --output <OUTPUT> --plugin <PLUGIN> --params <PARAMS>
```
