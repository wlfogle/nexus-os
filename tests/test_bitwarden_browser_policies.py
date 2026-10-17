import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock


SOURCE = (Path(__file__).resolve().parents[1]
          / "scripts/setup-bitwarden-browsers.py")
SPEC = importlib.util.spec_from_file_location("bitwarden_policies", SOURCE)
POLICIES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(POLICIES)


class BrowserPolicyTests(unittest.TestCase):
    def test_merge_preserves_unrelated_keys(self):
        existing = {"policies": {"DNSOverHTTPS": {"Enabled": False}}}
        updated = POLICIES.merge(existing, {
            "policies": {"3rdparty": {"Extensions": {POLICIES.FIREFOX_ID: {
                "environment": {"base": POLICIES.SERVER}
            }}}}
        })
        self.assertFalse(updated["policies"]["DNSOverHTTPS"]["Enabled"])
        self.assertNotIn("3rdparty", existing["policies"])

    def test_other_extension_configuration_survives(self):
        original = {
            "ExtensionSettings": {"other-extension": {"blocked": True}},
        }
        result = POLICIES.merge(original, POLICIES.chromium_policy(
            POLICIES.CHROME_ID, POLICIES.CHROME_UPDATE))
        self.assertEqual(result["ExtensionSettings"]["other-extension"],
                         {"blocked": True})

    def test_invalid_object_is_rejected(self):
        with self.assertRaises(ValueError):
            POLICIES.merge({"policies": []}, {"policies": {"3rdparty": {}}})

    def test_install_backup_and_idempotence(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "etc/firefox/policies/policies.json"
            target.parent.mkdir(parents=True)
            original = '{"policies":{"DNSOverHTTPS":{"Enabled":false}}}\n'
            target.write_text(original)
            backup = POLICIES.install(root)
            self.assertEqual((backup / "etc/firefox/policies/policies.json")
                             .read_text(), original)
            self.assertFalse(json.loads(target.read_text())["policies"]
                             ["DNSOverHTTPS"]["Enabled"])
            self.assertEqual(target.stat().st_mode & 0o777, 0o644)
            self.assertEqual(backup.stat().st_mode & 0o777, 0o700)
            self.assertIsNone(POLICIES.install(root))
            self.assertEqual(len(list((root / "var/backups").iterdir())), 1)

    def test_invalid_input_prevents_partial_application(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "etc/firefox/policies/policies.json"
            target.parent.mkdir(parents=True)
            target.write_text("not JSON")
            with self.assertRaises(json.JSONDecodeError):
                POLICIES.install(root)
            self.assertFalse((root / "etc/opt/chrome").exists())

    def test_policy_file_symlink_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = (root / "etc/opt/chrome/policies/managed"
                      / "nexus-bitwarden.json")
            target.parent.mkdir(parents=True)
            original = root / "other.json"
            original.write_text("{}")
            target.symlink_to(original)
            with self.assertRaises(ValueError):
                POLICIES.install(root)
            self.assertEqual(original.read_text(), "{}")

    def test_official_ids_and_server_cover_all_policy_roots(self):
        changes = POLICIES.policy_changes()
        self.assertEqual(len(changes), 7)
        for path, policy in changes.items():
            if "firefox" in path:
                policy = policy["policies"]
                environments = policy["3rdparty"]["Extensions"]
                install = policy["ExtensionSettings"][POLICIES.FIREFOX_ID]
                self.assertEqual(install["install_url"],
                                 POLICIES.FIREFOX_UPDATE)
            else:
                environments = policy["3rdparty"]["extensions"]
                extension_id = (POLICIES.OPERA_ID if "/opera" in path
                                else POLICIES.CHROME_ID)
                self.assertIn(extension_id, environments)
            for value in environments.values():
                self.assertEqual(value["environment"]["base"], POLICIES.SERVER)

    def test_flatpak_override_rejects_root(self):
        with mock.patch.object(POLICIES.os, "geteuid", return_value=0):
            with self.assertRaises(ValueError):
                POLICIES.configure_opera_flatpak()

    def test_flatpak_existing_readonly_override_is_idempotent(self):
        result = mock.Mock(stdout="[Context]\nfilesystems=host-etc:ro;\n")
        with mock.patch.object(POLICIES.os, "geteuid", return_value=1000), \
                mock.patch.object(POLICIES.subprocess, "run",
                                  return_value=result) as run:
            POLICIES.configure_opera_flatpak()
        self.assertEqual(run.call_count, 2)
        self.assertTrue(all("--filesystem=host-etc:ro" not in c.args[0]
                            for c in run.call_args_list))

    def test_flatpak_override_backup_preserves_original(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            data = root / "data"
            state = root / "state"
            override = data / "flatpak/overrides/com.opera.Opera"
            override.parent.mkdir(parents=True)
            original = "[Context]\nfilesystems=xdg-download;\n"
            override.write_text(original)
            with mock.patch.dict(POLICIES.os.environ, {
                "XDG_DATA_HOME": str(data), "XDG_STATE_HOME": str(state),
            }), mock.patch.object(POLICIES.os, "geteuid", return_value=1000), \
                    mock.patch.object(
                        POLICIES.subprocess, "run",
                        return_value=mock.Mock(stdout=""),
                    ) as run:
                POLICIES.configure_opera_flatpak()
            backup = next((state / "nexus-bitwarden").iterdir())
            self.assertEqual((backup / "opera-flatpak-override").read_text(),
                             original)
            self.assertTrue(json.loads((backup / "state.json").read_text())
                            ["existed"])
            self.assertEqual(backup.stat().st_mode & 0o777, 0o700)
            self.assertEqual(run.call_args.args[0], [
                "flatpak", "override", "--user", "--filesystem=host-etc:ro",
                "com.opera.Opera",
            ])


if __name__ == "__main__":
    unittest.main()
