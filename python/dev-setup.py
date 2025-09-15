#!/usr/bin/env python3
"""
Development environment setup for Claude Manager TUI
Sets up type checking, testing, and development tools
"""

import sys
import subprocess
import shutil
from pathlib import Path
from typing import List, Optional

def run_command(cmd: List[str], check: bool = True) -> subprocess.CompletedProcess:
    """Run a command and handle errors gracefully"""
    print(f"Running: {' '.join(cmd)}")
    try:
        result = subprocess.run(cmd, check=check, capture_output=True, text=True)
        if result.stdout:
            print(result.stdout)
        return result
    except subprocess.CalledProcessError as e:
        print(f"Error running command: {e}")
        if e.stderr:
            print(f"Error output: {e.stderr}")
        if check:
            raise
        return e

def check_python_version() -> bool:
    """Check if Python version is 3.13+"""
    version = sys.version_info
    if version.major != 3 or version.minor < 13:
        print(f"Error: Python 3.13+ required, found {version.major}.{version.minor}")
        return False
    print(f"✓ Python {version.major}.{version.minor}.{version.micro} detected")
    return True

def check_uv_available() -> bool:
    """Check if uv is available"""
    return shutil.which("uv") is not None

def install_with_uv() -> bool:
    """Install dependencies using uv (preferred)"""
    try:
        # Install base dependencies
        run_command(["uv", "pip", "install", "-e", ".[dev]"])
        print("✓ Dependencies installed with uv")
        return True
    except subprocess.CalledProcessError:
        print("⚠ Failed to install with uv, falling back to pip")
        return False

def install_with_pip() -> bool:
    """Install dependencies using pip (fallback)"""
    try:
        # Upgrade pip first
        run_command([sys.executable, "-m", "pip", "install", "--upgrade", "pip"])
        
        # Install in editable mode with dev dependencies
        run_command([sys.executable, "-m", "pip", "install", "-e", ".[dev]"])
        print("✓ Dependencies installed with pip")
        return True
    except subprocess.CalledProcessError:
        print("✗ Failed to install dependencies")
        return False

def setup_pre_commit() -> bool:
    """Set up pre-commit hooks"""
    try:
        # Create .pre-commit-config.yaml if it doesn't exist
        pre_commit_config = Path(".pre-commit-config.yaml")
        if not pre_commit_config.exists():
            config_content = """
repos:
  - repo: https://github.com/pre-commit/pre-commit-hooks
    rev: v4.5.0
    hooks:
      - id: trailing-whitespace
      - id: end-of-file-fixer
      - id: check-yaml
      - id: check-added-large-files
      - id: check-json
      - id: pretty-format-json
        args: [--autofix]

  - repo: https://github.com/astral-sh/ruff-pre-commit
    rev: v0.1.0
    hooks:
      - id: ruff
        args: [--fix, --exit-non-zero-on-fix]
      - id: ruff-format

  - repo: https://github.com/psf/black
    rev: 23.10.1
    hooks:
      - id: black

  - repo: https://github.com/pycqa/isort
    rev: 5.12.0
    hooks:
      - id: isort

  - repo: https://github.com/pre-commit/mirrors-mypy
    rev: v1.8.0
    hooks:
      - id: mypy
        additional_dependencies: [types-all]
        args: [--strict]
"""
            pre_commit_config.write_text(config_content.strip())
            print("✓ Created .pre-commit-config.yaml")
        
        # Install pre-commit hooks
        run_command(["pre-commit", "install"])
        print("✓ Pre-commit hooks installed")
        return True
    except subprocess.CalledProcessError:
        print("⚠ Failed to setup pre-commit hooks")
        return False

def run_type_checking() -> bool:
    """Run type checking tools"""
    success = True
    
    # Run mypy
    try:
        run_command(["mypy", "claude_manager_tui_typed.py"])
        print("✓ mypy type checking passed")
    except subprocess.CalledProcessError:
        print("✗ mypy type checking failed")
        success = False
    
    # Run pyright if available
    if shutil.which("pyright"):
        try:
            run_command(["pyright", "claude_manager_tui_typed.py"])
            print("✓ pyright type checking passed")
        except subprocess.CalledProcessError:
            print("✗ pyright type checking failed")
            success = False
    else:
        print("⚠ pyright not available, skipping")
    
    return success

