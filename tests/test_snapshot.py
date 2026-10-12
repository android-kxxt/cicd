import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


CICD = Path(__file__).resolve().parents[1]


class SnapshotTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        self.git("init", "-q")
        self.git("-c", "user.name=Test", "-c", "user.email=test@example.com",
                 "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null",
                 "commit", "-qm", "Manifest revision", "--allow-empty")
        self.commit = self.git("rev-parse", "HEAD").strip()
        self.git("update-ref", "refs/remotes/github/lineage-24.0", self.commit)
        self.git("-c", "user.name=Test", "-c", "user.email=test@example.com",
                 "-c", "commit.gpgsign=false", "-c", "core.hooksPath=/dev/null",
                 "commit", "-qm", "Local patch", "--allow-empty")

    def git(self, *args):
        return subprocess.check_output(
            ["git", *args], cwd=self.repo, text=True
        )

    def format(self, revision, *args):
        return subprocess.run(
            [str(CICD / "format-repo-commit.sh"), *args],
            cwd=self.repo, text=True, capture_output=True,
            env={**os.environ, "REPO_PATH": "test/project", "REPO_LREV": revision},
        )

    def test_manifest_ref_and_pinned_revision(self):
        # HEAD includes a local patch; snapshots must record the manifest base.
        for revision in ("refs/remotes/github/lineage-24.0", self.commit):
            with self.subTest(revision=revision):
                result = self.format(revision, "simple")
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(result.stdout, f"test/project: {self.commit[:12]}\n")

    def test_markdown_without_arguments(self):
        result = self.format(self.commit)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, f"- `test/project`: `{self.commit[:12]}` ")

    def test_bad_revision_fails_without_partial_output(self):
        for revision in ("refs/remotes/github/missing", ""):
            with self.subTest(revision=revision):
                result = self.format(revision, "simple")
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(result.stdout, "")

    def test_snapshot_invocation_and_failure_propagation(self):
        source = (CICD / "snapshot.c").read_text().split("\n", 1)[1]
        executable = self.repo / "snapshot.exe"
        subprocess.run(
            ["gcc", "-Wall", "-Wextra", "-Werror", f"-DSCRIPT_DIR={CICD}",
             "-x", "c", "-", "-o", str(executable)],
            input=source, text=True, check=True,
        )
        fake_repo = self.repo / "repo"
        fake_repo.write_text(
            "#!/usr/bin/env python3\n"
            "import json, sys\n"
            "print(json.dumps(sys.argv[1:]))\n"
            "sys.exit(23)\n"
        )
        fake_repo.chmod(0o755)
        result = subprocess.run(
            [str(executable)], text=True, capture_output=True,
            env={**os.environ, "PATH": f"{self.repo}:{os.environ['PATH']}"},
        )
        self.assertEqual(result.returncode, 23)
        self.assertEqual(json.loads(result.stdout), [
            "forall", "-e", "-c", str(CICD / "format-repo-commit.sh"), "simple"
        ])


if __name__ == "__main__":
    unittest.main()
