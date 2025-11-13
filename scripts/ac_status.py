#!/usr/bin/env python3
"""
AC Status Aggregator

Reads:
- specs/spec_ledger.yaml: AC definitions
- specs/features/*.feature: Gherkin scenarios tagged with @AC-####
- target/junit/acceptance.xml: Test results

Produces:
- docs/feature_status.md: AC status table
"""

import sys
import re
import yaml
import xml.etree.ElementTree as ET
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Set, Optional

# AC data structures
class AC:
    def __init__(self, ac_id: str, story_id: str, req_id: str, text: str):
        self.id = ac_id
        self.story_id = story_id
        self.req_id = req_id
        self.text = text
        self.status = "unknown"
        self.scenarios: List[str] = []

class Scenario:
    def __init__(self, name: str, ac_id: str, file: str):
        self.name = name
        self.ac_id = ac_id
        self.file = file

def parse_ledger(ledger_path: Path) -> Dict[str, AC]:
    """Parse spec_ledger.yaml and extract all AC IDs."""
    acs: Dict[str, AC] = {}

    with open(ledger_path, 'r') as f:
        ledger = yaml.safe_load(f)

    for story in ledger.get('stories', []):
        story_id = story.get('id', 'unknown')
        for req in story.get('requirements', []):
            req_id = req.get('id', 'unknown')
            for ac in req.get('acceptance_criteria', []):
                ac_id = ac.get('id')
                ac_text = ac.get('text', '')
                if ac_id:
                    acs[ac_id] = AC(ac_id, story_id, req_id, ac_text)

    return acs

def parse_features(features_dir: Path) -> Dict[str, Scenario]:
    """Parse all .feature files and extract scenario -> AC mappings."""
    scenarios: Dict[str, Scenario] = {}

    for feature_file in features_dir.glob('*.feature'):
        with open(feature_file, 'r') as f:
            content = f.read()

        # Find all scenarios with their tags
        # Match: @tags followed by Scenario: name
        pattern = r'(@[\w-]+(?:\s+@[\w-]+)*)\s+Scenario(?:\s+Outline)?:\s+(.+)'
        matches = re.finditer(pattern, content)

        for match in matches:
            tags_str = match.group(1)
            scenario_name = match.group(2).strip()

            # Extract AC ID from tags
            ac_match = re.search(r'@(AC-\d+)', tags_str)
            if ac_match:
                ac_id = ac_match.group(1)
                scenarios[scenario_name] = Scenario(
                    scenario_name,
                    ac_id,
                    str(feature_file.relative_to(features_dir.parent))
                )

    return scenarios

def normalize_testcase_name(name: str) -> str:
    """
    Normalize JUnit testcase name to match scenario name.
    Remove suffixes like ' (row 1)', ' (example 1)', etc.
    """
    # Extract just the scenario name part
    # Format is typically: "Scenario: <name>: <file>:<line>:<col>"
    match = re.match(r'Scenario:\s+(.+?):\s+', name)
    if match:
        scenario_name = match.group(1).strip()
        # Remove example/row suffixes
        scenario_name = re.sub(r'\s*\((?:row|example)\s+\d+\)\s*$', '', scenario_name)
        return scenario_name
    return name

def parse_junit(junit_path: Path, scenarios: Dict[str, Scenario]) -> Dict[str, str]:
    """
    Parse JUnit XML and map testcases to scenarios.
    Returns: {ac_id: status} where status is 'pass' or 'fail'
    """
    ac_results: Dict[str, List[bool]] = defaultdict(list)

    tree = ET.parse(junit_path)
    root = tree.getroot()

    for testcase in root.iter('testcase'):
        tc_name = testcase.get('name', '')
        normalized_name = normalize_testcase_name(tc_name)

        # Check if testcase has failures or errors
        has_failure = testcase.find('failure') is not None
        has_error = testcase.find('error') is not None
        passed = not (has_failure or has_error)

        # Find matching scenario
        scenario = scenarios.get(normalized_name)
        if scenario:
            ac_results[scenario.ac_id].append(passed)

    # Aggregate: AC passes only if all testcases pass
    ac_status = {}
    for ac_id, results in ac_results.items():
        ac_status[ac_id] = 'pass' if all(results) else 'fail'

    return ac_status

