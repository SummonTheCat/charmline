import os
import sys
from datetime import datetime

def find_files(root_path, extensions):
    matches = []
    extensions = [ext.lower().lstrip('.') for ext in extensions]

    for dirpath, _, filenames in os.walk(root_path):
        for filename in filenames:
            ext = filename.split('.')[-1].lower()
            if ext in extensions:
                matches.append(os.path.join(dirpath, filename))

    return matches


def clean_lines(text):
    # Remove extra blank lines and strip spaces
    lines = [line.rstrip() for line in text.splitlines()]
    cleaned = [line for line in lines if line.strip() != ""]
    return "\n".join(cleaned)


def collect_files(files, output_dir):
    os.makedirs(output_dir, exist_ok=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output_path = os.path.join(output_dir, f"collect_{timestamp}.clt")

    with open(output_path, "w", encoding="utf-8") as out:
        for path in files:
            try:
                with open(path, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception as e:
                print(f"Skipping {path} (Error: {e})")
                continue

            cleaned = clean_lines(content)
            out.write(f"\n{'='*80}\n# FILE: {path}\n{'='*80}\n")
            if cleaned.strip():
                out.write(f"\n{cleaned}\n")

    return output_path


def main():
    if len(sys.argv) < 4:
        print("Usage: python collect_files.py <root_path> <extensions> <output_dir>")
        print("Example: python collect_files.py ./ 'txt,log,json' ./output")
        sys.exit(1)

    root_path = sys.argv[1]
    extensions = sys.argv[2].split(',')
    output_dir = sys.argv[3]

    if not os.path.exists(root_path):
        print(f"Error: Path not found: {root_path}")
        sys.exit(1)

    files = find_files(root_path, extensions)
    if not files:
        print(f"No files found with extensions: {', '.join(extensions)}")
        sys.exit(0)

    print(f"Found {len(files)} file(s). Collecting...")
    output_file = collect_files(files, output_dir)
    print(f"\n✅ Combined file created at:\n{output_file}")


if __name__ == "__main__":
    main()
