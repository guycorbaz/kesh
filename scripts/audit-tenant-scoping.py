#!/usr/bin/env python3
"""
Audit script for multi-tenant scoping verification (KF-002).

Analyzes route files to verify tenant isolation is properly implemented.
Generates CSV and markdown reports of findings.
"""

import os
import re
import csv
from pathlib import Path
from typing import List, Dict, Tuple

# Configuration
ROUTES_DIR = Path("crates/kesh-api/src/routes")
REPOSITORIES_DIR = Path("crates/kesh-db/src/repositories")
OUTPUT_DIR = Path(".")

class RouteAnalyzer:
    """Analyzes Rust route files for tenant scoping."""

    def __init__(self):
        self.findings = []
        self.endpoints = []

    def analyze_all_routes(self) -> List[Dict]:
        """Analyze all route files and return findings."""
        if not ROUTES_DIR.exists():
            print(f"Routes directory not found: {ROUTES_DIR}")
            return []

        route_files = sorted(ROUTES_DIR.glob("*.rs"))

        for route_file in route_files:
            # Skip mod.rs and test files
            if route_file.name in ["mod.rs", "lib.rs"]:
                continue

            self.analyze_route_file(route_file)

        return self.endpoints

    def analyze_route_file(self, filepath: Path) -> None:
        """Analyze a single route file for tenant scoping."""
        try:
            with open(filepath, 'r', encoding='utf-8') as f:
                content = f.read()
        except Exception as e:
            print(f"Error reading {filepath}: {e}")
            return

        module_name = filepath.stem

        # Find all public async functions (handlers) - handle multiline signatures
        handler_pattern = r'pub\s+async\s+fn\s+(\w+)\s*\('
        handlers = re.findall(handler_pattern, content)

        for handler in handlers:
            # Check for CurrentUser extension
            has_current_user = f'Extension(current_user): Extension<CurrentUser>' in content

            # Check if handler uses current_user
            handler_section = re.search(
                rf'pub\s+async\s+fn\s+{handler}\s*\([^{{]*{{[^}}]*?(?={{}})',
                content,
                re.DOTALL
            )

            if handler_section:
                handler_code = handler_section.group(0)
                uses_current_user = 'current_user' in handler_code
                uses_company_id = 'company_id' in handler_code
            else:
                uses_current_user = 'current_user' in content
                uses_company_id = 'company_id' in content

            # Determine scoping status
            if has_current_user and uses_current_user and uses_company_id:
                status = "✅ PASS"
            elif has_current_user and uses_current_user:
                status = "⚠️ MANUAL_CHECK"
            elif handler in ['health_check', 'login', 'refresh_token', 'i18n_bundle']:
                # These are public endpoints that don't require tenant scoping
                status = "⚠️ PUBLIC_ENDPOINT"
            else:
                status = "❌ FAIL"

            self.endpoints.append({
                'module': module_name,
                'handler': handler,
                'has_current_user_extension': has_current_user,
                'uses_current_user': uses_current_user,
                'uses_company_id': uses_company_id,
                'status': status,
                'notes': self._get_notes(module_name, handler, status)
            })

    def _get_notes(self, module: str, handler: str, status: str) -> str:
        """Generate notes based on analysis."""
        if status == "✅ PASS":
            return "Proper tenant scoping via current_user.company_id"
        elif status == "⚠️ MANUAL_CHECK":
            return "Uses CurrentUser but needs manual verification"
        elif status == "⚠️ PUBLIC_ENDPOINT":
            return "Public endpoint (no tenant required)"
        else:
            return f"⚠️ Needs review - {module}/{handler}"

    def generate_csv_report(self, filepath: Path) -> None:
        """Generate CSV report of audit findings."""
        with open(filepath, 'w', newline='', encoding='utf-8') as f:
            writer = csv.DictWriter(f, fieldnames=[
                'module',
                'handler',
                'has_current_user_extension',
                'uses_current_user',
                'uses_company_id',
                'status',
                'notes'
            ])
            writer.writeheader()
            writer.writerows(self.endpoints)

        print(f"✅ CSV report generated: {filepath}")

    def generate_markdown_report(self, filepath: Path) -> None:
        """Generate markdown report of audit findings."""
        content = """# Multi-Tenant Scoping Audit Report (KF-002)

## Route Analysis Summary

| Status | Count |
|--------|-------|
"""

        # Count by status
        status_counts = {}
        for ep in self.endpoints:
            status = ep['status'].split()[0]  # Get emoji part
            status_counts[status] = status_counts.get(status, 0) + 1

        for status, count in sorted(status_counts.items()):
            content += f"| {status} | {count} |\n"

        content += "\n## Detailed Findings\n\n"

        # Group by module
        by_module = {}
        for ep in self.endpoints:
            if ep['module'] not in by_module:
                by_module[ep['module']] = []
            by_module[ep['module']].append(ep)

        for module in sorted(by_module.keys()):
            content += f"### {module}.rs\n\n"
            content += "| Handler | Status | Notes |\n"
            content += "|---------|--------|-------|\n"

            for ep in by_module[module]:
                content += f"| `{ep['handler']}` | {ep['status']} | {ep['notes']} |\n"

            content += "\n"

        # Findings section
        critical_findings = [ep for ep in self.endpoints if "❌" in ep['status']]
        if critical_findings:
            content += "## ⚠️ Critical Findings\n\n"
            for ep in critical_findings:
                content += f"- **{ep['module']}/{ep['handler']}**: {ep['notes']}\n"

        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(content)

        print(f"✅ Markdown report generated: {filepath}")

def main():
    """Run the audit."""
    print("🔍 Starting multi-tenant scoping audit...\n")

    analyzer = RouteAnalyzer()
    analyzer.analyze_all_routes()

    # Generate reports
    csv_report = OUTPUT_DIR / "endpoints-audit.csv"
    md_report = OUTPUT_DIR / "sql-audit.md"  # Will be updated separately for SQL

    analyzer.generate_csv_report(csv_report)
    analyzer.generate_markdown_report(md_report)

    # Summary
    total = len(analyzer.endpoints)
    passed = len([e for e in analyzer.endpoints if "✅" in e['status']])
    manual = len([e for e in analyzer.endpoints if "⚠️" in e['status']])
    failed = len([e for e in analyzer.endpoints if "❌" in e['status']])

    print(f"\n📊 Summary:")
    print(f"  Total endpoints: {total}")
    print(f"  ✅ Passed: {passed}")
    print(f"  ⚠️ Manual check: {manual}")
    print(f"  ❌ Failed: {failed}")

if __name__ == "__main__":
    main()
