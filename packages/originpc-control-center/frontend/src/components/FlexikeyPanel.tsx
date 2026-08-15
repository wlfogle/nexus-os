import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Action, Profile, ProfilesIndex } from "../types";

// Self-contained Flexikey profile/macro editor. Exported as `FlexikeyPanel`
// so frontend-agent's Key Bindings tab can mount it directly - see
// CONTRACT.md's flexikey-agent section for the integration point. Talks
// only to the eight `*flexikey*` commands added in `src-tauri/src/lib.rs`;
// it owns no state outside this component.

type ActionKind = Action["type"];

const ACTION_KINDS: ActionKind[] = ["remap", "combo", "text", "launch", "disabled"];

const ACTION_VALUE_HINT: Record<ActionKind, string> = {
  remap: "target key, e.g. KEY_A",
  combo: "comma-separated keys, e.g. KEY_LEFTCTRL,KEY_LEFTSHIFT,KEY_ESC",
  text: "literal text to type",
  launch: "shell command, e.g. gnome-terminal",
  disabled: "(no value needed - key is swallowed entirely)",
};

/// Builds an `Action` from the editor's kind selector + free-text value
/// field, mirroring the Python GUI's `add_mapping` parsing.
function buildAction(kind: ActionKind, value: string): Action {
  switch (kind) {
    case "remap":
      return { type: "remap", target: value.trim() };
    case "combo":
      return {
        type: "combo",
        keys: value
          .split(",")
          .map((k) => k.trim())
          .filter((k) => k.length > 0),
      };
    case "text":
      return { type: "text", text: value };
    case "launch":
      return { type: "launch", command: value.trim() };
    case "disabled":
      return { type: "disabled" };
  }
}

/// Renders an `Action` back into the editor's single free-text value field
/// when editing an existing mapping.
function actionToValue(action: Action): string {
  switch (action.type) {
    case "remap":
      return action.target;
    case "combo":
      return action.keys.join(",");
    case "text":
      return action.text;
    case "launch":
      return action.command;
    case "disabled":
      return "";
  }
}

function describeAction(action: Action): string {
  switch (action.type) {
    case "remap":
      return `remap -> ${action.target}`;
    case "combo":
      return `combo -> ${action.keys.join(" + ")}`;
    case "text":
      return `type "${action.text}"`;
    case "launch":
      return `launch "${action.command}"`;
    case "disabled":
      return "disabled";
  }
}

