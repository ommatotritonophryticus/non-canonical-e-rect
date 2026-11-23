#!/usr/bin/env python3
"""
Сборщик исходников проекта в один файл.
Показывает относительный путь каждого файла.
"""

import os
import argparse
from pathlib import Path

# Расширения, которые обычно интересуют (можно менять)
DEFAULT_EXTENSIONS = {
    # Rust
    ".rs", ".toml",
    # Общие
    ".py", ".js", ".ts", ".tsx", ".jsx", ".go", ".java", ".kt",
    ".c", ".cpp", ".h", ".hpp", ".cs", ".rb", ".php", ".swift",
    ".md", ".txt", ".json", ".yaml", ".yml", ".toml", ".ini",
    ".sh", ".bash", ".zsh", ".fish",
    ".html", ".css", ".scss", ".sql",
}

# Папки, которые почти всегда нужно пропускать
DEFAULT_IGNORE_DIRS = {
    ".git", ".svn", ".hg",
    "target", "build", "dist", "out",
    "node_modules", "vendor", "__pycache__",
    ".idea", ".vscode", ".vs",
    "venv", ".venv", "env",
    "coverage", "tmp", "temp",
}

def should_ignore(path: Path, ignore_dirs: set) -> bool:
    """Проверяет, нужно ли пропускать путь."""
    for part in path.parts:
        if part in ignore_dirs:
            return True
    return False

def collect_files(
    root: Path,
    extensions: set,
    ignore_dirs: set,
    max_size_kb: int = 500,
) -> list[tuple[Path, str]]:
    """Собирает список (относительный_путь, содержимое)."""
    results = []
    
    for dirpath, dirnames, filenames in os.walk(root):
        # Фильтруем директории на месте (чтобы os.walk не заходил внутрь)
        dirnames[:] = [
            d for d in dirnames
            if d not in ignore_dirs and not d.startswith(".")
        ]
        
        current = Path(dirpath)
        
        for name in filenames:
            filepath = current / name
            
            # Пропускаем скрытые файлы и слишком большие
            if name.startswith("."):
                continue
            if filepath.suffix.lower() not in extensions:
                continue
            try:
                size_kb = filepath.stat().st_size / 1024
                if size_kb > max_size_kb:
                    print(f"⚠ Пропущен (слишком большой): {filepath.relative_to(root)} ({size_kb:.1f} KB)")
                    continue
            except OSError:
                continue
            
            try:
                content = filepath.read_text(encoding="utf-8", errors="replace")
                rel_path = filepath.relative_to(root)
                results.append((rel_path, content))
            except Exception as e:
                print(f"⚠ Не удалось прочитать {filepath}: {e}")
    
    # Сортируем для предсказуемого порядка
    results.sort(key=lambda x: str(x[0]).lower())
    return results

def main():
    parser = argparse.ArgumentParser(
        description="Собирает исходники проекта в один текстовый файл"
    )
    parser.add_argument(
        "directory",
        nargs="?",
        default=".",
        help="Корневая папка проекта (по умолчанию текущая)"
    )
    parser.add_argument(
        "-o", "--output",
        default="project_dump.txt",
        help="Имя выходного файла (по умолчанию project_dump.txt)"
    )
    parser.add_argument(
        "-e", "--extensions",
        help="Список расширений через запятую (например: rs,toml,md)"
    )
    parser.add_argument(
        "--max-size",
        type=int,
        default=500,
        help="Максимальный размер файла в KB (по умолчанию 500)"
    )
    parser.add_argument(
        "--include-hidden",
        action="store_true",
        help="Включать скрытые файлы и папки"
    )
    
    args = parser.parse_args()
    
    root = Path(args.directory).resolve()
    if not root.is_dir():
        print(f"Ошибка: {root} не является директорией")
        return
    
    extensions = DEFAULT_EXTENSIONS
    if args.extensions:
        extensions = {f".{ext.strip().lstrip('.')}" for ext in args.extensions.split(",")}
    
    ignore_dirs = DEFAULT_IGNORE_DIRS.copy()
    if args.include_hidden:
        ignore_dirs = set()  # почти ничего не игнорируем
    
    print(f"Сканирую: {root}")
    files = collect_files(root, extensions, ignore_dirs, args.max_size)
    
    if not files:
        print("Не найдено подходящих файлов.")
        return
    
    output_path = Path(args.output)
    
    with output_path.open("w", encoding="utf-8") as out:
        out.write(f"# Project dump: {root.name}\n")
        out.write(f"# Generated from: {root}\n")
        out.write(f"# Total files: {len(files)}\n")
        out.write("=" * 80 + "\n\n")
        
        for rel_path, content in files:
            # Заголовок файла
            out.write(f"{'=' * 80}\n")
            out.write(f"FILE: {rel_path.as_posix()}\n")
            out.write(f"{'=' * 80}\n\n")
            out.write(content)
            if not content.endswith("\n"):
                out.write("\n")
            out.write("\n\n")
    
    print(f"\nГотово! Собрано {len(files)} файлов → {output_path.resolve()}")
    print(f"Размер итогового файла: {output_path.stat().st_size / 1024:.1f} KB")

if __name__ == "__main__":
    main()
