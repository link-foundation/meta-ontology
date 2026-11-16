#!/usr/bin/env python3
"""Exercise the workflow's real bootstrap shell with local registry/Cargo stubs."""

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import textwrap
import unittest

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = (ROOT / ".github/workflows/release.yml").read_text()
STEP = textwrap.dedent(WORKFLOW.split("      - name: Publish the committed version\n", 1)[1]
                       .split("        run: |\n", 1)[1].split("\n      - name:", 1)[0])


class BootstrapTests(unittest.TestCase):
    def bootstrap(self, status="404", token="synthetic-credential", network_failure=False):
        with tempfile.TemporaryDirectory(prefix="bootstrap-test-") as directory:
            root = Path(directory)
            (root / "Cargo.toml").write_text('[package]\nversion = "0.10.0"\n')
            stubs = {
                "curl": """
                    import os, sys
                    args = sys.argv[1:]
                    # crates.io rejects the generic curl agent. Require a
                    # project identifier and contact URL in the actual request.
                    assert '--user-agent' in args, 'missing registry user agent'
                    agent = args[args.index('--user-agent') + 1]
                    assert 'meta-ontology-release-bootstrap' in agent
                    assert 'https://github.com/link-foundation/meta-ontology' in agent
                    assert args[-1] == 'https://crates.io/api/v1/crates/meta-ontology/0.10.0'
                    if os.environ['NETWORK_FAILURE'] == '1':
                        sys.exit(7)
                    print(os.environ['REGISTRY_STATUS'], end='')
                """,
                "cargo": """
                    import pathlib, sys
                    assert sys.argv[1:] == ['publish', '--locked']
                    pathlib.Path('published').touch()
                """,
                "git": """
                    import sys
                    assert sys.argv[1:] == ['status', '--porcelain']
                """,
            }
            for name, body in stubs.items():
                path = root / name
                path.write_text(f"#!{sys.executable}\n" + textwrap.dedent(body))
                path.chmod(0o755)
            env = {**os.environ, "PATH": directory + os.pathsep + os.environ["PATH"],
                   "CARGO_REGISTRY_TOKEN": token, "REGISTRY_STATUS": status,
                   "NETWORK_FAILURE": str(int(network_failure))}
            result = subprocess.run(["bash", "-e", "-c", STEP], cwd=root, env=env,
                                    capture_output=True, text=True, timeout=20)
            self.assertNotIn("synthetic-credential", result.stdout + result.stderr)
            return result.returncode, (root / "published").exists()

    def test_missing_version_is_published_with_locked_dependencies(self):
        self.assertEqual(self.bootstrap(), (0, True))

    def test_existing_version_is_skipped_for_registration_retry(self):
        self.assertEqual(self.bootstrap(status="200"), (0, False))

    def test_registry_permission_failure_stops_publication(self):
        self.assertEqual(self.bootstrap(status="403"), (1, False))

    def test_missing_token_stops_publication(self):
        self.assertEqual(self.bootstrap(token=""), (1, False))

    def test_network_failure_stops_publication(self):
        code, published = self.bootstrap(network_failure=True)
        self.assertNotEqual(code, 0)
        self.assertFalse(published)


if __name__ == "__main__":
    unittest.main(verbosity=2)