export function FlexikeyPanel() {
  const [profilesIndex, setProfilesIndex] = useState<ProfilesIndex | null>(null);
  const [profile, setProfile] = useState<Profile | null>(null);
  const [newProfileName, setNewProfileName] = useState("");
  const [sourceKey, setSourceKey] = useState("");
  const [actionKind, setActionKind] = useState<ActionKind>("text");
  const [actionValue, setActionValue] = useState("");
  const [engineRunning, setEngineRunning] = useState(false);
  const [capturing, setCapturing] = useState(false);
  const [status, setStatus] = useState("");

  async function reloadProfilesIndex() {
    try {
      const index = await invoke<ProfilesIndex>("list_flexikey_profiles");
      setProfilesIndex(index);
      return index;
    } catch (err) {
      setStatus(`Failed to load Flexikey profiles: ${String(err)}`);
      return null;
    }
  }

  useEffect(() => {
    reloadProfilesIndex();
    // Only run once on mount - subsequent reloads are triggered explicitly
    // after profile-mutating actions below.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function selectProfile(name: string) {
    try {
      const loaded = await invoke<Profile>("get_flexikey_profile", { name });
      setProfile(loaded);
      setSourceKey("");
      setActionValue("");
    } catch (err) {
      setStatus(`Failed to load profile "${name}": ${String(err)}`);
    }
  }

  async function createProfile() {
    const name = newProfileName.trim();
    if (!name) {
      setStatus("Enter a profile name first");
      return;
    }
    try {
      await invoke("save_flexikey_profile", { profile: { name, mappings: {} } satisfies Profile });
      setNewProfileName("");
      await reloadProfilesIndex();
      await selectProfile(name);
      setStatus(`Created profile "${name}"`);
    } catch (err) {
      setStatus(`Failed to create profile: ${String(err)}`);
    }
  }

  async function deleteProfile(name: string) {
    try {
      await invoke("delete_flexikey_profile", { name });
      if (profile?.name === name) {
        setProfile(null);
      }
      await reloadProfilesIndex();
      setStatus(`Deleted profile "${name}"`);
    } catch (err) {
      setStatus(`Failed to delete profile: ${String(err)}`);
    }
  }

  async function activateProfile(name: string) {
    try {
      await invoke("set_active_flexikey_profile", { name });
      await reloadProfilesIndex();
      setStatus(`"${name}" is now the active Flexikey profile. Restart the engine to apply it.`);
    } catch (err) {
      setStatus(`Failed to activate profile: ${String(err)}`);
    }
  }

  async function captureKey() {
    setCapturing(true);
    setStatus("Press the key you want to remap now...");
    try {
      const key = await invoke<string>("capture_next_key");
      setSourceKey(key);
      setStatus(`Captured ${key}`);
    } catch (err) {
      setStatus(`Failed to capture key: ${String(err)}`);
    } finally {
      setCapturing(false);
    }
  }

  async function addOrUpdateMapping() {
    if (!profile) {
      setStatus("Select or create a profile first");
      return;
    }
    const key = sourceKey.trim();
    if (!key) {
      setStatus("Set a source key first (type it or use Capture)");
      return;
    }
    const updated: Profile = {
      ...profile,
      mappings: { ...profile.mappings, [key]: buildAction(actionKind, actionValue) },
    };
    try {
      await invoke("save_flexikey_profile", { profile: updated });
      setProfile(updated);
      setStatus(`Saved mapping for ${key}`);
    } catch (err) {
      setStatus(`Failed to save mapping: ${String(err)}`);
    }
  }

  async function removeMapping(key: string) {
    if (!profile) return;
    const mappings = { ...profile.mappings };
    delete mappings[key];
    const updated: Profile = { ...profile, mappings };
    try {
      await invoke("save_flexikey_profile", { profile: updated });
      setProfile(updated);
      if (sourceKey === key) {
        setSourceKey("");
        setActionValue("");
      }
      setStatus(`Removed mapping for ${key}`);
    } catch (err) {
      setStatus(`Failed to remove mapping: ${String(err)}`);
    }
  }

  function editMapping(key: string, action: Action) {
    setSourceKey(key);
    setActionKind(action.type);
    setActionValue(actionToValue(action));
  }

  async function startEngine() {
    try {
      await invoke("start_flexikey_engine");
      setEngineRunning(true);
      setStatus("Flexikey engine started - physical keyboard is now grabbed.");
    } catch (err) {
      setStatus(`Failed to start Flexikey engine: ${String(err)}`);
    }
  }

  async function stopEngine() {
    try {
      await invoke("stop_flexikey_engine");
      setEngineRunning(false);
      setStatus("Flexikey engine stopped - keyboard released.");
    } catch (err) {
      setStatus(`Failed to stop Flexikey engine: ${String(err)}`);
    }
  }

  const mappingEntries = profile ? Object.entries(profile.mappings) : [];

  return (
    <div style={{ display: "flex", gap: "1.5rem", flexWrap: "wrap" }}>
      <section style={{ minWidth: "220px" }}>
        <h3>Profiles (max 12)</h3>
        <p style={{ opacity: 0.8, margin: "0 0 0.5rem" }}>
          Active: {profilesIndex?.active_profile ?? "none"}
        </p>
        <ul style={{ listStyle: "none", padding: 0, margin: 0 }}>
          {(profilesIndex?.profiles ?? []).map((name) => (
            <li
              key={name}
              style={{
                display: "flex",
                alignItems: "center",
                gap: "0.4rem",
                padding: "0.2rem 0",
                fontWeight: profile?.name === name ? "bold" : "normal",
              }}
            >
              <button onClick={() => selectProfile(name)}>{name}</button>
              {profilesIndex?.active_profile === name && <span title="Active profile">*</span>}
              <button onClick={() => activateProfile(name)}>Set Active</button>
              <button onClick={() => deleteProfile(name)}>Delete</button>
            </li>
          ))}
        </ul>
        <div style={{ display: "flex", gap: "0.4rem", marginTop: "0.5rem" }}>
          <input
            type="text"
            placeholder="New profile name"
            value={newProfileName}
            onChange={(e) => setNewProfileName(e.target.value)}
          />
          <button onClick={createProfile}>New</button>
        </div>
        <div style={{ marginTop: "1rem" }}>
          {engineRunning ? (
            <button onClick={stopEngine}>Stop Flexikey Engine</button>
          ) : (
            <button onClick={startEngine}>Start Flexikey Engine</button>
          )}
        </div>
      </section>

      <section style={{ minWidth: "320px", flex: 1 }}>
        <h3>Mappings{profile ? ` for "${profile.name}"` : ""}</h3>
        {!profile && <p style={{ opacity: 0.8 }}>Select or create a profile to edit its mappings.</p>}
        {profile && (
          <>
            <ul style={{ listStyle: "none", padding: 0, margin: "0 0 1rem" }}>
              {mappingEntries.length === 0 && <li style={{ opacity: 0.8 }}>No mappings yet.</li>}
              {mappingEntries.map(([key, action]) => (
                <li
                  key={key}
                  style={{ display: "flex", alignItems: "center", gap: "0.5rem", padding: "0.2rem 0" }}
                >
                  <code>{key}</code>
                  <span>&rarr; {describeAction(action)}</span>
                  <button onClick={() => editMapping(key, action)}>Edit</button>
                  <button onClick={() => removeMapping(key)}>Remove</button>
                </li>
              ))}
            </ul>

            <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem", maxWidth: "420px" }}>
              <label>
                Source key:{" "}
                <input
                  type="text"
                  placeholder="e.g. KEY_F13"
                  value={sourceKey}
                  onChange={(e) => setSourceKey(e.target.value)}
                />
                <button onClick={captureKey} disabled={capturing} style={{ marginLeft: "0.4rem" }}>
                  {capturing ? "Press a key..." : "Capture next key"}
                </button>
              </label>

              <label>
                Action:{" "}
                <select value={actionKind} onChange={(e) => setActionKind(e.target.value as ActionKind)}>
                  {ACTION_KINDS.map((kind) => (
                    <option key={kind} value={kind}>
                      {kind}
                    </option>
                  ))}
                </select>
              </label>

              {actionKind !== "disabled" && (
                <label>
                  Value:{" "}
                  <input
                    type="text"
                    placeholder={ACTION_VALUE_HINT[actionKind]}
                    value={actionValue}
                    onChange={(e) => setActionValue(e.target.value)}
                    style={{ width: "100%" }}
                  />
                </label>
              )}

              <div>
                <button onClick={addOrUpdateMapping}>Add / Update Mapping</button>
              </div>
            </div>
          </>
        )}
      </section>

      {status && <p style={{ width: "100%", color: "#4CAF50" }}>{status}</p>}
    </div>
  );
}

export default FlexikeyPanel;
