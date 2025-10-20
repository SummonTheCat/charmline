#!/usr/bin/env python3
import os
import shutil
import subprocess
import sys
import signal
from pathlib import Path

# --- ANSI colors ---
RESET = "\033[0m"
BOLD = "\033[1m"
DIM = "\033[2m"
GREEN = "\033[92m"
YELLOW = "\033[93m"
RED = "\033[91m"
CYAN = "\033[96m"
MAGENTA = "\033[95m"

def section(title: str):
    print(f"\n{BOLD}{MAGENTA}{'=' * 60}{RESET}")
    print(f"{BOLD}{CYAN}>>> {title}{RESET}")
    print(f"{BOLD}{MAGENTA}{'=' * 60}{RESET}")

def build_and_run(project_dir: str, output_dir: str):
    project_dir = Path(project_dir).resolve()
    output_dir = Path(output_dir).resolve()

    if not project_dir.exists():
        print(f"{RED}✗ Error: Project directory does not exist: {project_dir}{RESET}")
        sys.exit(1)

    if not (project_dir / "Cargo.toml").exists():
        print(f"{RED}✗ Error: No Cargo.toml found in project directory: {project_dir}{RESET}")
        sys.exit(1)

    output_dir.mkdir(parents=True, exist_ok=True)

    # Build section
    section("Building Charmline Project")
    print(f"{DIM}→ Using project directory: {project_dir}{RESET}")
    try:
        subprocess.run(["cargo", "build", "--release"], cwd=project_dir, check=True)
    except subprocess.CalledProcessError:
        print(f"{RED}✗ Build failed. Exiting.{RESET}")
        sys.exit(1)

    # Extract executable name
    cargo_toml = (project_dir / "Cargo.toml").read_text()
    exe_name = None
    for line in cargo_toml.splitlines():
        if line.strip().startswith("name"):
            exe_name = line.split("=")[1].strip().strip('"')
            break
    if not exe_name:
        exe_name = project_dir.name

    build_path = project_dir / "target" / "release" / exe_name
    if os.name == "nt":
        build_path = build_path.with_suffix(".exe")

    if not build_path.exists():
        print(f"{RED}✗ Error: Built executable not found: {build_path}{RESET}")
        sys.exit(1)

    section("Copying Build Artifacts")
    dest_exe = output_dir / build_path.name
    shutil.copy2(build_path, dest_exe)
    print(f"{GREEN}✓ Copied executable → {dest_exe}{RESET}")

    static_dir = project_dir / "static"
    if static_dir.exists():
        dest_static = output_dir / "static"
        if dest_static.exists():
            shutil.rmtree(dest_static)
        shutil.copytree(static_dir, dest_static)
        print(f"{GREEN}✓ Copied static files → {dest_static}{RESET}")
    else:
        print(f"{YELLOW}⚠ No static directory found, skipping static assets.{RESET}")

    cfg_dir = project_dir / "cfg"
    if cfg_dir.exists():
        dest_cfg = output_dir / "cfg"
        if dest_cfg.exists():
            shutil.rmtree(dest_cfg)
        shutil.copytree(cfg_dir, dest_cfg)
        print(f"{GREEN}✓ Copied config files → {dest_cfg}{RESET}")
    else:
        print(f"{YELLOW}⚠ No cfg directory found, skipping config files.{RESET}")
    print(f"{GREEN}✓ Build artifacts prepared in {output_dir}{RESET}")

    section("Build Complete")
    print(f"{BOLD}{GREEN}Charmline built successfully!{RESET}")
    print(f"{DIM}Executable: {dest_exe}{RESET}")
    print(f"{DIM}Output dir: {output_dir}{RESET}")

    section("Running Charmline Server")
    print(f"{CYAN}→ Starting server... Press Ctrl+C to stop.{RESET}")

    # Run server with graceful Ctrl+C support
    try:
        process = subprocess.Popen(
            [str(dest_exe)],
            cwd=output_dir,
            stdout=sys.stdout,
            stderr=sys.stderr
        )
        process.wait()
    except KeyboardInterrupt:
        print(f"\n{YELLOW}⚠ KeyboardInterrupt received — shutting down Charmline...{RESET}")
        if process.poll() is None:
            try:
                # On Windows terminate is needed; on Unix send SIGINT
                if os.name == "nt":
                    process.terminate()
                else:
                    process.send_signal(signal.SIGINT)
            except Exception:
                pass
        process.wait()
        print(f"{GREEN}✓ Charmline stopped cleanly.{RESET}")
    finally:
        print(f"{DIM}Build-and-run session ended.{RESET}")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(f"{YELLOW}Usage:{RESET} {BOLD}build_and_run.py <project_dir> <output_dir>{RESET}")
        sys.exit(1)

    build_and_run(sys.argv[1], sys.argv[2])