def run_linting() -> bool:
    """Run linting tools"""
    success = True
    
    # Run ruff
    try:
        run_command(["ruff", "check", "."])
        print("✓ ruff linting passed")
    except subprocess.CalledProcessError:
        print("✗ ruff linting failed")
        success = False
    
    # Check black formatting
    try:
        run_command(["black", "--check", "."])
        print("✓ black formatting check passed")
    except subprocess.CalledProcessError:
        print("✗ black formatting check failed")
        success = False
    
    # Check isort
    try:
        run_command(["isort", "--check-only", "."])
        print("✓ isort import sorting check passed")
    except subprocess.CalledProcessError:
        print("✗ isort import sorting check failed")
        success = False
    
    return success

def run_tests() -> bool:
    """Run the test suite"""
    try:
        run_command(["pytest", "-v", "--cov-report=term-missing"])
        print("✓ Test suite passed")
        return True
    except subprocess.CalledProcessError:
        print("✗ Test suite failed")
        return False

def create_development_scripts() -> None:
    """Create useful development scripts"""
    scripts_dir = Path("scripts")
    scripts_dir.mkdir(exist_ok=True)
    
    # Type checking script
    type_check_script = scripts_dir / "type-check.sh"
    type_check_script.write_text("""#!/bin/bash
set -e

echo "Running type checking..."
mypy claude_manager_tui_typed.py
echo "✓ mypy passed"

if command -v pyright &> /dev/null; then
    pyright claude_manager_tui_typed.py
    echo "✓ pyright passed"
else
    echo "⚠ pyright not available"
fi

echo "All type checking passed!"
""")
    type_check_script.chmod(0o755)
    
    # Linting script
    lint_script = scripts_dir / "lint.sh"
    lint_script.write_text("""#!/bin/bash
set -e

echo "Running linting..."
ruff check .
echo "✓ ruff passed"

black --check .
echo "✓ black formatting check passed"

isort --check-only .
echo "✓ isort import sorting check passed"

echo "All linting passed!"
""")
    lint_script.chmod(0o755)
    
    # Format script
    format_script = scripts_dir / "format.sh"
    format_script.write_text("""#!/bin/bash
set -e

echo "Formatting code..."
ruff check --fix .
black .
isort .

echo "Code formatting complete!"
""")
    format_script.chmod(0o755)
    
    # Test script
    test_script = scripts_dir / "test.sh"
    test_script.write_text("""#!/bin/bash
set -e

echo "Running tests..."
pytest -v --cov-report=term-missing --cov-report=html

echo "Tests complete! Check htmlcov/index.html for coverage report."
""")
    test_script.chmod(0o755)
    
    print("✓ Development scripts created in scripts/")

def main() -> None:
    """Main setup function"""
    print("Claude Manager TUI Development Setup")
    print("=" * 40)
    
    # Check Python version
    if not check_python_version():
        sys.exit(1)
    
    # Install dependencies
    success = False
    if check_uv_available():
        print("Using uv for dependency management (recommended)")
        success = install_with_uv()
    
    if not success:
        print("Using pip for dependency management")
        success = install_with_pip()
    
    if not success:
        print("Failed to install dependencies")
        sys.exit(1)
    
    # Setup development environment
    setup_pre_commit()
    create_development_scripts()
    
    print("\nRunning initial validation...")
    
    # Run type checking
    type_check_success = run_type_checking()
    
    # Run linting
    lint_success = run_linting()
    
    # Run tests
    test_success = run_tests()
    
    print("\nSetup Summary:")
    print("=" * 40)
    print(f"Dependencies installed: ✓")
    print(f"Pre-commit hooks: {'✓' if Path('.pre-commit-config.yaml').exists() else '✗'}")
    print(f"Type checking: {'✓' if type_check_success else '✗'}")
    print(f"Linting: {'✓' if lint_success else '✗'}")
    print(f"Tests: {'✓' if test_success else '✗'}")
    
    if all([type_check_success, lint_success, test_success]):
        print("\n🎉 Development environment setup complete!")
        print("\nUseful commands:")
        print("  python -m pytest                    # Run tests")
        print("  mypy claude_manager_tui_typed.py    # Type checking")
        print("  ruff check .                        # Linting")
        print("  black .                             # Code formatting")
        print("  ./scripts/type-check.sh             # Full type checking")
        print("  ./scripts/lint.sh                   # Full linting")
        print("  ./scripts/format.sh                 # Auto-format code")
        print("  ./scripts/test.sh                   # Run tests with coverage")
    else:
        print("\n⚠ Setup completed with some issues. Check the output above.")
        sys.exit(1)

if __name__ == "__main__":
    main()