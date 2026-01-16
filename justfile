set shell := ["bash", "-c"]
#set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
# Установка зависимостей
install-modules:
    cd frontend && /home/phobos/.local/share/pnpm/pnpm install

# Запуск фронтенда
frontend-dev:
    cd frontend && /home/phobos/.local/share/pnpm/pnpm run serve

# Проверка фронтенда
frontend-check:
    cd frontend && /home/phobos/.local/share/pnpm/pnpm run tsc

# Запуск сервера
backend-dev:
    cargo run -p employee-accounting-server

# Запуск всего
dev:
    concurrently "just backend-dev" "just frontend-dev"

# Сборка
build:
    cd frontend && /home/phobos/.local/share/pnpm/pnpm run build
    cargo build --release -p employee-accounting-server
    
    # Сборка
build-win:
    cd frontend && /home/phobos/.local/share/pnpm/pnpm run build
    cargo build --target x86_64-pc-windows-gnu --release -p employee-accounting-server --verbose