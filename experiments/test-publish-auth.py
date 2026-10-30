#!/usr/bin/env python3
"""Exercise the real publish script with a local Cargo stub; never upload a crate."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class PublishAuthTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.build_dir = tempfile.TemporaryDirectory(prefix="publish-auth-build-")
        cls.addClassCleanup(cls.build_dir.cleanup)
        manifest = Path(cls.build_dir.name) / "Cargo.toml"
        manifest.write_text(
            '[package]\nname = "publish-auth-driver"\nversion = "0.0.0"\n'
            'edition = "2021"\n[dependencies]\nregex = "1"\n'
            '[[bin]]\nname = "publish-auth-driver"\npath = '
            + json.dumps(str(ROOT / "scripts/publish-crate.rs"))
            + '\n'
        )
        result = subprocess.run(
            ["cargo", "build", "--manifest-path", str(manifest),
             "--target-dir", str(ROOT / "target/publish-auth"),
             "--message-format=json"],
            capture_output=True, text=True,
        )
        if result.returncode:
            raise RuntimeError(result.stderr)
        cls.binary = next(
            event["executable"]
            for line in result.stdout.splitlines()
            if (event := json.loads(line)).get("executable")
        )

    def publish(self, native=None, legacy=None, expected=None, failure=""):
        with tempfile.TemporaryDirectory(prefix="publish-auth-fixture-") as directory:
            fixture = Path(directory)
            (fixture / "Cargo.toml").write_text(
                '[package]\nname = "auth-fixture"\nversion = "0.0.0"\n'
            )
            stub = fixture / "cargo"
            stub.write_text(
                f"#!{sys.executable}\n"
                "import os, sys\n"
                "args = ['publish', '--allow-dirty', '-p', 'auth-fixture']\n"
                "if os.environ['EXPECTED_TOKEN']:\n"
                "    args += ['--token', os.environ['EXPECTED_TOKEN']]\n"
                "if sys.argv[1:] != args:\n"
                "    sys.exit('unexpected cargo arguments')\n"
                "if os.environ['PUBLISH_FAILURE']:\n"
                "    sys.exit(os.environ['PUBLISH_FAILURE'])\n"
            )
            stub.chmod(0o755)
            env = os.environ.copy()
            for key in ("CARGO_REGISTRY_TOKEN", "CARGO_TOKEN", "RUST_ROOT"):
                env.pop(key, None)
            if native is not None:
                env["CARGO_REGISTRY_TOKEN"] = native
            if legacy is not None:
                env["CARGO_TOKEN"] = legacy
            env.update(
                PATH=str(fixture) + os.pathsep + env["PATH"],
                EXPECTED_TOKEN=expected or "",
                PUBLISH_FAILURE=failure,
                GITHUB_OUTPUT=str(fixture / "output"),
            )
            result = subprocess.run(
                [self.binary], cwd=fixture, env=env,
                capture_output=True, text=True, timeout=20,
            )
            output = (fixture / "output").read_text()
            # Synthetic credentials must not appear in logs either.
            for token in (native, legacy):
                if token:
                    self.assertNotIn(token, result.stdout + result.stderr)
            return result.returncode, output

    def test_oidc_token_overrides_legacy_token(self):
        self.assertEqual(
            self.publish("oidc-test-credential", "legacy-test-credential",
                         "oidc-test-credential"),
            (0, "publish_result=success\n"),
        )

    def test_native_token_fallback(self):
        self.assertEqual(
            self.publish(native="native-test-credential", expected="native-test-credential"),
            (0, "publish_result=success\n"),
        )

    def test_legacy_token_fallback(self):
        self.assertEqual(
            self.publish(legacy="legacy-test-credential", expected="legacy-test-credential"),
            (0, "publish_result=success\n"),
        )

    def test_empty_native_token_allows_legacy_fallback(self):
        self.assertEqual(
            self.publish("", "legacy-test-credential", "legacy-test-credential"),
            (0, "publish_result=success\n"),
        )

    def test_missing_credentials_cannot_report_success(self):
        self.assertEqual(
            self.publish(failure="please provide a non-empty token"),
            (1, "publish_result=auth_failed\n"),
        )

    def test_existing_version_cannot_report_success(self):
        self.assertEqual(
            self.publish(failure="already uploaded"),
            (1, "publish_result=already_exists\n"),
        )

    def test_other_publish_failure_cannot_report_success(self):
        self.assertEqual(
            self.publish(failure="registry unavailable"),
            (1, "publish_result=failed\n"),
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
