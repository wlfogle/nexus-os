import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getConfig, humanBytes, humanDate } from "./api";

interface Match {
  repo_role: string;
  repo: string; path: string; relation: string; score: number;
  evidence: string[]; citations: { path: string; line: number; excerpt: string }[];
}
interface Finding {
  path: string; root: string; sha256: string; size: number; modified: number;
  protected_tree: string | null; kind: string; disposition: string;
  ideas: string[]; cautions: string[]; matches: Match[];
}
interface Report {
  finished_at: number;
  assessments?: Record<string, Assessment>;
  repositories: string[]; protected_trees: string[]; findings: Finding[];
  skipped: Record<string, number>; warnings: string[]; limited: boolean; cancelled: boolean;
}
interface Status {
  running: boolean; progress: { phase: string; visited: number; indexed: number; compared: number };
  report: Report | null; error: string | null;
}
interface Preview { source: string; counterpart: string; diff: string; truncated: boolean }
interface Assessment {
  purpose: string;
  ideas: { behavior: string; status: string; evidence_ids: number[]; adaptation: string; verification: string;
    proofs?: { evidence_id: number; quote: string }[] }[];
  assumptions: string[]; unknowns: string[];
  evidence: { id: number; path: string; body: string }[];
}

export default function Comparison() {
  const [roots, setRoots] = useState("");
  const [vault, setVault] = useState("");
  const [status, setStatus] = useState<Status | null>(null);
  const [error, setError] = useState("");
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState("");
  const [repoFilter, setRepoFilter] = useState("");
  const [selected, setSelected] = useState<number | null>(null);
  const [counterpart, setCounterpart] = useState<number | null>(null);
  const [preview, setPreview] = useState<Preview | null>(null);
  const [assessment, setAssessment] = useState<Assessment | null>(null);
  const [busy, setBusy] = useState(false);
  const [repo, setRepo] = useState("");
  const [relative, setRelative] = useState("");
  const [reason, setReason] = useState("");
  const [message, setMessage] = useState("");
  const [maxFiles, setMaxFiles] = useState(10000);
  const [reportPath, setReportPath] = useState("");
  const [savedReports, setSavedReports] = useState<string[]>([]);
  const report = status?.report;
  const finding = selected === null ? undefined : report?.findings[selected];

  useEffect(() => {
    getConfig().then(c => { setRoots(c.roots.join("\n")); setVault(c.vault); }).catch(e => setError(String(e)));
    invoke<string[]>("saved_comparison_reports").then(paths => {
      setSavedReports(paths); setReportPath(paths[0] ?? "");
    }).catch(e => setError(String(e)));
    invoke<Status>("comparison_status").then(setStatus).catch(e => setError(String(e)));
  }, []);
  useEffect(() => {
    if (!status?.running) return;
    const timer = window.setInterval(() => {
      invoke<Status>("comparison_status").then(setStatus).catch(e => setError(String(e)));
    }, 1500);
    return () => window.clearInterval(timer);
  }, [status?.running]);
  useEffect(() => {
    if (selected === null) return;
    let active = true;
    setPreview(null); setError("");
    invoke<Preview>("comparison_preview", { index: selected, counterpart })
      .then(p => { if (active) setPreview(p); })
      .catch(e => { if (active) setError(String(e)); });
    return () => { active = false; };
  }, [selected, counterpart]);
  const rows = useMemo(() => report?.findings.map((f, index) => ({ f, index })).filter(({ f }) =>
    (!query || f.path.toLowerCase().includes(query.toLowerCase()) || f.ideas.join(" ").toLowerCase().includes(query.toLowerCase())) &&
    (!kind || f.kind === kind) && (!repoFilter || f.matches.some(m => m.repo === repoFilter))
  ) ?? [], [report, query, kind, repoFilter]);

  const start = async () => {
    try {
      setError(""); setSelected(null); setPreview(null); setAssessment(null); setMessage("");
      await invoke("start_comparison", { options: {
        roots: roots.split("\n").map(s => s.trim()).filter(Boolean), vault,
        max_files: maxFiles, max_repo_files: 30000, max_bytes: 131072,
      } });
      setStatus(await invoke<Status>("comparison_status"));
    } catch (e) { setError(String(e)); }
  };
  const select = (index: number) => {
    setSelected(index); setCounterpart(report!.findings[index].matches.length ? 0 : null);
    setAssessment(report?.assessments?.[report.findings[index].path] ?? null); setRepo(""); setReason(""); setMessage("");
    const f = report!.findings[index];
    setRelative(f.path.startsWith(f.root + "/") ? f.path.slice(f.root.length + 1) : f.path.split("/").pop()!);
  };
  const assess = async () => {
    if (selected === null) return;
    setBusy(true); setError("");
    try { setAssessment(await invoke<Assessment>("assess_ideas", { index: selected })); }
    catch (e) { setError(String(e)); } finally { setBusy(false); }
  };
  const queue = async () => {
    if (selected === null) return;
    try {
      setError("");
      const id = await invoke<number>("queue_reference", { index: selected, repo, relative, reason });
      setMessage(`Reference proposal ${id} is pending in Review. Nothing has moved.`);
    } catch (e) { setError(String(e)); }
  };

  return <>
    <div className="section">
      <h2>Study the history, recover the ideas</h2>
      <p>No date cutoff. Compare local scripts and documents against tracked repository files.
        Similarity is a lead—not proof that a feature is implemented. Old scripts are never executed.</p>
      <div className="grid2">
        <div className="field"><label>Source roots — one absolute directory per line</label>
          <textarea className="text" rows={3} value={roots} onChange={e => setRoots(e.target.value)} disabled={status?.running} /></div>
        <div className="field"><label>Repository vault</label>
          <input className="text" value={vault} onChange={e => setVault(e.target.value)} disabled={status?.running} />
          <label>Maximum source findings (partial coverage is reported)</label>
          <input className="text" type="number" min={1} max={100000} value={maxFiles}
            onChange={e => setMaxFiles(Number(e.target.value))} /></div>
      </div>
      <details><summary>Open a saved private comparison report</summary>
        <select className="text" value={reportPath} onChange={e => setReportPath(e.target.value)}>
          <option value="">Choose a saved report</option>
          {savedReports.map(p => <option key={p} value={p}>{p}</option>)}
        </select>
        <input className="text grow" value={reportPath} onChange={e => setReportPath(e.target.value)} aria-label="Private comparison report path" />
        <button className="btn" disabled={status?.running || !reportPath} onClick={async () => {
          try {
            await invoke("load_comparison_report", { path: reportPath });
            setSelected(null); setAssessment(null); setPreview(null);
            setStatus(await invoke<Status>("comparison_status")); setError("");
          } catch (e) { setError(String(e)); }
        }}>Open report (read-only)</button>
      </details>
      <div className="row">
        <button className="btn primary" disabled={!roots || !vault || status?.running} onClick={start}>Start read-only comparison</button>
        {report && <button className="btn" onClick={async () => {
          try {
            const path = await invoke<string>("save_comparison_report");
            setMessage(`Private report saved: ${path}`); setReportPath(path);
            setSavedReports(await invoke<string[]>("saved_comparison_reports"));
          } catch (e) { setError(String(e)); }
        }}>Save report & assessments</button>}
        {status?.running && <button className="btn danger" onClick={() => invoke("stop_comparison").catch(e => setError(String(e)))}>Cancel comparison</button>}
        <span className="small muted">{status?.progress.phase}: {status?.progress.indexed ?? 0} indexed · {status?.progress.compared ?? 0} compared</span>
      </div>
    </div>
    {(error || status?.error) && <div className="err">{error || status?.error}</div>}
    {message && <p>{message}</p>}
    {report && <>
      <div className="section">
        <h2>{report.findings.length} findings · {report.repositories.length} repositories</h2>
        <p className="small muted">Snapshot: {new Date(report.finished_at * 1000).toLocaleString()}. Previews recheck file contents; assessment conclusions remain advisory.</p>
        {(report.limited || report.cancelled) && <div className="err">Partial comparison: {report.cancelled ? "cancelled" : "a coverage limit was reached"}. Do not interpret missing matches as proof of absence.</div>}
        <details><summary>Coverage, exclusions and protected source trees</summary>
          <pre>{JSON.stringify(report.skipped, null, 2)}</pre>
          {report.protected_trees.map(p => <div className="path" key={p}>Protected: {p}</div>)}
          {report.warnings.map((w, i) => <div className="small" key={i}>{w}</div>)}
          <p>Hidden directories, secrets, app state, generated files, media and reference archives are excluded.
            Git-marked source trees are compared but cannot be moved file-by-file.</p>
        </details>
        <div className="filters">
          <input className="text grow" placeholder="Filter paths or extracted purpose comments" value={query} onChange={e => setQuery(e.target.value)} />
          <select className="text" value={kind} onChange={e => setKind(e.target.value)}>
            <option value="">All finding types</option>
            {["identical", "divergent_copy", "related_evidence", "ambiguous", "unmatched"].map(k => <option key={k}>{k}</option>)}
          </select>
          <select className="text" value={repoFilter} onChange={e => setRepoFilter(e.target.value)}>
            <option value="">All repositories</option>
            {report.repositories.map(p => <option key={p} value={p}>{p.split("/").pop()}</option>)}
          </select>
        </div>
        <p className="small muted">{rows.length} matches; first 200 shown. Narrow filters to inspect more.</p>
        <table><thead><tr><th>Source</th><th>Evidence</th><th>Review</th></tr></thead><tbody>
          {rows.slice(0, 200).map(({ f, index }) => <tr key={f.path}>
            <td><div className="path">{f.path}</div><span className="small muted">{humanBytes(f.size)} · modified {humanDate(f.modified)} (context only)</span>
              {f.protected_tree && <div className="tag">Protected source tree</div>}</td>
            <td>{f.kind}<div className="path">{f.matches[0]?.path ?? "No counterpart found within coverage"}</div></td>
            <td><button className="btn tiny" disabled={busy} onClick={() => select(index)}>Study</button></td>
          </tr>)}
        </tbody></table>
      </div>
      {finding && <div className="section">
        <h2>Source review</h2><div className="path">{finding.path}</div>
        <p>{finding.disposition}</p>
        <ul>{finding.cautions.map(c => <li key={c}>{c}</li>)}</ul>
        <h3>Purpose comments (verbatim, not a semantic conclusion)</h3>
        <ul>{finding.ideas.map((idea, i) => <li key={i}>{idea}</li>)}</ul>
        <select className="text" value={counterpart ?? ""} onChange={e => setCounterpart(e.target.value === "" ? null : Number(e.target.value))}>
          <option value="">Source only</option>
          {finding.matches.map((m, i) => <option key={m.path} value={i}>{m.relation}: {m.path}</option>)}
        </select>
        {counterpart !== null && finding.matches[counterpart] && <div>
          <ul>{finding.matches[counterpart].evidence.map(e => <li key={e}>{e}</li>)}</ul>
          <p>Repository role: {finding.matches[counterpart].repo_role || "Unclassified in this saved report"}</p>
          {finding.matches[counterpart].citations.map((c, i) => <div key={i}>
            <div className="path">{c.path}:{c.line}</div><pre>{c.excerpt}</pre></div>)}
        </div>}
        {preview && <>
          <details open><summary>Side-by-side content</summary><div className="grid2">
            <pre className="comparison-code">{preview.source}</pre><pre className="comparison-code">{preview.counterpart}</pre>
          </div></details>
          <details><summary>Unified difference {preview.truncated ? "(truncated, not an applicable patch)" : ""}</summary>
            <pre className="comparison-code">{preview.diff}</pre></details>
        </>}
        <button className="btn" disabled={busy} onClick={assess}>{busy ? "Studying with local model…" : "Assess useful ideas with local Ollama"}</button>
        <p className="small muted">Explicit local-model request. Advisory only; it cannot edit code or approve archives.</p>
        {assessment && <div>
          <h3>{assessment.purpose}</h3>
          {assessment.ideas.map((idea, i) => <div className="card" key={i}>
            <strong>{idea.behavior}</strong> <span className="tag">Advisory: {idea.status}</span>
            <p>{idea.adaptation}</p><p>Verification required: {idea.verification}</p>
            <p>Evidence: {idea.evidence_ids.map(id => `[${id}] ${assessment.evidence[id]?.path}`).join("; ")}</p>
            {idea.proofs?.map((proof, index) => <blockquote key={index}>
              [{proof.evidence_id}] {proof.quote}
            </blockquote>)}
          </div>)}
          <h3>Assumptions and unresolved questions</h3>
          <ul>{[...assessment.assumptions, ...assessment.unknowns].map((s, i) => <li key={i}>{s}</li>)}</ul>
          {assessment.evidence.map(e => <details key={e.id}><summary>[{e.id}] {e.path}</summary><pre className="comparison-code">{e.body}</pre></details>)}
        </div>}
        {!finding.protected_tree && <div>
          <h3>Preserve original as a repository reference</h3>
          <p>Choose ownership explicitly. Queueing does not move or approve anything.</p>
          <div className="field"><label>Repository</label><select className="text" value={repo} onChange={e => setRepo(e.target.value)}>
            <option value="">Choose owning repository</option>
            {report.repositories.map(p => <option key={p} value={p}>{p}</option>)}
          </select></div>
          <div className="field"><label>Path beneath archive/reference/</label><input className="text" value={relative} onChange={e => setRelative(e.target.value)} /></div>
          <div className="field"><label>Reason / useful behavior to retain</label><textarea className="text" value={reason} onChange={e => setReason(e.target.value)} /></div>
          <div className="path">{repo ? `${repo}/archive/reference/${relative}` : "No repository selected"}</div>
          <button className="btn" disabled={!repo || !relative || !reason.trim()} onClick={queue}>Queue reference proposal for review</button>
          {message && <p>{message}</p>}
        </div>}
      </div>}
    </>}
  </>;
}
