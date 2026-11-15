#!/usr/bin/env python3
"""Test registry configuration requests without credentials or network access."""

import importlib.util
import io
import json
from pathlib import Path
import unittest
from unittest.mock import patch
from urllib.error import HTTPError

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "publisher", ROOT / "scripts/register-trusted-publisher.py"
)
publisher = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(publisher)


def response(data):
    return io.BytesIO(json.dumps(data).encode())


class RegistrationTests(unittest.TestCase):
    @patch.object(publisher, "urlopen")
    def test_registers_the_caller_workflow(self, request):
        request.side_effect = [
            response({"github_configs": [], "meta": {"next_page": None}}),
            response({"github_config": publisher.PUBLISHER}),
        ]
        publisher.register("synthetic-credential")
        post = request.call_args_list[1].args[0]
        self.assertEqual(post.method, "POST")
        self.assertEqual(post.full_url, publisher.ENDPOINT)
        self.assertEqual(json.loads(post.data), {"github_config": publisher.PUBLISHER})
        self.assertEqual(post.get_header("Authorization"), "synthetic-credential")

    @patch.object(publisher, "urlopen")
    def test_existing_registration_on_later_page_is_reused(self, request):
        request.side_effect = [
            response({"github_configs": [], "meta": {"next_page": "?crate=meta-ontology&seek=2"}}),
            response({"github_configs": [publisher.PUBLISHER]}),
        ]
        publisher.register("synthetic-credential")
        self.assertEqual(request.call_count, 2)
        self.assertTrue(request.call_args.args[0].full_url.endswith("&seek=2"))

    @patch.object(publisher, "urlopen")
    def test_missing_token_does_not_send_requests(self, request):
        with self.assertRaisesRegex(ValueError, "crate owner API token"):
            publisher.register("")
        request.assert_not_called()

    @patch.object(publisher, "urlopen")
    def test_permission_failure_is_not_success(self, request):
        request.side_effect = HTTPError(publisher.ENDPOINT, 403, "Forbidden", {}, None)
        with self.assertRaises(HTTPError):
            publisher.register("synthetic-credential")

    @patch.object(publisher, "urlopen")
    def test_unexpected_configuration_is_not_success(self, request):
        request.side_effect = [
            response({"github_configs": []}),
            response({"github_config": {**publisher.PUBLISHER, "workflow_filename": "other.yml"}}),
        ]
        with self.assertRaisesRegex(ValueError, "unexpected trusted publisher"):
            publisher.register("synthetic-credential")


if __name__ == "__main__":
    unittest.main(verbosity=2)
