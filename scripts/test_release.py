import io
import json
import unittest
import urllib.error
from unittest.mock import patch

import release


class ReleaseTests(unittest.TestCase):
    def metadata(self):
        return json.dumps({
            "packages": [{"name": name, "version": "1.0.0", "id": name}
                         for name in (*release.ORDER, "ddd-toolkit-facade-tests")],
            "workspace_members": [*release.ORDER, "ddd-toolkit-facade-tests"],
        })

    @patch("release.subprocess.run")
    @patch("release.subprocess.check_output")
    @patch("release.published")
    def test_default_is_dry_run_in_dependency_order(self, published, metadata, run):
        metadata.return_value = self.metadata()
        release.release()
        published.assert_not_called()
        self.assertEqual([c.args[0] for c in run.call_args_list], [
            ["cargo", "publish", "--locked", "-p", name, "--dry-run"]
            for name in release.ORDER
        ])

    @patch("release.subprocess.run")
    @patch("release.subprocess.check_output")
    @patch("release.published", side_effect=[True, False, False])
    def test_publish_skips_existing_versions(self, published, metadata, run):
        metadata.return_value = self.metadata()
        release.release(publish=True)
        self.assertEqual([c.args[0] for c in run.call_args_list], [
            ["cargo", "publish", "--locked", "-p", name]
            for name in release.ORDER[1:]
        ])

    @patch("release.subprocess.run", side_effect=RuntimeError("publish failed"))
    @patch("release.subprocess.check_output")
    @patch("release.published", return_value=False)
    def test_failure_stops_dependents(self, published, metadata, run):
        metadata.return_value = self.metadata()
        with self.assertRaisesRegex(RuntimeError, "publish failed"):
            release.release(publish=True)
        self.assertEqual(run.call_count, 1)

    @patch("release.urllib.request.urlopen")
    def test_only_404_means_missing(self, urlopen):
        for status in (404, 403, 429, 500):
            error = urllib.error.HTTPError("url", status, "error", {}, io.BytesIO())
            urlopen.side_effect = error
            try:
                if status == 404:
                    self.assertFalse(release.published("name", "1.0.0"))
                else:
                    with self.assertRaises(urllib.error.HTTPError):
                        release.published("name", "1.0.0")
            finally:
                error.close()

    @patch("release.urllib.request.urlopen")
    def test_existing_and_yanked(self, urlopen):
        for yanked in (False, True):
            urlopen.return_value = io.StringIO(json.dumps({"version": {"yanked": yanked}}))
            if yanked:
                with self.assertRaisesRegex(RuntimeError, "yanked"):
                    release.published("name", "1.0.0")
            else:
                self.assertTrue(release.published("name", "1.0.0"))


if __name__ == "__main__":
    unittest.main()
