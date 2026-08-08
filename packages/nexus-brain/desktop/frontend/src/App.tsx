import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import * as api from "./api";
import type {
  ActionRow,
  CatalogRow,
  Config,
  DupGroup,
  Health,
  Hit,
  Progress,
  RepoInfo,
  Stats,
  TagModel,
} from "./api";

type View =
  | "dashboard"
  | "catalog"
  | "review"
  | "search"
  | "stale"
  | "repos"
  | "dupes"
  | "history"
  | "settings";

const VIEWS: { id: View; label: string }[] = [
  { id: "dashboard", label: "Dashboard" },
  { id: "search", label: "Search" },
  { id: "catalog", label: "Catalog" },
  { id: "review", label: "Review" },
  { id: "stale", label: "Stale" },
  { id: "dupes", label: "Duplicates" },
  { id: "repos", label: "Repos" },
  { id: "history", label: "History" },
  { id: "settings", label: "Settings" },
];

/* ------------------------------------------------------------------ shell */

export default function App() {
  const [view, setView] = useState<View>("dashboard");
  const [stats, setStats] = useState<Stats | null>(null);
  const [progress, setProgress] = useState<Progress | null>(null);
  const [healthInfo, setHealthInfo] = useState<Health | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refreshStats = useCallback(async () => {
    try {
      setStats(await api.getStats());
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    refreshStats();
    api.health().then(setHealthInfo).catch(() => setHealthInfo(null));

    const un = api.onProgress((p) => {
      setProgress(p);
      setStats(p.stats);
    });
    const timer = setInterval(refreshStats, 5000);
    return () => {
      un.then((f) => f()).catch(() => {});
      clearInterval(timer);
    };
  }, [refreshStats]);

  const running = progress?.running ?? healthInfo?.running ?? false;

  const start = async () => {
    setError(null);
    try {
      await api.startPipeline();
    } catch (e) {
      setError(String(e));
    }
  };
  const stop = async () => {
    try {
      await api.stopPipeline();
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="brand">
          <span className="dot" />
          Librarian
        </div>
        <nav className="nav">
          {VIEWS.map((v) => (
            <button
              key={v.id}
              className={view === v.id ? "active" : ""}
              onClick={() => setView(v.id)}
            >
              <span>{v.label}</span>
              {v.id === "review" && (stats?.pending_actions ?? 0) > 0 && (
                <span className="count">{stats!.pending_actions}</span>
              )}
              {v.id === "dupes" && (stats?.duplicate_groups ?? 0) > 0 && (
                <span className="count">{stats!.duplicate_groups}</span>
              )}
              {v.id === "repos" && (stats?.repos ?? 0) > 0 && (
                <span className="count">{stats!.repos}</span>
              )}
            </button>
          ))}
        </nav>
        <div className="sidebar-foot">
          <div>
            <span
              className={`led ${healthInfo?.ollama_up ? "up" : "down"}`}
            />
            Ollama {healthInfo?.ollama_up ? "up" : "down"}
          </div>
          {healthInfo?.ollama_up && (
            <div style={{ marginTop: 3 }}>
              {healthInfo.models_installed} models
            </div>
          )}
        </div>
      </aside>

      <main className="main">
        <header className="topbar">
          <h1>{VIEWS.find((v) => v.id === view)?.label}</h1>
          {progress && (
            <span className="muted small">
              {progress.phase}: {progress.detail}
            </span>
          )}
          <span className="spacer" />
          {running ? (
            <button className="btn danger" onClick={stop}>
              Pause
            </button>
          ) : (
            <button className="btn primary" onClick={start}>
              Run pipeline
            </button>
          )}
        </header>

        <div className="content">
          {error && <div className="err">{error}</div>}
          {view === "dashboard" && <Dashboard stats={stats} health={healthInfo} />}
          {view === "search" && <Search />}
          {view === "catalog" && <Catalog />}
          {view === "review" && <Review onChange={refreshStats} />}
          {view === "stale" && <Stale />}
          {view === "dupes" && <Duplicates />}
          {view === "repos" && <Repos />}
          {view === "history" && <History />}
          {view === "settings" && <Settings />}
        </div>
      </main>
    </div>
  );
}

/* -------------------------------------------------------------- dashboard */

function Dashboard({ stats, health }: { stats: Stats | null; health: Health | null }) {
  if (!stats) return <div className="empty">Loading…</div>;

  // Progress is measured against files that are actually candidates for
  // interpretation. Counting skipped binaries in the denominator made the bar
  // look almost complete before a single file had been read.
  const eligible = Math.max(stats.files_present - stats.skipped, 1);
  const pct = (n: number) => `${Math.min((n / eligible) * 100, 100)}%`;
  const onlyText = Math.max(stats.with_text - stats.embedded, 0);
  const onlyEmbedded = Math.max(stats.embedded - stats.interpreted, 0);
  const notYetRead = Math.max(eligible - stats.with_text, 0);

  return (
    <>
      <div className="cards">
        <Card
          k="Files catalogued"
          v={stats.files_present.toLocaleString()}
          sub={`${stats.skipped.toLocaleString()} skipped as binary or oversized`}
        />
        <Card
          k="Interpreted"
          v={stats.interpreted.toLocaleString()}
          sub={`${Math.round((stats.interpreted / eligible) * 100)}% of ${eligible.toLocaleString()} eligible`}
        />
        <Card k="Git repos" v={stats.repos.toLocaleString()} />
        <Card
          k="Loose files"
          v={stats.loose_files.toLocaleString()}
          sub={`${api.humanBytes(stats.bytes_loose)} owned by no repo`}
        />
        <Card k="Duplicate groups" v={stats.duplicate_groups.toLocaleString()} />
        <Card k="Awaiting review" v={stats.pending_actions.toLocaleString()} />
      </div>

      <div className="section">
        <h2>Pipeline progress</h2>
        <div className="bar">
          <span className="s3" style={{ width: pct(stats.interpreted) }} />
          <span className="s2" style={{ width: pct(onlyEmbedded) }} />
          <span className="s1" style={{ width: pct(onlyText) }} />
          <span className="s0" style={{ width: pct(notYetRead) }} />
        </div>
        <div className="legend">
          <span>
            <i style={{ background: "var(--green)" }} />
            interpreted {stats.interpreted.toLocaleString()}
          </span>
          <span>
            <i style={{ background: "var(--accent)" }} />
            embedded {onlyEmbedded.toLocaleString()}
          </span>
          <span>
            <i style={{ background: "#47739b" }} />
            has text {onlyText.toLocaleString()}
          </span>
          <span>
            <i style={{ background: "#33415a" }} />
            not read yet {notYetRead.toLocaleString()}
          </span>
          <span className="muted">
            ({stats.skipped.toLocaleString()} skipped, excluded)
          </span>
        </div>
      </div>

      {health && (
        <div className="section">
          <h2>Backend</h2>
          <table>
            <tbody>
              <tr>
                <td className="muted">Ollama</td>
                <td className="path">
                  {health.ollama_url} — {health.ollama_up ? "reachable" : "unreachable"}
                </td>
              </tr>
              <tr>
                <td className="muted">Models installed</td>
                <td>{health.models_installed}</td>
              </tr>
              <tr>
                <td className="muted">Catalog</td>
                <td className="path">{health.db_path}</td>
              </tr>
            </tbody>
          </table>
        </div>
      )}
    </>
  );
}

function Card({ k, v, sub }: { k: string; v: string; sub?: string }) {
  return (
    <div className="card">
      <div className="k">{k}</div>
      <div className="v">{v}</div>
      {sub && <div className="sub">{sub}</div>}
    </div>
  );
}

/* ----------------------------------------------------------------- search */

function Search() {
  const [q, setQ] = useState("");
  const [hits, setHits] = useState<Hit[]>([]);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [ran, setRan] = useState(false);

  const go = async () => {
    if (!q.trim()) return;
    setBusy(true);
    setErr(null);
    try {
      setHits(await api.searchCatalog(q, 40));
      setRan(true);
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <>
      <div className="filters">
        <input
          className="text grow"
          placeholder="Ask for anything you have written — concepts work, not just exact words"
          value={q}
          onChange={(e) => setQ(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && go()}
        />
        <button className="btn primary" onClick={go} disabled={busy}>
          {busy ? "Searching…" : "Search"}
        </button>
      </div>
      {err && <div className="err">{err}</div>}
      {hits.length === 0 && ran && !busy && (
        <div className="empty">Nothing matched.</div>
      )}
      {hits.map((h) => (
        <div key={h.file_id} className="card" style={{ marginBottom: 10 }}>
          <div className="row wrap">
            <span className="name">{h.title || h.name}</span>
            {h.canonical && <span className="tag current">canonical</span>}
            {h.status && <span className={`tag ${h.status}`}>{h.status}</span>}
            {h.repo && <span className="tag repo">{h.repo}</span>}
            <span className="spacer" />
            <span className="muted small">{api.humanDate(h.mtime)}</span>
          </div>
          <div className="path">{h.path}</div>
          {h.summary && <div className="snippet">{h.summary}</div>}
          {h.snippet && <div className="snippet">{h.snippet}</div>}
          {h.duplicates.length > 0 && (
            <div className="dupes">
              {h.duplicates.length} other identical{" "}
              {h.duplicates.length === 1 ? "copy" : "copies"}:{" "}
              {h.duplicates.slice(0, 3).join("  ·  ")}
              {h.duplicates.length > 3 && " …"}
            </div>
          )}
        </div>
      ))}
    </>
  );
}

/* ---------------------------------------------------------------- catalog */

function Catalog() {
  const [rows, setRows] = useState<CatalogRow[]>([]);
  const [cls, setCls] = useState("");
  const [status, setStatus] = useState("");
  const [loose, setLoose] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    api
      .listCatalog({
        class: cls || null,
        status: status || null,
        looseOnly: loose,
        limit: 300,
        offset: 0,
      })
      .then(setRows)
      .catch((e) => setErr(String(e)));
  }, [cls, status, loose]);

  return (
    <>
      <div className="filters">
        <select className="text" value={cls} onChange={(e) => setCls(e.target.value)}>
          <option value="">all classes</option>
          {["doc", "document", "script", "code", "config", "data", "web", "image"].map(
            (c) => (
              <option key={c} value={c}>
                {c}
              </option>
            )
          )}
        </select>
        <select
          className="text"
          value={status}
          onChange={(e) => setStatus(e.target.value)}
        >
          <option value="">any status</option>
          {["current", "stale", "superseded", "reference", "junk"].map((s) => (
            <option key={s} value={s}>
              {s}
            </option>
          ))}
        </select>
        <label className="row small muted">
          <input
            type="checkbox"
            checked={loose}
            onChange={(e) => setLoose(e.target.checked)}
          />
          only files no repo owns
        </label>
        <span className="spacer" />
        <span className="muted small">{rows.length} shown</span>
      </div>
      {err && <div className="err">{err}</div>}
      <FileTable rows={rows} />
    </>
  );
}

function FileTable({ rows }: { rows: CatalogRow[] }) {
  if (rows.length === 0) {
    return <div className="empty">Nothing here yet — run the pipeline.</div>;
  }
  return (
    <table>
      <thead>
        <tr>
          <th style={{ width: "30%" }}>File</th>
          <th>What the model decided it is</th>
          <th style={{ width: 110 }}>Status</th>
          <th style={{ width: 90 }}>Confidence</th>
          <th style={{ width: 90 }}>Modified</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((r) => (
          <tr key={r.id}>
            <td>
              <div className="name">{r.title || r.name}</div>
              <div className="path">{r.path}</div>
              {r.repo && <span className="tag repo">{r.repo}</span>}
            </td>
            <td>
              {r.purpose || <span className="muted">not interpreted yet</span>}
              {r.summary && <div className="snippet">{r.summary}</div>}
              {r.topics.length > 0 && (
                <div style={{ marginTop: 4 }}>
                  {r.topics.slice(0, 5).map((t) => (
                    <span key={t} className="tag">
                      {t}
                    </span>
                  ))}
                </div>
              )}
            </td>
            <td>
              {r.kind === "secret" && <span className="tag secret">SECRET</span>}
              {r.status && <span className={`tag ${r.status}`}>{r.status}</span>}
            </td>
            <td>
              <span
                className={`conf ${
                  r.confidence >= 0.8 ? "hi" : r.confidence < 0.55 ? "lo" : ""
                }`}
              >
                {r.confidence ? r.confidence.toFixed(2) : "-"}
              </span>
              {r.model && <div className="path">{r.model}</div>}
            </td>
            <td className="muted small">{api.humanDate(r.mtime)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

/* ----------------------------------------------------------------- review */

function Review({ onChange }: { onChange: () => void }) {
  const [rows, setRows] = useState<ActionRow[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const load = useCallback(() => {
    api.listReview(300).then(setRows).catch((e) => setErr(String(e)));
  }, []);
  useEffect(load, [load]);

  const decide = async (id: number, approve: boolean) => {
    try {
      await api.decideAction(id, approve);
      setRows((r) => r.filter((x) => x.id !== id));
      onChange();
    } catch (e) {
      setErr(String(e));
    }
  };

  const applyAll = async () => {
    setBusy(true);
    try {
      const r = await api.applyApproved();
      setErr(null);
      alert(`Applied ${r.applied}, failed ${r.failed}, skipped ${r.skipped}`);
      load();
      onChange();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <>
      <div className="filters">
        <span className="muted small">
          {rows.length} action{rows.length === 1 ? "" : "s"} the model was not
          confident enough to apply on its own
        </span>
        <span className="spacer" />
        <button className="btn primary" onClick={applyAll} disabled={busy}>
          {busy ? "Applying…" : "Apply approved"}
        </button>
      </div>
      {err && <div className="err">{err}</div>}
      {rows.length === 0 ? (
        <div className="empty">Nothing waiting on you.</div>
      ) : (
        <table>
          <thead>
            <tr>
              <th style={{ width: "38%" }}>Move</th>
              <th>Why</th>
              <th style={{ width: 80 }}>Conf.</th>
              <th style={{ width: 150 }}>Decide</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((a) => (
              <tr key={a.id}>
                <td>
                  <div className="path">{a.src}</div>
                  <div className="muted small">↓ {a.category}</div>
                  <div className="path">{a.dest}</div>
                </td>
                <td>{a.reason}</td>
                <td>
                  <span className="conf lo">{a.confidence.toFixed(2)}</span>
                </td>
                <td>
                  <div className="row">
                    <button
                      className="btn tiny"
                      onClick={() => decide(a.id, true)}
                    >
                      Approve
                    </button>
                    <button
                      className="btn tiny danger"
                      onClick={() => decide(a.id, false)}
                    >
                      Reject
                    </button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </>
  );
}

/* ------------------------------------------------------------------ stale */

function Stale() {
  const [rows, setRows] = useState<Hit[]>([]);
  const [err, setErr] = useState<string | null>(null);
  useEffect(() => {
    api.listStale(300).then(setRows).catch((e) => setErr(String(e)));
  }, []);

  return (
    <>
      <div className="filters">
        <span className="muted small">
          Content the model judged out of date — the reason you keep referencing
          old information
        </span>
      </div>
      {err && <div className="err">{err}</div>}
      {rows.length === 0 ? (
        <div className="empty">Nothing flagged stale.</div>
      ) : (
        <table>
          <thead>
            <tr>
              <th style={{ width: "34%" }}>File</th>
              <th>Summary</th>
              <th style={{ width: 110 }}>Status</th>
              <th style={{ width: 90 }}>Modified</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((h) => (
              <tr key={h.file_id}>
                <td>
                  <div className="name">{h.title || h.name}</div>
                  <div className="path">{h.path}</div>
                </td>
                <td>{h.summary}</td>
                <td>
                  <span className={`tag ${h.status}`}>{h.status}</span>
                  {h.repo && <span className="tag repo">{h.repo}</span>}
                </td>
                <td className="muted small">
                  {api.humanDate(h.mtime)}
                  <div className="path">{api.ageDays(h.mtime)}d</div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </>
  );
}

/* ------------------------------------------------------------- duplicates */

function Duplicates() {
  const [groups, setGroups] = useState<DupGroup[]>([]);
  const [err, setErr] = useState<string | null>(null);
  useEffect(() => {
    api.listDuplicates(150).then(setGroups).catch((e) => setErr(String(e)));
  }, []);

  const reclaimable = useMemo(
    () => groups.reduce((a, g) => a + g.reclaimable, 0),
    [groups]
  );

  return (
    <>
      <div className="filters">
        <span className="muted small">
          {groups.length} groups · {api.humanBytes(reclaimable)} reclaimable by
          keeping one copy of each
        </span>
      </div>
      {err && <div className="err">{err}</div>}
      {groups.length === 0 ? (
        <div className="empty">No duplicates found.</div>
      ) : (
        groups.map((g) => (
          <div key={g.sha256} className="card" style={{ marginBottom: 10 }}>
            <div className="row">
              <strong>{g.count} identical copies</strong>
              <span className="muted small">
                {api.humanBytes(g.size)} each · {api.humanBytes(g.reclaimable)}{" "}
                reclaimable
              </span>
            </div>
            {g.paths.map((p) => (
              <div key={p} className="path">
                {p}
              </div>
            ))}
          </div>
        ))
      )}
    </>
  );
}

/* ------------------------------------------------------------------ repos */

function Repos() {
  const [rows, setRows] = useState<RepoInfo[]>([]);
  const [err, setErr] = useState<string | null>(null);
  useEffect(() => {
    api.listRepos().then(setRows).catch((e) => setErr(String(e)));
  }, []);

  return (
    <>
      {err && <div className="err">{err}</div>}
      {rows.length === 0 ? (
        <div className="empty">No repositories indexed yet.</div>
      ) : (
        <table>
          <thead>
            <tr>
              <th>Repository</th>
              <th style={{ width: 110 }}>Owner</th>
              <th style={{ width: 90 }}>Last commit</th>
              <th style={{ width: 200 }}>Local-only state</th>
              <th style={{ width: 110 }}>Recoverable</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.path}>
                <td>
                  <div className="name">{r.name}</div>
                  <div className="path">{r.path}</div>
                  {r.kind !== "repo" && <span className="tag">{r.kind}</span>}
                  {r.branch && <span className="tag">{r.branch}</span>}
                </td>
                <td className="muted">{r.owner}</td>
                <td className="muted small">{api.humanDate(r.last_commit)}</td>
                <td className="small">
                  {r.unpushed > 0 && (
                    <span className="tag stale">{r.unpushed} unpushed</span>
                  )}
                  {r.dirty > 0 && <span className="tag stale">{r.dirty} dirty</span>}
                  {r.untracked > 0 && (
                    <span className="tag">{r.untracked} untracked</span>
                  )}
                  {r.stashes > 0 && <span className="tag">{r.stashes} stash</span>}
                  {r.unpushed + r.dirty + r.untracked + r.stashes === 0 && (
                    <span className="muted">clean</span>
                  )}
                </td>
                <td>
                  {r.recoverable ? (
                    <span className="tag current">re-clonable</span>
                  ) : (
                    <span className="tag junk">local only</span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </>
  );
}

/* ---------------------------------------------------------------- history */

function History() {
  const [rows, setRows] = useState<ActionRow[]>([]);
  const [err, setErr] = useState<string | null>(null);

  const load = useCallback(() => {
    api.listHistory(300).then(setRows).catch((e) => setErr(String(e)));
  }, []);
  useEffect(load, [load]);

  const plans = useMemo(
    () => Array.from(new Set(rows.map((r) => r.plan_id))).sort((a, b) => b - a),
    [rows]
  );

  const undo = async (planId: number) => {
    try {
      const r = await api.undoPlan(planId);
      alert(`Reverted ${r.applied}, failed ${r.failed}`);
      load();
    } catch (e) {
      setErr(String(e));
    }
  };

  return (
    <>
      {err && <div className="err">{err}</div>}
      {rows.length === 0 ? (
        <div className="empty">Nothing has been moved yet.</div>
      ) : (
        plans.map((pid) => (
          <div key={pid} className="section">
            <div className="row">
              <h2 style={{ margin: 0 }}>Plan {pid}</h2>
              <span className="spacer" />
              <button className="btn tiny danger" onClick={() => undo(pid)}>
                Undo this plan
              </button>
            </div>
            <table>
              <tbody>
                {rows
                  .filter((r) => r.plan_id === pid)
                  .map((a) => (
                    <tr key={a.id}>
                      <td>
                        <div className="path">{a.src}</div>
                        <div className="path">→ {a.dest}</div>
                      </td>
                      <td style={{ width: 120 }}>
                        <span className="tag">{a.category}</span>
                      </td>
                    </tr>
                  ))}
              </tbody>
            </table>
          </div>
        ))
      )}
    </>
  );
}

/* --------------------------------------------------------------- settings */

function Settings() {
  const [cfg, setCfg] = useState<Config | null>(null);
  const [models, setModels] = useState<TagModel[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const first = useRef(true);

  useEffect(() => {
    api.getConfig().then(setCfg).catch((e) => setErr(String(e)));
    api.listModels().then(setModels).catch(() => setModels([]));
  }, []);

  useEffect(() => {
    if (first.current) {
      first.current = false;
      return;
    }
    setSaved(false);
  }, [cfg]);

  if (!cfg) return <div className="empty">Loading…</div>;

  const setModel = (k: keyof Config["models"], v: string) =>
    setCfg({ ...cfg, models: { ...cfg.models, [k]: v } });

  const save = async () => {
    try {
      await api.saveConfig(cfg);
      setSaved(true);
      setErr(null);
    } catch (e) {
      setErr(String(e));
    }
  };

  const ModelPick = ({
    k,
    label,
    hint,
  }: {
    k: keyof Config["models"];
    label: string;
    hint: string;
  }) => (
    <div className="field">
      <label>
        {label} — <span className="muted">{hint}</span>
      </label>
      <select
        className="text"
        value={cfg.models[k] as string}
        onChange={(e) => setModel(k, e.target.value)}
      >
        {models.length === 0 && (
          <option value={cfg.models[k] as string}>{cfg.models[k]}</option>
        )}
        {models.map((m) => (
          <option key={m.name} value={m.name}>
            {m.name}
          </option>
        ))}
      </select>
    </div>
  );

  return (
    <>
      {err && <div className="err">{err}</div>}
      <div className="section">
        <h2>Model routing — {models.length} local models available</h2>
        <div className="grid2">
          <ModelPick k="embed" label="Embeddings" hint="semantic index" />
          <ModelPick k="triage" label="Triage" hint="cheap first pass" />
          <ModelPick k="code" label="Code" hint="source, scripts, config" />
          <ModelPick k="docs" label="Documents" hint="notes and prose" />
          <ModelPick k="vision" label="Vision" hint="screenshots" />
          <ModelPick k="vision_escalate" label="Vision escalation" hint="dense images" />
          <ModelPick k="escalate" label="Escalation" hint="low-confidence retry" />
          <ModelPick k="escalate_max" label="Final escalation" hint="last resort" />
        </div>
        <div className="field">
          <label>
            Escalate when confidence is below{" "}
            <strong>{cfg.models.escalate_below.toFixed(2)}</strong>
          </label>
          <input
            type="range"
            min={0}
            max={1}
            step={0.05}
            value={cfg.models.escalate_below}
            onChange={(e) =>
              setCfg({
                ...cfg,
                models: {
                  ...cfg.models,
                  escalate_below: Number(e.target.value),
                },
              })
            }
          />
        </div>
      </div>

      <div className="section">
        <h2>Behaviour</h2>
        <div className="field">
          <label>
            Apply automatically at or above confidence{" "}
            <strong>{cfg.auto_apply_above.toFixed(2)}</strong> — anything lower
            waits in Review
          </label>
          <input
            type="range"
            min={0.5}
            max={1}
            step={0.05}
            value={cfg.auto_apply_above}
            onChange={(e) =>
              setCfg({ ...cfg, auto_apply_above: Number(e.target.value) })
            }
          />
        </div>
        <div className="field">
          <label>Scan roots</label>
          <input
            className="text"
            value={cfg.roots.join(", ")}
            onChange={(e) =>
              setCfg({
                ...cfg,
                roots: e.target.value.split(",").map((s) => s.trim()).filter(Boolean),
              })
            }
          />
        </div>
        <div className="field">
          <label>Repo vault</label>
          <input
            className="text"
            value={cfg.vault}
            onChange={(e) => setCfg({ ...cfg, vault: e.target.value })}
          />
        </div>
        <div className="field">
          <label>Managed library</label>
          <input
            className="text"
            value={cfg.library}
            onChange={(e) => setCfg({ ...cfg, library: e.target.value })}
          />
        </div>
        <div className="field">
          <label>Ollama URL</label>
          <input
            className="text"
            value={cfg.ollama_url}
            onChange={(e) => setCfg({ ...cfg, ollama_url: e.target.value })}
          />
        </div>
      </div>

      <div className="row">
        <button className="btn primary" onClick={save}>
          Save settings
        </button>
        {saved && <span className="muted small">Saved.</span>}
      </div>
    </>
  );
}
