import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/* ------------------------------------------------------------------ types */

export interface Stats {
  files_total: number;
  files_present: number;
  scanned: number;
  extracted: number;
  with_text: number;
  embedded: number;
  interpreted: number;
  /** Deliberately not read: binary, media, oversized. */
  skipped: number;
  repos: number;
  notes: number;
  pending_actions: number;
  duplicate_groups: number;
  loose_files: number;
  bytes_loose: number;
}

export interface Progress {
  phase: string;
  detail: string;
  stats: Stats;
  running: boolean;
}

export interface Health {
  ollama_up: boolean;
  ollama_url: string;
  models_installed: number;
  db_path: string;
  running: boolean;
}

export interface CatalogRow {
  id: number;
  path: string;
  name: string;
  class: string;
  size: number;
  mtime: number;
  repo: string | null;
  stage: number;
  title: string;
  kind: string;
  purpose: string;
  summary: string;
  topics: string[];
  status: string;
  action: string;
  reason: string;
  confidence: number;
  model: string;
}

export interface Hit {
  file_id: number;
  path: string;
  name: string;
  class: string;
  size: number;
  mtime: number;
  repo: string | null;
  title: string;
  summary: string;
  status: string;
  label: string | null;
  score: number;
  snippet: string;
  duplicates: string[];
  canonical: boolean;
}

export interface RepoInfo {
  path: string;
  name: string;
  owner: string;
  remote: string | null;
  kind: string;
  branch: string | null;
  last_commit: number;
  dirty: number;
  untracked: number;
  unpushed: number;
  stashes: number;
  recoverable: boolean;
  size_bytes: number;
}

export interface DupGroup {
  sha256: string;
  count: number;
  size: number;
  paths: string[];
  reclaimable: number;
}

export interface ActionRow {
  id: number;
  plan_id: number;
  file_id: number | null;
  kind: string;
  src: string;
  dest: string;
  category: string;
  reason: string;
  confidence: number;
  state: string;
  error: string | null;
}

export interface ApplyReport {
  applied: number;
  failed: number;
  skipped: number;
}

export interface ModelRouting {
  embed: string;
  triage: string;
  code: string;
  docs: string;
  vision: string;
  vision_escalate: string;
  escalate: string;
  escalate_max: string;
  escalate_below: number;
}

export interface Config {
  roots: string[];
  prune_names: string[];
  prune_fragments: string[];
  vault: string;
  monorepo: string;
  library: string;
  ollama_url: string;
  models: ModelRouting;
  max_read_bytes: number;
  auto_apply_above: number;
  stale_days: number;
  interpret_concurrency: number;
}

export interface TagModel {
  name: string;
  size: number;
}

export interface Supersession {
  old_id: number;
  old_path: string;
  new_id: number;
  new_path: string;
  new_repo: string | null;
  similarity: number;
  reason: string;
}

export interface Note {
  id: number;
  path: string;
  title: string;
  body: string;
  tags: string[];
  created_at: number;
  updated_at: number;
}

export interface NoteLink {
  target: string;
  dst_id: number | null;
}

export interface NoteDetail {
  note: Note;
  links: NoteLink[];
  /** [id, title] of notes pointing at this one. */
  backlinks: [number, string][];
}

export interface DocSyncCandidate {
  repoPath: string;
  repoName: string;
  docFiles: string[];
  reason: string;
}

export interface DocSyncResult {
  repoPath: string;
  updatedFiles: string[];
  diffSummary: string;
}

export type CodeFindingKind =
  | "environment_drift"
  | "unreferenced"
  | "contradicts_docs";

export interface CodeFinding {
  filePath: string;
  kind: CodeFindingKind;
  description: string;
  suggestedRelocation?: string | null;
}

export interface CodeSweepCandidate {
  repoPath: string;
  repoName: string;
  findings: CodeFinding[];
}

/* --------------------------------------------------------------- commands */

export const getStats = () => invoke<Stats>("get_stats");
export const getConfig = () => invoke<Config>("get_config");
export const saveConfig = (cfg: Config) => invoke<void>("save_config", { cfg });
export const health = () => invoke<Health>("health");
export const listModels = () => invoke<TagModel[]>("list_models");

export const startPipeline = () => invoke<void>("start_pipeline");
export const stopPipeline = () => invoke<void>("stop_pipeline");

export const listCatalog = (opts: {
  class?: string | null;
  status?: string | null;
  looseOnly: boolean;
  limit: number;
  offset: number;
}) =>
  invoke<CatalogRow[]>("list_catalog", {
    class: opts.class ?? null,
    status: opts.status ?? null,
    looseOnly: opts.looseOnly,
    limit: opts.limit,
    offset: opts.offset,
  });

export const searchCatalog = (query: string, limit = 40) =>
  invoke<Hit[]>("search_catalog", { query, limit });

export const listStale = (limit = 200) => invoke<Hit[]>("list_stale", { limit });
export const listRepos = () => invoke<RepoInfo[]>("list_repos");
export const listDuplicates = (limit = 100) =>
  invoke<DupGroup[]>("list_duplicates", { limit });
export const listReview = (limit = 200) =>
  invoke<ActionRow[]>("list_review", { limit });
export const listHistory = (limit = 200) =>
  invoke<ActionRow[]>("list_history", { limit });

export const decideAction = (actionId: number, approve: boolean) =>
  invoke<void>("decide_action", { actionId, approve });
export const applyApproved = () => invoke<ApplyReport>("apply_approved");
export const undoPlan = (planId: number) =>
  invoke<ApplyReport>("undo_plan", { planId });

export const similarFiles = (fileId: number, limit = 10) =>
  invoke<[number, string, number][]>("similar_files", { fileId, limit });
export const readFileText = (fileId: number) =>
  invoke<string>("read_file_text", { fileId });

export const listSupersessions = (limit = 200) =>
  invoke<Supersession[]>("list_supersessions", { limit });
export const repoTopics = (repo: string, limit = 25) =>
  invoke<[string, number][]>("repo_topics", { repo, limit });

export const listNotes = (limit = 200) => invoke<Note[]>("list_notes", { limit });
export const getNote = (id: number) => invoke<NoteDetail>("get_note", { id });
export const saveNote = (title: string, body: string) =>
  invoke<number>("save_note", { title, body });
export const deleteNote = (id: number) => invoke<void>("delete_note", { id });

export const listDocsyncCandidates = () =>
  invoke<DocSyncCandidate[]>("list_docsync_candidates");
export const runDocsync = (repoPath: string) =>
  invoke<DocSyncResult>("run_docsync", { repoPath });

export const listCodeSweepCandidates = () =>
  invoke<CodeSweepCandidate[]>("list_code_sweep_candidates");
export const runCodeRelocation = (
  repoPath: string,
  filePath: string,
  destination: string
) => invoke<string>("run_code_relocation", { repoPath, filePath, destination });

export const onProgress = (cb: (p: Progress) => void) =>
  listen<Progress>("librarian://progress", (e) => cb(e.payload));

/* ---------------------------------------------------------------- helpers */

export function humanBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let v = n / 1024;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toFixed(v < 10 ? 1 : 0)} ${units[i]}`;
}

export function humanDate(epochSeconds: number): string {
  if (!epochSeconds) return "-";
  return new Date(epochSeconds * 1000).toISOString().slice(0, 10);
}

export function ageDays(epochSeconds: number): number {
  if (!epochSeconds) return 0;
  return Math.floor((Date.now() / 1000 - epochSeconds) / 86400);
}
