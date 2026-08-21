import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

type LogType = "info" | "success" | "error";
type ModalMode = "search" | "prompt" | "picker";

let cachedItems: string[] = [];
let modalResolve: ((value: string | null) => void) | null = null;
let currentModalMode: ModalMode = "search";
let executionContextMode: "download" | "stream" = "download";
let designatedTargetString: string | null = null;

window.addEventListener("DOMContentLoaded", () => {
  const logContent = document.getElementById("logContent")!;
  const clearLogBtn = document.getElementById("clearLogBtn")!;

  const modalElement = document.getElementById("pipelinePickerModal")!;
  const modalHeadline = document.getElementById("modalHeadline")!;
  const modalFilter = document.getElementById("modalFilter") as HTMLInputElement;
  const modalDataContainer = document.getElementById("modalDataContainer")!;
  const modalMetaStatus = document.getElementById("modalMetaStatus")!;
  const btnCancelPicker = document.getElementById("btnCancelPicker")!;
  const btnConfirmPicker = document.getElementById("btnConfirmPicker")!;

  // ── Log Panel ───────────────────────────────────────────────────────────

  function appendLog(message: string, type: LogType = "info") {
    const entry = document.createElement("div");
    entry.className = `log-entry log-${type}`;
    const ts = new Date().toLocaleTimeString();
    entry.textContent = `[${ts}] ${message}`;
    logContent.appendChild(entry);
    logContent.scrollTop = logContent.scrollHeight;
  }

  clearLogBtn.addEventListener("click", () => {
    logContent.innerHTML = "";
  });

  // ── Modal: Close Helper ─────────────────────────────────────────────────

  function closeModal(result: string | null = null) {
    modalElement.classList.add("invisible");
    modalDataContainer.style.display = "";
    modalMetaStatus.style.display = "";
    if (modalResolve) {
      modalResolve(result);
      modalResolve = null;
    }
  }

  // ── Modal: Prompt Mode ──────────────────────────────────────────────────

  function showPrompt(
    title: string,
    placeholder: string = "",
    defaultValue: string = ""
  ): Promise<string | null> {
    return new Promise((resolve) => {
      currentModalMode = "prompt";
      modalHeadline.textContent = title;
      modalFilter.placeholder = placeholder;
      modalFilter.value = defaultValue;
      modalDataContainer.style.display = "none";
      modalMetaStatus.style.display = "none";
      designatedTargetString = null;
      modalElement.classList.remove("invisible");
      modalFilter.focus();
      if (defaultValue) modalFilter.select();
      modalResolve = resolve;
    });
  }

  // ── Modal: Picker Mode (pre-loaded list) ────────────────────────────────

  function showPicker(
    title: string,
    items: string[]
  ): Promise<string | null> {
    return new Promise((resolve) => {
      currentModalMode = "picker";
      cachedItems = items;
      modalHeadline.textContent = title;
      modalFilter.placeholder = "Filter...";
      modalFilter.value = "";
      modalDataContainer.style.display = "";
      modalMetaStatus.style.display = "";
      modalMetaStatus.textContent = `${items.length} items`;
      designatedTargetString = null;
      populateSelectionList(items);
      modalElement.classList.remove("invisible");
      modalFilter.focus();
      modalResolve = resolve;
    });
  }

  // ── Modal: Search Mode (Play Store) ─────────────────────────────────────

  function launchPlayStoreSearch(mode: "download" | "stream") {
    currentModalMode = "search";
    executionContextMode = mode;
    modalHeadline.textContent =
      mode === "stream"
        ? "Play Store: Search & Stream-Install"
        : "Play Store: Search & Download APK";
    cachedItems = [];
    modalDataContainer.innerHTML = "";
    modalDataContainer.style.display = "";
    modalFilter.placeholder = "Type query + press Enter to search...";
    modalFilter.value = "";
    modalMetaStatus.style.display = "";
    modalMetaStatus.textContent = "Awaiting search term...";
    designatedTargetString = null;
    modalElement.classList.remove("invisible");
    modalFilter.focus();
    modalResolve = null;
  }

  // ── Modal: Event Handlers ───────────────────────────────────────────────

  btnCancelPicker.addEventListener("click", () => closeModal(null));

  btnConfirmPicker.addEventListener("click", () => {
    if (currentModalMode === "prompt") {
      closeModal(modalFilter.value.trim() || null);
    } else if (currentModalMode === "picker") {
      closeModal(designatedTargetString);
    } else {
      commitSelectedPipeline();
    }
  });

  modalFilter.addEventListener("input", () => {
    if (currentModalMode === "prompt") return;
    const term = modalFilter.value.toLowerCase();
    const matches = cachedItems.filter((p) => p.toLowerCase().includes(term));
    populateSelectionList(matches);
  });

  modalFilter.addEventListener("keydown", async (e) => {
    if (e.key === "Escape") {
      closeModal(null);
      return;
    }

    if (e.key !== "Enter") return;

    if (currentModalMode === "prompt") {
      closeModal(modalFilter.value.trim() || null);
      return;
    }

    if (currentModalMode === "search") {
      const term = modalFilter.value.trim();
      if (!term) return;

      modalMetaStatus.textContent = "Searching...";
      modalDataContainer.innerHTML =
        "<li class='picker-row'>Querying Play Store...</li>";

      try {
        cachedItems = await invoke("search_play_store", { query: term });
        populateSelectionList(cachedItems);
        modalMetaStatus.textContent = `Found ${cachedItems.length} results.`;
      } catch (err) {
        modalDataContainer.innerHTML = `<li class='picker-row' style='color: #ff4444;'>Error: ${err}</li>`;
        modalMetaStatus.textContent = "Search failed.";
      }
    }
  });

  // ── List Population ─────────────────────────────────────────────────────

  function populateSelectionList(elements: string[]) {
    modalDataContainer.innerHTML = "";
    if (elements.length === 0) {
      modalDataContainer.innerHTML =
        "<li class='picker-row'>No results.</li>";
      return;
    }

    elements.forEach((item) => {
      const row = document.createElement("li");
      row.className = "picker-row";
      row.textContent = item;

      row.addEventListener("click", () => {
        Array.from(modalDataContainer.children).forEach((r) =>
          (r as HTMLElement).classList.remove("selected")
        );
        row.classList.add("selected");
        designatedTargetString = item;
      });

      row.addEventListener("dblclick", () => {
        designatedTargetString = item;
        if (currentModalMode === "picker") {
          closeModal(item);
        } else if (currentModalMode === "search") {
          commitSelectedPipeline();
        }
      });

      modalDataContainer.appendChild(row);
    });
  }

  // ── Play Store Pipeline ─────────────────────────────────────────────────

  async function commitSelectedPipeline() {
    const rawText =
      designatedTargetString ||
      modalDataContainer.firstElementChild?.textContent;
    if (
      !rawText ||
      rawText.includes("No results") ||
      rawText.includes("Querying")
    )
      return;

    const parsedId = rawText.match(/\[(.*?)\]/)?.[1];
    if (!parsedId) return;

    modalElement.classList.add("invisible");

    if (executionContextMode === "stream") {
      appendLog(`Streaming ${parsedId} to device...`);
      try {
        const result = await invoke<string>("execute_stream_pipeline", {
          packageId: parsedId,
        });
        appendLog(result, "success");
      } catch (err) {
        appendLog(`Stream failed: ${err}`, "error");
      }
    } else {
      appendLog(`Downloading ${parsedId}...`);
      try {
        const result = await invoke<string>("download_apk", {
          packageId: parsedId,
          folder: "/tmp/gplay_downloads",
        });
        appendLog(result, "success");
      } catch (err) {
        appendLog(`Download failed: ${err}`, "error");
      }
    }
  }

  // ── Button Handlers ─────────────────────────────────────────────────────

  // Play Store
  document
    .getElementById("fetch-apk-btn")
    ?.addEventListener("click", () => launchPlayStoreSearch("download"));
  document
    .getElementById("stream-apk-btn")
    ?.addEventListener("click", () => launchPlayStoreSearch("stream"));

  // Push File
  document
    .getElementById("push-file-btn")
    ?.addEventListener("click", async () => {
      const file = await open({ multiple: false, title: "Select file to push" });
      if (!file) return;
      const localPath = file as string;
      const filename = localPath.split("/").pop() || "file";
      const remotePath = await showPrompt(
        "Remote destination path:",
        "/sdcard/path",
        `/sdcard/${filename}`
      );
      if (!remotePath) return;

      appendLog(`Pushing ${filename} to ${remotePath}...`);
      try {
        const result = await invoke<string>("push_file", {
          localPath,
          remotePath,
        });
        appendLog(result, "success");
      } catch (err) {
        appendLog(`Push failed: ${err}`, "error");
      }
    });

  // Pull File
  document
    .getElementById("pull-file-btn")
    ?.addEventListener("click", async () => {
      const remotePath = await showPrompt(
        "Remote file path to pull:",
        "/sdcard/path/to/file"
      );
      if (!remotePath) return;
      const filename = remotePath.split("/").pop() || "pulled_file";
      const localPath = await save({
        title: "Save pulled file as...",
        defaultPath: filename,
      });
      if (!localPath) return;

      appendLog(`Pulling ${remotePath}...`);
      try {
        const result = await invoke<string>("pull_file", {
          remotePath,
          localPath,
        });
        appendLog(result, "success");
      } catch (err) {
        appendLog(`Pull failed: ${err}`, "error");
      }
    });

  // Install APK
  document
    .getElementById("install-apk-btn")
    ?.addEventListener("click", async () => {
      const file = await open({
        multiple: false,
        title: "Select APK to install",
        filters: [{ name: "APK Files", extensions: ["apk"] }],
      });
      if (!file) return;
      const path = file as string;

      appendLog(`Installing ${path.split("/").pop()}...`);
      try {
        const result = await invoke<string>("install_apk", { path });
        appendLog(result, "success");
      } catch (err) {
        appendLog(`Install failed: ${err}`, "error");
      }
    });

  // Batch Install
  document
    .getElementById("batch-apk-btn")
    ?.addEventListener("click", async () => {
      const folder = await open({
        directory: true,
        title: "Select folder containing APKs",
      });
      if (!folder) return;

      appendLog(`Batch installing from ${folder}...`);
      try {
        const result = await invoke<string>("batch_install_apks", {
          folder: folder as string,
        });
        appendLog(result, "success");
      } catch (err) {
        appendLog(`Batch install: ${err}`, "error");
      }
    });

  // Purge App Cache
  document
    .getElementById("purge-cache-btn")
    ?.addEventListener("click", async () => {
      appendLog("Fetching installed packages...");
      try {
        const packages = await invoke<string[]>("list_packages");
        const selected = await showPicker(
          "Select package to clear data:",
          packages
        );
        if (!selected) return;

        appendLog(`Clearing data for ${selected}...`);
        const result = await invoke<string>("purge_app_cache", {
          packageId: selected,
        });
        appendLog(result, "success");
      } catch (err) {
        appendLog(`Purge failed: ${err}`, "error");
      }
    });

  // Inject Text
  document
    .getElementById("inject-macro-btn")
    ?.addEventListener("click", async () => {
      const text = await showPrompt("Enter text to inject:", "Type text here...");
      if (!text) return;

      appendLog(`Injecting text...`);
      try {
        const result = await invoke<string>("inject_text", { text });
        appendLog(result, "success");
      } catch (err) {
        appendLog(`Injection failed: ${err}`, "error");
      }
    });

  // Logcat
  document
    .getElementById("open-logcat-btn")
    ?.addEventListener("click", async () => {
      appendLog("Capturing logcat...");
      try {
        const result = await invoke<string>("capture_logcat");
        appendLog("─── Logcat ───", "info");
        result.split("\n").forEach((line) => {
          if (line.trim()) appendLog(line);
        });
        appendLog("─── End ───", "info");
      } catch (err) {
        appendLog(`Logcat failed: ${err}`, "error");
      }
    });

  // Screenshot
  document
    .getElementById("capture-screen-btn")
    ?.addEventListener("click", async () => {
      const path = await save({
        title: "Save screenshot as...",
        defaultPath: "screenshot.png",
        filters: [{ name: "PNG Image", extensions: ["png"] }],
      });
      if (!path) return;

      appendLog("Capturing screenshot...");
      try {
        const result = await invoke<string>("capture_screenshot", {
          savePath: path,
        });
        appendLog(result, "success");
      } catch (err) {
        appendLog(`Screenshot failed: ${err}`, "error");
      }
    });

  // Screen Record
  document
    .getElementById("record-screen-btn")
    ?.addEventListener("click", async () => {
      const path = await save({
        title: "Save recording as...",
        defaultPath: "screenrecord.mp4",
        filters: [{ name: "MP4 Video", extensions: ["mp4"] }],
      });
      if (!path) return;

      appendLog("Recording screen for 10 seconds...");
      try {
        const result = await invoke<string>("record_screen", {
          savePath: path,
        });
        appendLog(result, "success");
      } catch (err) {
        appendLog(`Recording failed: ${err}`, "error");
      }
    });

  // Copy to Mounted SD Image
  document
    .getElementById("copy-image-btn")
    ?.addEventListener("click", async () => {
      const file = await open({ multiple: false, title: "Select file to copy" });
      if (!file) return;

      appendLog(`Copying to mount point...`);
      try {
        const result = await invoke<string>("copy_to_mount", {
          source: file as string,
          mountPoint: "/home/loufogle/mount_point",
        });
        appendLog(result, "success");
      } catch (err) {
        appendLog(`Copy failed: ${err}`, "error");
      }
    });

  // Restart Framework
  document
    .getElementById("restart-framework-btn")
    ?.addEventListener("click", async () => {
      appendLog("Restarting UI framework...");
      try {
        const result = await invoke<string>("restart_framework");
        appendLog(result, "success");
      } catch (err) {
        appendLog(`Restart failed: ${err}`, "error");
      }
    });

  // Reboot Bootloader
  document
    .getElementById("bootloader-btn")
    ?.addEventListener("click", async () => {
      appendLog("Rebooting to bootloader...");
      try {
        const result = await invoke<string>("reboot_bootloader");
        appendLog(result, "success");
      } catch (err) {
        appendLog(`Reboot failed: ${err}`, "error");
      }
    });

  // Reboot Recovery
  document
    .getElementById("recovery-btn")
    ?.addEventListener("click", async () => {
      appendLog("Rebooting to recovery...");
      try {
        const result = await invoke<string>("reboot_recovery");
        appendLog(result, "success");
      } catch (err) {
        appendLog(`Reboot failed: ${err}`, "error");
      }
    });

  // ── Init ────────────────────────────────────────────────────────────────

  appendLog("ADB Toolbox initialized.", "success");
});
