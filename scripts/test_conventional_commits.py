import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from scripts import conventional_commits


class ConventionalCommitHistoryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="herdr-commit-history-")
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        excludes = self.repo / ".test-global-excludes"
        excludes.write_text("", encoding="utf-8")
        self.env = {**os.environ, "GIT_CONFIG_COUNT": "1",
                    "GIT_CONFIG_KEY_0": "core.excludesfile", "GIT_CONFIG_VALUE_0": str(excludes)}
        self.git("init", "-q", "-b", "master")
        self.git("config", "user.name", "History Test")
        self.git("config", "user.email", "history@example.invalid")
        self.commit("base", "chore: initial history")
        self.git("switch", "-q", "-c", "foreign-fork")
        self.commit("foreign", "Original external history")
        self.git("switch", "-q", "master")
        self.commit("mainline", "fix: mainline change")
        self.base = self.git("rev-parse", "HEAD").strip()
        self.git("merge", "-q", "--no-ff", "foreign-fork", "-m", "feat: integrate external fork")

    def git(self, *args):
        result = subprocess.run(["git", *args], cwd=self.repo, env=self.env,
                                text=True, capture_output=True)
        self.assertEqual(result.returncode, 0,
                         f"git {args!r}: {result.stdout}{result.stderr}")
        return result.stdout

    def commit(self, filename, message):
        (self.repo / filename).write_text(message, encoding="utf-8")
        self.git("add", filename)
        self.git("commit", "-qm", message)

    def validate(self):
        script = Path(conventional_commits.__file__).resolve()
        return subprocess.run([sys.executable, str(script), "--range", f"{self.base}..HEAD"],
                              cwd=self.repo, env=self.env, text=True, capture_output=True)

    def test_foreign_history_preserves_authors_subjects(self):
        result = self.validate()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_invalid_mainline_commit_is_still_rejected(self):
        self.commit("bad-mainline", "Invalid mainline subject")
        result = self.validate()
        self.assertEqual(result.returncode, 1)
        self.assertIn("Invalid mainline subject", result.stdout)
        self.assertNotIn("Original external history", result.stdout)

    def test_invalid_merge_subject_is_still_rejected(self):
        self.git("commit", "--amend", "-qm", "Invalid integration subject")
        result = self.validate()
        self.assertEqual(result.returncode, 1)
        self.assertIn("Invalid integration subject", result.stdout)


if __name__ == "__main__":
    unittest.main()
