/**
 * Key Bindings tab: hosts Flexikey profile management.
 *
 * ============================ INTEGRATION POINT ============================
 * flexikey-agent: once `hw/src/flexikey.rs` and its Tauri commands land
 * (`list_flexikey_profiles`, `get_flexikey_profile`, `save_flexikey_profile`,
 * `delete_flexikey_profile`, `set_active_flexikey_profile`,
 * `capture_next_key`, `start_flexikey_engine`, `stop_flexikey_engine` - see
 * CONTRACT.md's "flexikey-agent" section for exact signatures and the
 * profile JSON shape), replace the placeholder `<div className="placeholder-box">`
 * block below with your real component, e.g.:
 *
 *   import { FlexikeyProfileManager } from "./FlexikeyProfileManager";
 *   ...
 *   <FlexikeyProfileManager />
 *
 * That component should own: listing/switching/creating/deleting profiles,
 * editing per-key mappings (remap/combo/text/launch/disabled actions), the
 * "capture next key" flow, and the flexikey engine start/stop toggle. It can
 * mount directly inside the `panel-card` section already scaffolded here, or
 * replace it outright - whichever fits the real UI better.
 * =============================================================================
 */
export function KeyBindingsTab() {
  return (
    <div className="tab-panel">
      <section className="panel-card">
        <h3>Key Bindings</h3>
        <p className="panel-hint">
          Flexikey profile management (remap, combo, text, and launch actions per
          key) is owned by flexikey-agent and will be mounted here once its crate
          module and Tauri commands land - see the integration point comment in
          this file.
        </p>
        <div className="placeholder-box">
          <span className="placeholder-icon">⌨</span>
          <p>Flexikey profile manager - coming soon.</p>
        </div>
      </section>
    </div>
  );
}
