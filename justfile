set shell := ["bash", "-c"]
#set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
# Установка зависимостей
install-modules:
    cd frontend && /home/phobos/.local/share/pnpm/pnpm install

# Запуск фронтенда
frontend-dev:
    cd frontend && /home/phobos/.local/share/pnpm/pnpm run serve

frontend-build:
    cd frontend && pnpm run build

    # Запуск фронтенда
gateway-ui-dev:
    cd gateway-ui && pnpm run dev
    # Сборка gateway ui
gateway-ui-build:
    cd gateway-ui && pnpm run build
# Запуск сервера
gateway-back-dev:
    cargo run -p gateway


# Проверка фронтенда
frontend-check:
    cd frontend && /home/phobos/.local/share/pnpm/pnpm run tsc

# Запуск сервера
backend-dev:
    cargo run -p api-service

# Запуск rag service
dev:
    concurrently "just backend-dev" "just frontend-dev"

# Запуск gateway
gateway-dev:
    concurrently "just gateway-ui-build" "just gateway-back-dev"

# Сборка
build:
    cd frontend && /home/phobos/.local/share/pnpm/pnpm run build
    cargo build --release
    
    # Сборка
build-win:
    cd frontend && /home/phobos/.local/share/pnpm/pnpm run build
    cargo build --target x86_64-pc-windows-gnu --release --verbose