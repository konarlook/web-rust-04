# Blog

Блог-платформа на Rust: регистрация и вход по JWT, публикация постов, редактирование и
удаление — только автором. Просмотр списка и отдельных постов доступен без входа.

Один и тот же бэкенд отдаёт два API — REST через `actix-web` и gRPC через
`tonic`, — используя общий слой бизнес-логики. Клиентская библиотека умеет работать с
обоими транспортами за единым интерфейсом; поверх неё построен CLI. Отдельно есть
фронтенд на WebAssembly, который ходит в HTTP API напрямую.

## Архитектура

Cargo workspace из пяти крейтов:

| Крейт         | Назначение                                                                                                                                                              |
|---------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `blog-proto`  | Единственный экземпляр protobuf-схемы. Генерирует типы, gRPC-сервер и клиент через `tonic-prost-build`. Подключается сервером и клиентом, чтобы схема не дублировалась. |
| `blog-server` | Бэкенд: HTTP API (порт 8080), gRPC API (порт 50051), PostgreSQL через `sqlx`, JWT, Argon2.                                                                              |
| `blog-client` | Библиотека доступа к API. Поддерживает `Transport::Http` (reqwest) и `Transport::Grpc` (tonic) за одинаковым набором методов.                                           |
| `blog-cli`    | CLI поверх `blog-client`. Хранит токен в `.blog_token`.                                                                                                                 |
| `blog-wasm`   | Фронтенд на WebAssembly. Ходит в HTTP API напрямую через `gloo-net`.                                                                                                    |

### Слои `blog-server`

Чистая архитектура: зависимости направлены внутрь, домен не знает ни о фреймворках, ни о
базе.

```
src/
├── domain/         модели, доменные ошибки и порты (трейты репозиториев,
│                   PasswordHasher, TokenIssuer)
├── application/    бизнес-логика: AuthService, BlogService
├── data/           реализация репозиториев на sqlx
├── infra/          БД, JWT, хеширование паролей, конфигурация, логирование
└── presentation/   http/ (хендлеры, JWT-middleware, маршруты, маппинг ошибок)
                    grpc/ (реализация сервиса, конвертация моделей)
```

HTTP- и gRPC-слои вызывают одни и те же `AuthService` и `BlogService`, поэтому
бизнес-правила — валидация, проверка авторства, лимиты пагинации — работают одинаково в
обоих API.

```bash
cargo install sqlx-cli --no-default-features --features postgres
cargo install wasm-pack
rustup target add wasm32-unknown-unknown
```

## Запуск

### 1. Переменные окружения

```bash
cp .env.example .env
```

Затем сгенерируйте настоящий ключ подписи и впишите его в `.env` — сервер откажется
стартовать с секретом короче 32 байт:

```bash
openssl rand -base64 48
```

### 2. База данных

```bash
docker compose up -d
```

Поднимет PostgreSQL 17 на `localhost:5432` с базой `blog`
и пользователем `app/app` (соответствует `DATABASE_URL` из `.env.example`).

### 3. Миграции — обязательно до сборки

```bash
cd blog-server && sqlx migrate run && cd ..
```

> **Важно.** Проект использует макрос `sqlx::query_as!`, который проверяет
> SQL-запросы **во время компиляции**, обращаясь к живой базе. Если запустить
> `cargo build` до применения миграций, сборка упадёт с
> `relation "users" does not exist`. Порядок именно такой: поднять базу →
> применить миграции → собирать.

Миграция создаёт таблицы `users` и `posts` с внешним ключом
`posts.author_id → users.id` и индексами.

### 4. Сборка

```bash
cargo build --workspace
```

## Запуск сервера

```bash
cargo run --bin blog-server
```

Поднимутся оба сервера одновременно:

- HTTP API — `http://localhost:8080`
- gRPC API — `localhost:50051`

Остановка — `Ctrl+C`, оба сервера завершаются вместе.

## HTTP API

| Метод  | Путь                 | Аутентификация | Ответ                                 |
|--------|----------------------|----------------|---------------------------------------|
| POST   | `/api/auth/register` | нет            | 201 + `{token, user}`                 |
| POST   | `/api/auth/login`    | нет            | 200 + `{token, user}`                 |
| GET    | `/api/posts`         | нет            | 200 + `{posts, total, limit, offset}` |
| GET    | `/api/posts/{id}`    | нет            | 200 + пост, 404 если нет              |
| POST   | `/api/posts`         | Bearer         | 201 + пост                            |
| PUT    | `/api/posts/{id}`    | Bearer         | 200 + пост, 403 если чужой            |
| DELETE | `/api/posts/{id}`    | Bearer         | 204, 403 если чужой                   |

`GET /api/posts` принимает `limit` (по умолчанию 10, максимум 100)
и `offset` (по умолчанию 0).

### Пример сценария через curl

