import { FlexikeyPanel } from "../FlexikeyPanel";

/**
 * Key Bindings tab: hosts Flexikey profile management.
 *
 * flexikey-agent's `FlexikeyPanel` (profile list/create/delete/activate,
 * per-key mapping editor, capture-next-key, engine start/stop) mounted
 * here during integration - both were built in parallel worktrees against
 * CONTRACT.md and connect cleanly since neither side deviated from the
 * frozen command contract.
 */
export function KeyBindingsTab() {
  return (
    <div className="tab-panel">
      <section className="panel-card">
        <h3>Key Bindings</h3>
        <p className="panel-hint">
          Flexikey: remap, combo, text, and launch actions per key.
        </p>
        <FlexikeyPanel />
      </section>
    </div>
  );
}