def generate_status_md(acs: Dict[str, AC], scenarios: Dict[str, Scenario],
                       ac_status: Dict[str, str], output_path: Path):
    """Generate feature_status.md with AC status table."""

    # Map scenarios to ACs
    for scenario_name, scenario in scenarios.items():
        if scenario.ac_id in acs:
            acs[scenario.ac_id].scenarios.append(scenario_name)

    # Update AC status
    for ac_id, status in ac_status.items():
        if ac_id in acs:
            acs[ac_id].status = status

    # ACs with no scenarios remain 'unknown'

    output_path.parent.mkdir(parents=True, exist_ok=True)

    with open(output_path, 'w') as f:
        f.write("# Feature Status\n\n")
        f.write("Auto-generated AC status from acceptance tests.\n\n")

        f.write("## AC Status Summary\n\n")
        f.write("| AC ID | Story | Requirement | Status | Scenarios |\n")
        f.write("|-------|-------|-------------|--------|----------|\n")

        for ac_id in sorted(acs.keys()):
            ac = acs[ac_id]
            status_icon = {
                'pass': '✅',
                'fail': '❌',
                'unknown': '❓'
            }.get(ac.status, '❓')

            scenarios_str = f"{len(ac.scenarios)}"

            f.write(f"| {ac.id} | {ac.story_id} | {ac.req_id} | "
                   f"{status_icon} {ac.status} | {scenarios_str} |\n")

        # Unmapped ACs
        unmapped = [ac for ac in acs.values() if not ac.scenarios]
        if unmapped:
            f.write("\n## Unmapped ACs\n\n")
            f.write("ACs with no mapped scenarios:\n\n")
            for ac in unmapped:
                f.write(f"- {ac.id}: {ac.text[:100]}\n")

        # Unmapped scenarios
        unmapped_scenarios = [
            (name, scenario)
            for name, scenario in scenarios.items()
            if scenario.ac_id not in acs
        ]
        if unmapped_scenarios:
            f.write("\n## Unmapped Scenarios\n\n")
            f.write("Scenarios referencing non-existent ACs:\n\n")
            for name, scenario in unmapped_scenarios:
                f.write(f"- Scenario '{name}' references {scenario.ac_id} "
                       f"(in {scenario.file})\n")

    print(f"✓ Generated {output_path}")

def main():
    workspace_root = Path(__file__).parent.parent
    ledger_path = workspace_root / 'specs' / 'spec_ledger.yaml'
    features_dir = workspace_root / 'specs' / 'features'
    junit_path = workspace_root / 'target' / 'junit' / 'acceptance.xml'
    output_path = workspace_root / 'docs' / 'feature_status.md'

    # Validate inputs
    if not ledger_path.exists():
        print(f"Error: {ledger_path} not found", file=sys.stderr)
        sys.exit(1)

    if not features_dir.exists():
        print(f"Error: {features_dir} not found", file=sys.stderr)
        sys.exit(1)

    if not junit_path.exists():
        print(f"Error: {junit_path} not found", file=sys.stderr)
        print("Run acceptance tests first: cargo test -p acceptance", file=sys.stderr)
        sys.exit(1)

    print(f"Parsing ledger: {ledger_path}")
    acs = parse_ledger(ledger_path)
    print(f"  Found {len(acs)} ACs")

    print(f"Parsing features: {features_dir}")
    scenarios = parse_features(features_dir)
    print(f"  Found {len(scenarios)} scenarios")

    print(f"Parsing JUnit results: {junit_path}")
    ac_status = parse_junit(junit_path, scenarios)
    print(f"  Found results for {len(ac_status)} ACs")

    print(f"Generating status: {output_path}")
    generate_status_md(acs, scenarios, ac_status, output_path)

    # Exit with non-zero if any AC failed
    failed = [ac_id for ac_id, status in ac_status.items() if status == 'fail']
    if failed:
        print(f"\n❌ {len(failed)} AC(s) failed: {', '.join(failed)}", file=sys.stderr)
        sys.exit(1)

    print("\n✓ All ACs passed")

if __name__ == '__main__':
    main()