```bash
# Регистрация — вернёт токен
curl -s -X POST localhost:8080/api/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"username":"ivan","email":"ivan@example.com","password":"secret123"}'

# Вход
TOKEN=$(curl -s -X POST localhost:8080/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username":"ivan","password":"secret123"}' | jq -r .token)

# Создание поста
curl -s -X POST localhost:8080/api/posts \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"title":"Первый пост","content":"Содержание"}'

# Список постов — публично
curl -s 'localhost:8080/api/posts?limit=10&offset=0'

# Один пост — публично
curl -s localhost:8080/api/posts/1

# Частичное обновление: content не трогаем
curl -s -X PUT localhost:8080/api/posts/1 \
  -H "Authorization: Bearer $TOKEN" \
  -H 'Content-Type: application/json' \
  -d '{"title":"Новый заголовок"}'

# Удаление
curl -si -X DELETE localhost:8080/api/posts/1 \
  -H "Authorization: Bearer $TOKEN"

# Без токена — 401
curl -si -X POST localhost:8080/api/posts \
  -H 'Content-Type: application/json' -d '{"title":"t","content":"c"}'
```

## gRPC API

Схема: `blog-proto/proto/blog/blog.proto`, сервис `blog.BlogService`.

| Метод                                    | Аутентификация                           |
|------------------------------------------|------------------------------------------|
| `Register`, `Login`                      | нет                                      |
| `GetPost`, `ListPosts`                   | нет                                      |
| `CreatePost`, `UpdatePost`, `DeletePost` | metadata `authorization: Bearer <token>` |

Ошибки отдаются штатными кодами: `NOT_FOUND`, `ALREADY_EXISTS`,
`UNAUTHENTICATED`, `PERMISSION_DENIED`, `INVALID_ARGUMENT`, `INTERNAL`.

### Пример через grpcurl

Серверная рефлексия не включена, поэтому схема указывается явно:

```bash
PROTO="-import-path blog-proto/proto/blog -proto blog.proto"

# Регистрация
grpcurl -plaintext $PROTO \
  -d '{"username":"petr","email":"petr@example.com","password":"secret123"}' \
  localhost:50051 blog.BlogService/Register

# Публичный список
grpcurl -plaintext $PROTO -d '{}' localhost:50051 blog.BlogService/ListPosts

# Создание поста с токеном
grpcurl -plaintext $PROTO \
  -H "authorization: Bearer $TOKEN" \
  -d '{"title":"Из gRPC","content":"Содержание"}' \
  localhost:50051 blog.BlogService/CreatePost

# Без токена — UNAUTHENTICATED
grpcurl -plaintext $PROTO \
  -d '{"title":"t","content":"c"}' \
  localhost:50051 blog.BlogService/CreatePost
```

## CLI-клиент

По умолчанию работает по HTTP (`http://localhost:8080`). Флаг `--grpc`
переключает на gRPC (`http://localhost:50051`), `--server` задаёт адрес вручную. Оба
флага глобальные — их можно писать в любом месте команды.

Токен после `register`/`login` сохраняется в файл `.blog_token`
(на Unix — с правами `0600`) и подхватывается при следующих запусках.

```bash
cargo build --bin blog-cli

# Регистрация и вход
cargo run -p blog-cli -- register --username ivan --email ivan@example.com --password secret123
cargo run -p blog-cli -- login --username ivan --password secret123

# Посты
cargo run -p blog-cli -- create --title "Мой первый пост" --content "Содержание"
cargo run -p blog-cli -- list --limit 20 --offset 0
cargo run -p blog-cli -- get --id 1
cargo run -p blog-cli -- update --id 1 --title "Обновлённый заголовок"
cargo run -p blog-cli -- delete --id 1

# То же самое через gRPC
cargo run -p blog-cli -- create --title "Через gRPC" --content "Содержание" --grpc
cargo run -p blog-cli -- list --grpc

# Другой адрес сервера
cargo run -p blog-cli -- list --server http://192.168.1.10:8080
```

Команда `update` меняет только переданные поля: `--title` без `--content`
оставит содержимое нетронутым.

Полный список команд: `cargo run -p blog-cli -- --help`.

## WASM-фронтенд

### Сборка

```bash
cd blog-wasm
wasm-pack build --target web
```

Результат — каталог `blog-wasm/pkg/` с `.wasm` и JS-обёрткой.

### Запуск

```bash
cd blog-wasm
python3 -m http.server 8000
```

Открыть `http://localhost:8000`.

Сервер блога должен быть запущен, а в `.env` должен быть `ALLOWED_ORIGINS=*`
либо `http://localhost:8000` — иначе браузер заблокирует запросы по CORS.

### Что умеет интерфейс

- Список постов виден без входа
- Формы регистрации и входа с проверкой пустых полей
- JWT сохраняется в `localStorage` и восстанавливается при перезагрузке
- Форма создания поста — только после входа
- Кнопки «Редактировать» и «Удалить» — только у своих постов
- Статус аутентификации в шапке и кнопка выхода
- Ошибки сервера показываются на странице; при истёкшем токене (401)
  интерфейс автоматически возвращается в состояние «не выполнен вход»

## Полный сценарий проверки

```bash
cp .env.example .env                                   # + вписать JWT_SECRET
docker compose up -d
cd blog-server && sqlx migrate run && cd ..
cargo build --workspace

cargo run --bin blog-server                            # терминал 1

cd blog-wasm && wasm-pack build --target web           # терминал 2
python3 -m http.server 8000

cargo run -p blog-cli -- register --username ivan \
  --email ivan@example.com --password secret123        # терминал 3
cargo run -p blog-cli -- create --title "Пост" --content "Текст"
cargo run -p blog-cli -- list
```

Затем открыть `http://localhost:8000` — созданный через CLI пост должен отображаться в
браузере.
