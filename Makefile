.PHONY: help install uninstall test clean check-deps

# Default target
help:
	@echo "Claude Manager - Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  make install        Install claude-manager"
	@echo "  make uninstall      Remove claude-manager"
	@echo "  make test           Run tests"
	@echo "  make clean          Clean build artifacts"
	@echo "  make check-deps     Check dependencies"
	@echo "  make help           Show this help"
	@echo ""

# Installation
install: check-deps
	@echo "Installing claude-manager..."
	@./install.sh

# Uninstallation
uninstall:
	@echo "Uninstalling claude-manager..."
	@rm -f $(HOME)/.local/bin/claude-manager
	@rm -f $(HOME)/.claude-manager.conf
	@echo "Uninstalled. You may need to manually remove shell rc entries."

# Run tests
test:
	@echo "Running tests..."
	@bash tests/test_basic.sh
	@echo "Tests completed."

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -f *.tar.gz *.zip
	@rm -rf dist/ build/
	@find . -name "*.bak" -delete
	@find . -name "*~" -delete
	@echo "Clean completed."

# Check dependencies
check-deps:
	@echo "Checking dependencies..."
	@command -v bash >/dev/null 2>&1 || { echo "ERROR: bash not found"; exit 1; }
	@command -v sed >/dev/null 2>&1 || { echo "ERROR: sed not found"; exit 1; }
	@command -v grep >/dev/null 2>&1 || { echo "ERROR: grep not found"; exit 1; }
	@command -v find >/dev/null 2>&1 || { echo "ERROR: find not found"; exit 1; }
	@command -v python3 >/dev/null 2>&1 || echo "WARNING: python3 not found (recommended)"
	@echo "Dependency check completed."

# Package release
package: clean
	@echo "Creating release package..."
	@mkdir -p dist
	@tar -czf dist/claude-manager-$(shell date +%Y%m%d).tar.gz \
		src/ docs/ install.sh README.md LICENSE Makefile .gitignore
	@echo "Package created: dist/claude-manager-$(shell date +%Y%m%d).tar.gz"
