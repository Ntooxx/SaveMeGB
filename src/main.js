const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const HISTORY_KEY = "SaveMeGB.history";
const LAST_SCAN_KEY = "SaveMeGB.lastScan";
const SETTINGS_KEY = "SaveMeGB.settings";
const LAST_PURGE_KEY = "SaveMeGB.lastPurge";
const MAX_HISTORY = 30;
const MAX_TREE_ROWS = 200;

const state = {
  report: null,
  previousReport: null,
  selected: new Set(),
  screen: "dashboard",
  filter: "all",
  search: "",
  sort: "size-desc",
  history: [],
  settings: null,
  currentDetail: null,
  scanning: false,
  lastPurge: null,
  lastPurgeErrors: [],
};

const els = {
  tabs: document.querySelectorAll("#tabs .nav-item"),
  screens: document.querySelectorAll(".screen"),
  scanBtn: document.getElementById("scan-btn"),
  scanBtnTop: document.getElementById("scan-btn-top"),
  scanLabel: document.getElementById("scan-label"),
  scanSub: document.getElementById("scan-sub"),
  scanEmptyBtn: document.getElementById("empty-scan"),
  libEmptyBtn: document.getElementById("lib-empty-scan"),
  insightEmptyBtn: document.getElementById("insight-empty-scan"),
  qaStandard: document.getElementById("qa-standard"),
  qaSmart: document.getElementById("qa-smart"),
  qaBtns: document.querySelectorAll(".qa-btn[data-mode]"),
  manifestPill: document.getElementById("manifest-pill"),
  manifestText: document.getElementById("manifest-text"),
  lastScanned: document.getElementById("last-scanned"),
  lastReclaimable: document.getElementById("last-reclaimable"),
  lastReclaimDelta: document.getElementById("last-reclaim-delta"),
  lastSafe: document.getElementById("last-safe"),
  lastSafeCount: document.getElementById("last-safe-count"),
  lastGames: document.getElementById("last-games"),
  lastSources: document.getElementById("last-sources"),
  lastScanMode: document.getElementById("last-scan-mode"),
  sysOs: document.getElementById("sys-os"),
  sysDisk: document.getElementById("sys-disk"),
  sysHost: document.getElementById("sys-host"),
  diskBarFill: document.getElementById("disk-bar-fill"),
  diskLegend: document.getElementById("disk-legend"),
  totalReclaim: document.getElementById("total-reclaim"),
  orphanCount: document.getElementById("orphan-count"),
  safeSubset: document.getElementById("safe-subset"),
  selectedTotal: document.getElementById("selected-total"),
  selectedCount: document.getElementById("selected-count"),
  orphanList: document.getElementById("orphan-list"),
  emptyState: document.getElementById("empty-state"),
  search: document.getElementById("search"),
  sort: document.getElementById("sort"),
  filters: document.querySelectorAll("#filters .chip"),
  resultsBadge: document.getElementById("results-badge"),
  crumbs: document.getElementById("crumbs"),
  statusText: document.getElementById("status-text"),
  statusDot: document.getElementById("status-dot"),
  statusMeta: document.getElementById("status-meta"),
  progress: document.getElementById("progress"),
  progTitle: document.getElementById("prog-title"),
  progFill: document.getElementById("prog-fill"),
  progMeta: document.getElementById("prog-meta"),
  backupPath: document.getElementById("backup-path"),
  pickBackup: document.getElementById("pick-backup"),
  migrateGame: document.getElementById("migrate-game"),
  migrateBtn: document.getElementById("migrate-btn"),
  smartClean: document.getElementById("smart-clean"),
  historyList: document.getElementById("history-list"),
  historyClear: document.getElementById("history-clear"),
  activityFeed: document.getElementById("activity-feed"),
  miniList: document.getElementById("mini-list"),
  viewAllActivity: document.getElementById("view-all-activity"),
  viewAllResults: document.getElementById("view-all-results"),
  settingsBtn: document.getElementById("settings-btn"),
  settingsModal: document.getElementById("settings-modal"),
  settingsCancel: document.getElementById("settings-cancel"),
  settingsSave: document.getElementById("settings-save"),
  setTheme: document.getElementById("set-theme"),
  setBackup: document.getElementById("set-backup"),
  setPickBackup: document.getElementById("set-pick-backup"),
  setAuto: document.getElementById("set-auto"),
  setAutoScan: document.getElementById("set-auto-scan"),
  setNotif: document.getElementById("set-notif"),
  setConf: document.getElementById("set-conf"),
  setConfVal: document.getElementById("set-conf-val"),
  setMode: document.getElementById("set-mode"),
  confirmModal: document.getElementById("confirm-modal"),
  confirmTitle: document.getElementById("confirm-title"),
  confirmBody: document.getElementById("confirm-body"),
  confirmOk: document.getElementById("confirm-ok"),
  confirmCancel: document.getElementById("confirm-cancel"),
  confirmDetail: document.getElementById("confirm-detail"),
  detailModal: document.getElementById("detail-modal"),
  detailName: document.getElementById("detail-name"),
  detailPath: document.getElementById("detail-path"),
  detailSize: document.getElementById("detail-size"),
  detailMeta: document.getElementById("detail-meta"),
  detailTree: document.getElementById("detail-tree"),
  detailReveal: document.getElementById("detail-reveal"),
  detailWhitelist: document.getElementById("detail-whitelist"),
  detailClose: document.getElementById("detail-close"),
  detailCopy: document.getElementById("detail-copy"),
  whitelistAdd: document.getElementById("whitelist-add"),
  wlNames: document.getElementById("wl-names"),
  wlPaths: document.getElementById("wl-paths"),
  wlNameInput: document.getElementById("wl-name-input"),
  wlNameAdd: document.getElementById("wl-name-add"),
  wlPathInput: document.getElementById("wl-path-input"),
  wlPathAdd: document.getElementById("wl-path-add"),
  libraryGrid: document.getElementById("library-grid"),
  libraryBadge: document.getElementById("library-badge"),
  libEmpty: document.getElementById("lib-empty"),
  libCount: document.getElementById("lib-count"),
  libSummary: document.getElementById("lib-summary"),
  libSearch: document.getElementById("lib-search"),
  insightTotal: document.getElementById("insight-total"),
  insightSub: document.getElementById("insight-sub"),
  insightCount: document.getElementById("insight-count"),
  insightEmpty: document.getElementById("insight-empty"),
  chartCategory: document.getElementById("chart-category"),
  chartPublisher: document.getElementById("chart-publisher"),
  chartConfidence: document.getElementById("chart-confidence"),
  chartTop: document.getElementById("chart-top"),
  exportJson: document.getElementById("export-json"),
  exportCsv: document.getElementById("export-csv"),
  runTestsBtn: document.getElementById("run-tests-btn"),
  toastHost: document.getElementById("toast-host"),
  shortcutsModal: document.getElementById("shortcuts-modal"),
  shortcutsClose: document.getElementById("shortcuts-close"),
  topbar: document.querySelector(".topbar"),
  deleteVisible: document.getElementById("delete-visible"),
  celebrationModal: document.getElementById("celebration-modal"),
  celebrationTitle: document.getElementById("celebration-title"),
  celebrationSub: document.getElementById("celebration-sub"),
  celebrationStats: document.getElementById("celebration-stats"),
  celebrationQuote: document.getElementById("celebration-quote"),
  celebrationRecycle: document.getElementById("celebration-recycle"),
  celebrationAgain: document.getElementById("celebration-again"),
  celebrationClose: document.getElementById("celebration-close"),
  confettiCanvas: document.getElementById("confetti-canvas"),
};

els.tabs.forEach((btn) => btn.addEventListener("click", () => switchScreen(btn.dataset.screen)));
els.scanBtn.addEventListener("click", () => runScan("standard"));
els.scanBtnTop.addEventListener("click", () => runScan("standard"));
els.scanEmptyBtn.addEventListener("click", () => runScan("standard"));
els.libEmptyBtn.addEventListener("click", () => runScan("standard"));
els.insightEmptyBtn.addEventListener("click", () => runScan("standard"));
els.qaStandard.addEventListener("click", () => runScan("standard"));
els.qaSmart.addEventListener("click", () => smartClean());
els.qaBtns.forEach((b) => b.addEventListener("click", () => runScan(b.dataset.mode)));

els.search.addEventListener("input", (e) => { state.search = e.target.value.toLowerCase(); renderOrphans(); });
els.sort.addEventListener("change", (e) => { state.sort = e.target.value; renderOrphans(); });
els.filters.forEach((chip) => chip.addEventListener("click", () => {
  els.filters.forEach((c) => c.classList.remove("active"));
  chip.classList.add("active");
  state.filter = chip.dataset.cat;
  renderOrphans();
}));

els.pickBackup.addEventListener("click", pickBackup);
els.smartClean.addEventListener("click", smartClean);
els.migrateBtn.addEventListener("click", planMigration);

els.settingsBtn.addEventListener("click", openSettings);
els.settingsCancel.addEventListener("click", () => hideModal(els.settingsModal));
els.settingsSave.addEventListener("click", saveSettings);
els.setConf.addEventListener("input", (e) => { els.setConfVal.textContent = e.target.value; });
els.setPickBackup.addEventListener("click", async () => {
  const picked = await openDialogFolder("Pick backup folder");
  if (picked) els.setBackup.value = picked;
});
els.confirmCancel.addEventListener("click", () => hideModal(els.confirmModal));
els.confirmOk.addEventListener("click", () => { hideModal(els.confirmModal); if (state._confirmCb) state._confirmCb(); });
els.historyClear.addEventListener("click", () => { state.history = []; saveHistory(); renderHistory(); renderActivityFeed(); });
els.viewAllActivity.addEventListener("click", () => switchScreen("history"));
els.viewAllResults.addEventListener("click", () => switchScreen("results"));
els.exportJson.addEventListener("click", () => exportReport("json"));
els.exportCsv.addEventListener("click", () => exportReport("csv"));
els.runTestsBtn.addEventListener("click", runEngineTests);

els.whitelistAdd.addEventListener("click", () => {
  if (!state.currentDetail) return;
  if (!state.settings.whitelist.names.includes(state.currentDetail.game_hint)) {
    state.settings.whitelist.names.push(state.currentDetail.game_hint);
    saveSettingsSilent().then(() => { renderWhitelistEdit(); toast("Added to whitelist", "ok"); });
  } else {
    toast("Already whitelisted", "warn");
  }
});
els.wlNameAdd.addEventListener("click", () => addWl("names", els.wlNameInput));
els.wlPathAdd.addEventListener("click", () => addWl("paths", els.wlPathInput));
els.wlNameInput.addEventListener("keydown", (e) => { if (e.key === "Enter") addWl("names", els.wlNameInput); });
els.wlPathInput.addEventListener("keydown", (e) => { if (e.key === "Enter") addWl("paths", els.wlPathInput); });

els.detailReveal.addEventListener("click", async () => {
  if (!state.currentDetail) return;
  try { await invoke("open_in_explorer", { path: state.currentDetail.path }); }
  catch (e) { toast(`Open failed: ${e}`, "err"); }
});
els.detailCopy.addEventListener("click", async () => {
  if (!state.currentDetail) return;
  try { await invoke("copy_to_clipboard", { text: state.currentDetail.path }); toast("Path copied to clipboard", "ok", 2000); }
  catch (e) { toast(`Copy failed: ${e}`, "err"); }
});
els.detailClose.addEventListener("click", () => hideModal(els.detailModal));
els.detailWhitelist.addEventListener("click", () => {
  if (!state.currentDetail) return;
  const path = state.currentDetail.path;
  if (!state.settings.whitelist.paths.some((p) => path.toLowerCase().includes(p.toLowerCase()))) {
    state.settings.whitelist.paths.push(path);
    saveSettingsSilent().then(() => { renderWhitelistEdit(); toast("Path added to whitelist", "ok"); });
  }
});
els.detailClose.addEventListener("click", () => hideModal(els.detailModal));

els.libSearch.addEventListener("input", renderLibrary);

document.querySelectorAll("[data-bulk]").forEach((b) => b.addEventListener("click", () => bulkSelect(b.dataset.bulk)));

if (els.shortcutsClose) els.shortcutsClose.addEventListener("click", () => hideModal(els.shortcutsModal));
if (els.deleteVisible) els.deleteVisible.addEventListener("click", deleteAllVisible);
if (els.celebrationRecycle) els.celebrationRecycle.addEventListener("click", async () => { try { await invoke("open_recycle_bin"); } catch (e) {} });
if (els.celebrationAgain) els.celebrationAgain.addEventListener("click", () => { hideModal(els.celebrationModal); switchScreen("results"); });
if (els.celebrationClose) els.celebrationClose.addEventListener("click", () => { hideModal(els.celebrationModal); stopConfetti(); });

document.addEventListener("keydown", (e) => {
  if (e.target.tagName === "INPUT" || e.target.tagName === "TEXTAREA" || e.target.isContentEditable) return;
  if (e.key === "?" || (e.shiftKey && e.key === "/")) { e.preventDefault(); showModal(els.shortcutsModal); }
});

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    [els.confirmModal, els.settingsModal, els.detailModal, els.welcomeModal, els.shortcutsModal].forEach((m) => {
      if (m && !m.hidden) hideModal(m);
    });
  }
  if (e.ctrlKey && e.key === "f") { e.preventDefault(); switchScreen("results"); els.search.focus(); }
  if (e.ctrlKey && e.key === ",") { e.preventDefault(); openSettings(); }
  if (e.ctrlKey && e.key === "e") { e.preventDefault(); if (state.report) exportReport("json"); }
  if (e.key === "F5" || (e.ctrlKey && e.key === "r")) { e.preventDefault(); runScan(state.settings?.scan_mode || "standard"); }
});

function switchScreen(name) {
  state.screen = name;
  els.tabs.forEach((t) => t.classList.toggle("active", t.dataset.screen === name));
  els.screens.forEach((s) => s.classList.toggle("active", s.id === `screen-${name}`));
  const labels = { dashboard: "Dashboard", results: "Scan Results", library: "Library", insights: "Insights", backup: "Backup & Migrate", history: "History" };
  els.crumbs.textContent = labels[name] || name;
  if (name === "library") renderLibrary();
  if (name === "insights") renderInsights();
  if (name === "history") renderHistory();
  if (name === "dashboard") renderActivityFeed();
}

function setStatus(text, level = "") {
  els.statusText.textContent = text;
  els.statusDot.className = "status-dot" + (level ? " " + level : "");
}

function toast(message, level = "ok", timeout = 4000) {
  const div = document.createElement("div");
  div.className = `toast ${level}`;
  div.textContent = message;
  els.toastHost.appendChild(div);
  setTimeout(() => { div.style.opacity = "0"; div.style.transform = "translateX(20px)"; setTimeout(() => div.remove(), 200); }, timeout);
}

async function openDialogFolder(title) {
  try { return await invoke("plugin:dialog|open", { options: { directory: true, multiple: false, title } }); }
  catch (e) { toast(`Dialog failed: ${e}`, "err"); return null; }
}

async function openDialogSave(defaultName, filters) {
  try { return await invoke("plugin:dialog|save", { options: { defaultPath: defaultName, filters } }); }
  catch (e) { toast(`Dialog failed: ${e}`, "err"); return null; }
}

async function runScan(mode) {
  mode = mode || (state.settings && state.settings.scan_mode) || "standard";
  state.scanning = true;
  els.scanBtn.classList.add("scanning");
  els.scanLabel.textContent = "Scanning…";
  els.scanSub.textContent = `Mode: ${mode}`;
  setStatus(`Scanning (${mode})…`, "busy");
  showProgress(true, `Scanning (${mode})`, 0, "Initializing");
  try {
    const wl = state.settings ? state.settings.whitelist : { paths: [], names: [], publishers: [] };
    const report = await invoke("scan_system", { mode, whitelist: wl });
    state.previousReport = state.report;
    state.report = report;
    saveLastScan();
    pushHistory({ type: "scan", message: `Found ${report.orphaned_files.length} candidate(s) [${mode}]`, bytes: report.total_reclaimable_bytes });
    renderReport(report);
    setStatus(`Scan complete: ${report.orphaned_files.length} candidate(s) · ${humanSize(report.total_reclaimable_bytes)}`, "ok");
    toast(`Scan complete · ${humanSize(report.total_reclaimable_bytes)} reclaimable`, "ok");
    if (state.settings && state.settings.notifications_enabled) {
      try { await sendNotification("SaveMeGB scan complete", `${report.orphaned_files.length} candidate(s), ${humanSize(report.total_reclaimable_bytes)}`); } catch (e) { /* ignore */ }
    }
    if (report.orphaned_files.length > 0) switchScreen("results");
  } catch (e) {
    setStatus(`Scan failed: ${e}`, "err");
    toast(`Scan failed: ${e}`, "err", 6000);
  } finally {
    state.scanning = false;
    els.scanBtn.classList.remove("scanning");
    els.scanLabel.textContent = "Tap to scan";
    els.scanSub.textContent = "Detect orphaned game data";
    showProgress(false);
  }
}

function showProgress(show, title = "", pct = 0, meta = "") {
  if (show && !state.scanning) return;
  els.progress.hidden = !show;
  if (show) {
    els.progTitle.textContent = title;
    els.progFill.style.width = `${Math.max(0, Math.min(100, pct))}%`;
    els.progMeta.textContent = meta;
  }
}

function renderReport(report) {
  els.lastScanned.textContent = formatTime(report.scanned_at);
  els.lastReclaimable.textContent = humanSize(report.total_reclaimable_bytes);
  els.lastSafe.textContent = humanSize(report.safe_to_delete_bytes);
  els.lastSafeCount.textContent = `${report.orphaned_files.filter((o) => o.category.is_safer_to_delete || ["cache", "shaders", "crashes", "logs"].includes(o.category)).length} safe items`;
  els.lastGames.textContent = report.installed_games.length.toString();
  const sources = new Set(report.installed_games.map((g) => g.source));
  els.lastSources.textContent = `${sources.size} launcher(s): ${Array.from(sources).join(", ")}`;
  els.lastScanMode.textContent = `Mode: ${report.mode || "standard"}`;

  if (state.previousReport) {
    const delta = report.total_reclaimable_bytes - state.previousReport.total_reclaimable_bytes;
    const sign = delta > 0 ? "+" : "";
    els.lastReclaimDelta.textContent = `${sign}${humanSize(delta)} vs previous scan`;
    els.lastReclaimDelta.className = delta > 0 ? "stat-foot warn" : "stat-foot safe";
  } else {
    els.lastReclaimDelta.textContent = "First scan";
  }

  els.sysOs.textContent = report.system.windows_version;
  els.sysHost.textContent = report.system.hostname;
  if (report.system.total_bytes > 0) {
    const used = report.system.total_bytes - report.system.free_bytes;
    const pct = (used / report.system.total_bytes) * 100;
    els.diskBarFill.style.width = `${pct.toFixed(1)}%`;
    els.sysDisk.textContent = `${humanSize(used)} / ${humanSize(report.system.total_bytes)}`;
    els.diskLegend.textContent = `${humanSize(report.system.free_bytes)} free of ${humanSize(report.system.total_bytes)} (${pct.toFixed(1)}% used)`;
  }
  els.totalReclaim.textContent = humanSize(report.total_reclaimable_bytes);
  els.orphanCount.textContent = `${report.orphaned_files.length} candidate(s) · scan took ${(report.duration_ms / 1000).toFixed(1)}s`;
  els.safeSubset.textContent = humanSize(report.safe_to_delete_bytes);
  updateSelectedReadout();
  renderOrphans();
  populateMigrate(report);
  renderLibrary();
  renderInsights();
  renderMiniList();
  els.resultsBadge.textContent = report.orphaned_files.length;
  els.resultsBadge.hidden = report.orphaned_files.length === 0;
  els.libraryBadge.textContent = report.installed_games.length;
  els.emptyState.hidden = report.orphaned_files.length > 0;
  els.orphanList.hidden = report.orphaned_files.length === 0;
  els.libEmpty.hidden = report.installed_games.length > 0;
  els.libCount.textContent = report.installed_games.length;
  els.libSummary.textContent = `Detected across ${sources.size} launcher(s) · ${humanSize(report.installed_games.reduce((s, g) => s + g.size_bytes, 0))} total`;
  els.insightEmpty.hidden = report.orphaned_files.length > 0;
}

function getFilteredOrphans() {
  if (!state.report) return [];
  let list = state.report.orphaned_files.slice();
  const minConf = (state.settings && state.settings.min_confidence) || 0;
  list = list.filter((o) => o.confidence >= minConf);
  if (state.filter !== "all") list = list.filter((o) => o.category === state.filter);
  if (state.search) {
    const q = state.search;
    list = list.filter((o) =>
      o.game_hint.toLowerCase().includes(q) ||
      o.path.toLowerCase().includes(q) ||
      (o.source || "").toLowerCase().includes(q)
    );
  }
  switch (state.sort) {
    case "size-asc": list.sort((a, b) => a.size_bytes - b.size_bytes); break;
    case "name": list.sort((a, b) => a.game_hint.localeCompare(b.game_hint)); break;
    case "confidence": list.sort((a, b) => b.confidence - a.confidence); break;
    case "age": list.sort((a, b) => (b.last_modified || "").localeCompare(a.last_modified || "")); break;
    default: list.sort((a, b) => b.size_bytes - a.size_bytes);
  }
  return list;
}

function renderOrphans() {
  if (!state.report) return;
  const list = getFilteredOrphans();
  els.orphanList.innerHTML = "";
  for (const o of list) {
    const li = document.createElement("li");
    li.className = "orphan" + (state.selected.has(o.id) ? " selected" : "");
    li.innerHTML = `
      <div class="orphan-row">
        <input type="checkbox" ${state.selected.has(o.id) ? "checked" : ""} />
        <div>
          <div class="orphan-name">${escapeHtml(o.game_hint)}</div>
          <div class="orphan-meta">
            <span class="orphan-tag ${o.category}">${o.category}</span>
            <span class="orphan-confidence">confidence ${o.confidence}%</span>
            ${o.source ? `<span>${o.source}</span>` : ""}
            ${o.file_count > 0 ? `<span>${o.file_count.toLocaleString()} files</span>` : ""}
            ${o.last_modified ? `<span>${formatRelative(o.last_modified)}</span>` : ""}
          </div>
          <div class="orphan-path">${escapeHtml(o.path)}</div>
          <div class="orphan-meta">${escapeHtml(o.reason)}</div>
        </div>
        <div style="text-align:right">
          <div class="orphan-size">${humanSize(o.size_bytes)}</div>
          <div class="orphan-actions" style="margin-top:6px">
            <button class="orphan-action" data-act="detail" title="View details"><svg viewBox="0 0 24 24" width="14" height="14"><circle cx="12" cy="12" r="10" fill="none" stroke="currentColor" stroke-width="2"/><line x1="12" y1="16" x2="12" y2="12" stroke="currentColor" stroke-width="2" stroke-linecap="round"/><line x1="12" y1="8" x2="12.01" y2="8" stroke="currentColor" stroke-width="2" stroke-linecap="round"/></svg></button>
            <button class="orphan-action" data-act="reveal" title="Show in Explorer"><svg viewBox="0 0 24 24" width="14" height="14"><use href="#ico-external"/></svg></button>
            <button class="orphan-action" data-act="whitelist" title="Whitelist this"><svg viewBox="0 0 24 24" width="14" height="14"><use href="#ico-plus"/></svg></button>
            <button class="orphan-action" data-act="copy" title="Copy path"><svg viewBox="0 0 24 24" width="14" height="14"><use href="#ico-copy"/></svg></button>
            <button class="orphan-action" data-act="delete" title="Move to Recycle Bin"><svg viewBox="0 0 24 24" width="14" height="14"><use href="#ico-trash"/></svg></button>
          </div>
        </div>
      </div>
    `;
    const cb = li.querySelector("input[type=checkbox]");
    cb.addEventListener("change", () => toggleSelect(o.id, cb.checked));
    li.addEventListener("click", (e) => {
      const act = e.target.closest(".orphan-action")?.dataset?.act;
      if (act === "detail") return openDetail(o);
      if (act === "reveal") return revealPath(o.path);
      if (act === "whitelist") return whitelistOrphan(o);
      if (act === "copy") {
        invoke("copy_to_clipboard", { text: o.path }).then(() => toast("Path copied", "ok", 1500)).catch(() => {});
        return;
      }
      if (act === "delete") return deleteOne(o);
      cb.checked = !cb.checked;
      toggleSelect(o.id, cb.checked);
    });
    els.orphanList.appendChild(li);
  }
  updateSelectedReadout();
}

async function deleteOne(o) {
  const strategy = getStrategy();
  const verb = strategy.type === "direct_delete" ? "Force delete" : strategy.type === "backup_folder" ? "Move to backup" : "Recycle";
  const ok = await confirm({
    title: `${verb} "${o.game_hint}"?`,
    body: strategy.type === "direct_delete"
      ? `⚠ This will permanently remove ${humanSize(o.size_bytes)} and CANNOT be undone.`
      : `This will free ${humanSize(o.size_bytes)}. The file goes to the ${strategy.type === "recycle_bin" ? "Recycle Bin" : "backup folder"} and can be restored.`,
    detail: `<li class="size">${escapeHtml(humanSize(o.size_bytes))}</li><li>${escapeHtml(o.path)}</li>`,
  });
  if (!ok) return;
  setStatus("Purging…", "busy");
  try {
    const report = await purge([o], strategy);
    state.lastPurge = { report, orphans: [o], strategy, time: new Date().toISOString() };
    state.lastPurgeErrors = report.errors;
    try { localStorage.setItem(LAST_PURGE_KEY, JSON.stringify(state.lastPurge)); } catch (e) {}
    pushHistory({ type: "purge", message: `${verb} ${o.game_hint}`, bytes: report.bytes_freed });
    setStatus(`Done. ${report.moved.length} moved, ${report.errors.length} errors.`, report.errors.length ? "warn" : "ok");
    toast(`Freed ${humanSize(report.bytes_freed)}`, "ok");
    if (report.errors.length) showPurgeErrors(report.errors);
    showUndoToast(report, [o]);
    maybeCelebrate(report.bytes_freed, [o]);
    state.selected.delete(o.id);
    await runScan(state.report.mode || "standard");
  } catch (e) {
    setStatus(`Purge failed: ${e}`, "err");
    toast(`Purge failed: ${e}`, "err", 6000);
  }
}

async function deleteAllVisible() {
  const list = getFilteredOrphans();
  if (!list.length) { toast("No items to delete", "warn"); return; }
  const total = list.reduce((s, o) => s + o.size_bytes, 0);
  const strategy = getStrategy();
  const verb = strategy.type === "recycle_bin" ? "Recycle" : strategy.type === "direct_delete" ? "Force delete" : "Move to backup";
  const ok = await confirm({
    title: `⚠ ${verb} ALL ${list.length} visible items?`,
    body: strategy.type === "direct_delete"
      ? `⚠ This will permanently remove ${humanSize(total)} and CANNOT be undone.`
      : `This will free ${humanSize(total)}. Files ${strategy.type === "recycle_bin" ? "go to the Recycle Bin" : "are moved to " + strategy.path} and can be restored.`,
    detail: list.slice(0, 30).map((o) => `<li class="size">${escapeHtml(humanSize(o.size_bytes))}</li><li>${escapeHtml(o.game_hint)}</li>`).join("") + (list.length > 30 ? `<li>… and ${list.length - 30} more</li>` : ""),
  });
  if (!ok) return;
  setStatus(`Purging ${list.length} items…`, "busy");
  try {
    const report = await purge(list, strategy);
    state.lastPurge = { report, orphans: list, strategy, time: new Date().toISOString() };
    state.lastPurgeErrors = report.errors;
    try { localStorage.setItem(LAST_PURGE_KEY, JSON.stringify(state.lastPurge)); } catch (e) {}
    pushHistory({ type: "purge", message: `${verb} ${report.moved.length} item(s) (Delete all)`, bytes: report.bytes_freed });
    setStatus(`Done. ${report.moved.length} moved, ${report.errors.length} errors.`, report.errors.length ? "warn" : "ok");
    state.selected.clear();
    if (report.errors.length) {
      toast(`${report.errors.length} error(s) — likely locked. Try Force Delete.`, "warn", 8000);
      showPurgeErrors(report.errors);
    }
    showUndoToast(report, list);
    maybeCelebrate(report.bytes_freed, list);
    await runScan(state.report.mode || "standard");
  } catch (e) {
    setStatus(`Purge failed: ${e}`, "err");
    toast(`Purge failed: ${e}`, "err", 6000);
  }
}

function maybeCelebrate(bytesFreed, items) {
  const count = items.length;
  if (!bytesFreed || bytesFreed < 100 * 1024 * 1024) return;
  if (!els.celebrationModal) return;
  const phrases = [
    "Your SSD just felt a breeze.",
    "That's enough room for another 4K game.",
    "Your disk says thanks.",
    "A small act of digital hygiene.",
    "Future-you sends regards.",
  ];
  const phrase = phrases[Math.floor(Math.random() * phrases.length)];
  els.celebrationTitle.textContent = `You cleaned ${humanSize(bytesFreed)}!`;
  els.celebrationSub.textContent = count > 1 ? `${count} items gone. Your SSD thanks you.` : "Your SSD thanks you.";
  els.celebrationQuote.textContent = `"${phrase}"`;
  const byCat = {};
  for (const o of items) {
    byCat[o.category] = (byCat[o.category] || 0) + o.size_bytes;
  }
  const topCats = Object.entries(byCat).sort((a, b) => b[1] - a[1]).slice(0, 4);
  els.celebrationStats.innerHTML = topCats.map(([k, v]) => `<div class="celebration-stat"><div class="celebration-stat-key">${escapeHtml(k)}</div><div class="celebration-stat-val accent">${humanSize(v)}</div></div>`).join("");
  showModal(els.celebrationModal);
  setTimeout(() => startConfetti(), 100);
  setTimeout(() => { try { navigator.vibrate && navigator.vibrate([80, 40, 80]); } catch (e) {} }, 50);
}

let confettiAnim = null;
function startConfetti() {
  const canvas = els.confettiCanvas;
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  const rect = canvas.parentElement.getBoundingClientRect();
  canvas.width = rect.width;
  canvas.height = rect.height;
  const colors = ["#6ee7b7", "#34d399", "#38bdf8", "#0ea5e9", "#fbbf24", "#f87171", "#a78bfa"];
  const particles = [];
  for (let i = 0; i < 140; i++) {
    particles.push({
      x: Math.random() * canvas.width,
      y: -20 - Math.random() * canvas.height,
      vx: (Math.random() - 0.5) * 3,
      vy: 2 + Math.random() * 4,
      size: 4 + Math.random() * 6,
      color: colors[Math.floor(Math.random() * colors.length)],
      rot: Math.random() * Math.PI * 2,
      vr: (Math.random() - 0.5) * 0.2,
      shape: Math.random() < 0.5 ? "rect" : "circle",
    });
  }
  let frames = 0;
  function frame() {
    frames++;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    for (const p of particles) {
      p.x += p.vx;
      p.y += p.vy;
      p.vy += 0.08;
      p.rot += p.vr;
      ctx.save();
      ctx.translate(p.x, p.y);
      ctx.rotate(p.rot);
      ctx.fillStyle = p.color;
      if (p.shape === "rect") {
        ctx.fillRect(-p.size / 2, -p.size / 4, p.size, p.size / 2);
      } else {
        ctx.beginPath();
        ctx.arc(0, 0, p.size / 2, 0, Math.PI * 2);
        ctx.fill();
      }
      ctx.restore();
    }
    if (frames < 320) confettiAnim = requestAnimationFrame(frame);
    else { ctx.clearRect(0, 0, canvas.width, canvas.height); }
  }
  if (confettiAnim) cancelAnimationFrame(confettiAnim);
  confettiAnim = requestAnimationFrame(frame);
}

function stopConfetti() {
  if (confettiAnim) cancelAnimationFrame(confettiAnim);
  confettiAnim = null;
  if (els.confettiCanvas) {
    const ctx = els.confettiCanvas.getContext("2d");
    if (ctx) ctx.clearRect(0, 0, els.confettiCanvas.width, els.confettiCanvas.height);
  }
}

function toggleSelect(id, on) {
  if (on) state.selected.add(id);
  else state.selected.delete(id);
  document.querySelectorAll(".orphan").forEach((el) => {
    const cb = el.querySelector("input");
    el.classList.toggle("selected", cb && cb.checked);
  });
  updateSelectedReadout();
}

function bulkSelect(cat) {
  if (!state.report) return;
  const list = getFilteredOrphans();
  if (cat === "all") { list.forEach((o) => state.selected.add(o.id)); }
  else if (cat === "none") { state.selected.clear(); }
  else if (cat === "invert") { list.forEach((o) => { if (state.selected.has(o.id)) state.selected.delete(o.id); else state.selected.add(o.id); }); }
  else { list.forEach((o) => { if (o.category === cat) state.selected.add(o.id); }); }
  renderOrphans();
}

function updateSelectedReadout() {
  if (!state.report) return;
  const orphans = state.report.orphaned_files.filter((o) => state.selected.has(o.id));
  const bytes = orphans.reduce((s, o) => s + o.size_bytes, 0);
  els.selectedCount.textContent = orphans.length.toString();
  els.selectedTotal.textContent = humanSize(bytes);
}

function populateMigrate(report) {
  const games = report.installed_games;
  els.migrateGame.innerHTML = "";
  for (const g of games) {
    const o = document.createElement("option");
    o.value = g.id;
    o.textContent = g.name;
    els.migrateGame.appendChild(o);
  }
  els.migrateBtn.disabled = games.length === 0;
}

async function pickBackup() {
  const picked = await openDialogFolder("Pick backup folder");
  if (picked) {
    document.getElementById("backup-path").value = picked;
    if (state.settings) { state.settings.backup_folder = picked; await saveSettingsSilent(); }
  }
}

document.querySelectorAll('input[name="strategy"]').forEach((r) => {
  r.addEventListener("change", () => {
    const row = document.getElementById("backup-folder-row");
    if (row) row.hidden = r.value !== "backup_folder";
  });
});

function getStrategy() {
  const sel = document.querySelector('input[name="strategy"]:checked');
  if (!sel) return { type: "recycle_bin" };
  if (sel.value === "recycle_bin") return { type: "recycle_bin" };
  if (sel.value === "direct_delete") return { type: "direct_delete" };
  if (sel.value === "backup_folder") {
    const path = document.getElementById("backup-path").value.trim();
    if (path) return { type: "backup_folder", path };
    toast("Pick a backup folder first", "warn");
    return { type: "recycle_bin" };
  }
  return { type: "recycle_bin" };
}

function getSelectedOrphans() {
  if (!state.report) return [];
  return state.report.orphaned_files.filter((o) => state.selected.has(o.id));
}

function purge(orphans, strategy) {
  return invoke("purge_orphans", { paths: orphans.map((o) => o.path), strategy });
}

async function confirmAndPurge(orphans, strategy) {
  if (!orphans.length) return;
  const total = orphans.reduce((s, o) => s + o.size_bytes, 0);
  const verb = strategy.type === "recycle_bin" ? "Recycle" : strategy.type === "direct_delete" ? "Force delete" : "Move to backup";
  const ok = await confirm({
    title: `${verb} ${orphans.length} item(s)?`,
    body: strategy.type === "direct_delete"
      ? `⚠ Force delete will permanently remove ${humanSize(total)}. Use this for locked shader/cache files.`
      : `This will free ${humanSize(total)}. Files ${strategy.type === "recycle_bin" ? "go to the Recycle Bin" : "are moved to " + strategy.path} and can be restored.`,
    detail: orphans.slice(0, 20).map((o) => `<li class="size">${escapeHtml(humanSize(o.size_bytes))}</li> <li>${escapeHtml(o.game_hint)} — ${escapeHtml(o.path)}</li>`).join(""),
  });
  if (!ok) return;
  setStatus("Purging…", "busy");
  try {
    const report = await purge(orphans, strategy);
    state.lastPurge = { report, orphans: [...orphans], strategy, time: new Date().toISOString() };
    state.lastPurgeErrors = report.errors;
    try { localStorage.setItem(LAST_PURGE_KEY, JSON.stringify(state.lastPurge)); } catch (e) {}
    pushHistory({ type: "purge", message: `${verb} ${report.moved.length} item(s)`, bytes: report.bytes_freed });
    setStatus(`Done. ${report.moved.length} moved, ${report.errors.length} errors.`, report.errors.length ? "warn" : "ok");
    toast(`Freed ${humanSize(report.bytes_freed)}`, "ok");
    if (report.errors.length) {
      const msg = `${report.errors.length} item(s) couldn't be deleted — likely locked. Try Force Delete or close running apps.`;
      toast(msg, "warn", 8000);
      showPurgeErrors(report.errors);
    }
    showUndoToast(report, orphans);
    maybeCelebrate(report.bytes_freed, orphans);
    state.selected.clear();
    await runScan(state.report.mode || "standard");
  } catch (e) {
    setStatus(`Purge failed: ${e}`, "err");
    toast(`Purge failed: ${e}`, "err", 6000);
  }
}

function showPurgeErrors(errors) {
  const ok = confirm({
    title: `${errors.length} item(s) couldn't be deleted`,
    body: "Some files are in use (likely by the GPU driver or a running game). Switch to Force Delete in the Backup & Migrate screen, or close the apps using them.",
    detail: errors.slice(0, 10).map((e) => `<li>${escapeHtml(e.path)}<br><span class="muted small">${escapeHtml(e.message)}</span></li>`).join(""),
  });
}

function showUndoToast(report, orphans) {
  if (state.lastPurge && state.lastPurge.strategy.type === "recycle_bin") {
    const div = document.createElement("div");
    div.className = "undo-toast";
    div.innerHTML = `<span>Freed ${humanSize(report.bytes_freed)}</span><button id="undo-btn">↩ Undo (open Recycle Bin)</button>`;
    els.toastHost.appendChild(div);
    div.querySelector("#undo-btn").addEventListener("click", async () => {
      try { await invoke("open_recycle_bin"); toast("Recycle Bin opened — restore the items you want", "ok", 4000); }
      catch (e) { toast(`Could not open Recycle Bin: ${e}`, "err"); }
      div.remove();
    });
    setTimeout(() => { if (div.parentNode) div.remove(); }, 12000);
  }
}

async function smartClean() {
  if (!state.report) { toast("Run a scan first", "warn"); return; }
  const cats = (state.settings && state.settings.smart_clean_categories) || ["cache", "shaders", "crashes"];
  const targets = state.report.orphaned_files.filter(
    (o) => cats.includes(o.category) && o.confidence >= ((state.settings && state.settings.min_confidence) || 0)
  );
  if (!targets.length) { toast("Nothing safe to clean in this scan", "warn"); return; }
  targets.forEach((t) => state.selected.add(t.id));
  renderOrphans();
  await confirmAndPurge(targets, getStrategy());
}

async function planMigration() {
  const gameId = els.migrateGame.value;
  if (!gameId || !state.report) return;
  const game = state.report.installed_games.find((g) => g.id === gameId);
  if (!game) return;
  const from = document.getElementById("migrate-from").value;
  const to = document.getElementById("migrate-to").value;
  if (from === to) { toast("Source and target are the same", "warn"); return; }
  const estimated = game.estimated_save_paths.length;
  const msg = `Plan for ${game.name}:\n\nFrom: ${from}\nTo:   ${to}\n\nKnown save location(s): ${estimated === 0 ? "(none found)" : estimated + " folder(s)"}\n` +
    (game.estimated_save_paths.length ? game.estimated_save_paths.map((p) => "  - " + p).join("\n") : "  No estimated save paths for this game. Manual review required.");
  const ok = await confirm({ title: "Migration plan", body: msg });
  if (ok) {
    pushHistory({ type: "migrate", message: `Plan: ${game.name} ${from}→${to}` });
    toast("Migration plan recorded. Use the source paths to copy your saves.", "ok", 6000);
  }
}

function confirm({ title, body, detail }) {
  return new Promise((resolve) => {
    els.confirmTitle.textContent = title;
    els.confirmBody.textContent = body;
    if (detail) els.confirmDetail.innerHTML = detail;
    else els.confirmDetail.innerHTML = "";
    showModal(els.confirmModal);
    state._confirmCb = () => resolve(true);
    els.confirmCancel.onclick = () => { hideModal(els.confirmModal); resolve(false); };
  });
}

function showModal(m) { m.hidden = false; const btn = m.querySelector(".modal-close"); if (btn) setTimeout(() => btn.focus(), 50); }
function hideModal(m) { m.hidden = true; }

document.querySelectorAll(".modal-close").forEach((btn) => {
  btn.addEventListener("click", () => {
    const id = btn.getAttribute("data-close");
    const m = document.getElementById(id);
    if (m) hideModal(m);
  });
});
document.querySelectorAll(".modal").forEach((m) => {
  m.addEventListener("click", (e) => {
    if (e.target === m) hideModal(m);
  });
});

function humanSize(n) {
  if (n == null || n === undefined) return "--";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  let v = Number(n);
  while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
  return `${v.toFixed(v >= 100 || i === 0 ? 0 : 1)} ${units[i]}`;
}

function formatTime(iso) {
  if (!iso) return "Never";
  try { return new Date(iso).toLocaleString(); }
  catch { return iso; }
}

function formatRelative(iso) {
  if (!iso) return "";
  try {
    const t = new Date(iso).getTime();
    const diff = Date.now() - t;
    if (diff < 0) return "just now";
    const days = Math.floor(diff / 86400000);
    if (days === 0) return "today";
    if (days === 1) return "yesterday";
    if (days < 7) return `${days} days ago`;
    if (days < 30) return `${Math.floor(days / 7)}w ago`;
    if (days < 365) return `${Math.floor(days / 30)}mo ago`;
    return `${Math.floor(days / 365)}y ago`;
  } catch { return iso; }
}

function escapeHtml(s) { return String(s ?? "").replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c])); }

function pushHistory(entry) {
  entry.time = new Date().toISOString();
  state.history.unshift(entry);
  if (state.history.length > MAX_HISTORY) state.history.length = MAX_HISTORY;
  saveHistory();
  renderActivityFeed();
}

function saveHistory() { try { localStorage.setItem(HISTORY_KEY, JSON.stringify(state.history)); } catch (e) { /* ignore */ } }
function loadHistory() { try { const raw = localStorage.getItem(HISTORY_KEY); if (raw) state.history = JSON.parse(raw); } catch (e) { state.history = []; } }

function saveLastScan() {
  try { if (state.report) localStorage.setItem(LAST_SCAN_KEY, JSON.stringify(state.report)); } catch (e) { /* ignore */ }
}
function loadLastScan() {
  try { const raw = localStorage.getItem(LAST_SCAN_KEY); if (raw) { state.report = JSON.parse(raw); renderReport(state.report); } }
  catch (e) { /* ignore */ }
}

function renderHistory() {
  els.historyList.innerHTML = "";
  if (!state.history.length) {
    const li = document.createElement("li");
    li.className = "muted small";
    li.textContent = "No activity yet.";
    els.historyList.appendChild(li);
    return;
  }
  for (const h of state.history) {
    const li = document.createElement("li");
    li.className = "history-item";
    const bytes = h.bytes ? ` · ${humanSize(h.bytes)}` : "";
    li.innerHTML = `<span class="history-dot ${h.type}"></span><span class="history-msg">${escapeHtml(h.message)}${bytes}</span><span class="history-time">${formatTime(h.time)}</span>`;
    els.historyList.appendChild(li);
  }
}

function renderActivityFeed() {
  els.activityFeed.innerHTML = "";
  const items = state.history.slice(0, 8);
  if (!items.length) {
    const li = document.createElement("li");
    li.className = "muted small";
    li.textContent = "No activity yet — run a scan to begin.";
    els.activityFeed.appendChild(li);
    return;
  }
  for (const h of items) {
    const li = document.createElement("li");
    li.className = "activity-item";
    const bytes = h.bytes ? ` · ${humanSize(h.bytes)}` : "";
    li.innerHTML = `<span class="activity-dot ${h.type}"></span><span>${escapeHtml(h.message)}${bytes}</span><span class="muted small">${formatRelative(h.time)}</span>`;
    els.activityFeed.appendChild(li);
  }
}

function renderMiniList() {
  els.miniList.innerHTML = "";
  if (!state.report || !state.report.orphaned_files.length) {
    const li = document.createElement("li");
    li.className = "muted small";
    li.textContent = "No candidates yet.";
    els.miniList.appendChild(li);
    return;
  }
  for (const o of state.report.orphaned_files.slice(0, 6)) {
    const li = document.createElement("li");
    li.className = "mini-item";
    li.innerHTML = `<span class="mini-name">${escapeHtml(o.game_hint)}</span><span class="mini-size">${humanSize(o.size_bytes)}</span>`;
    li.addEventListener("click", () => openDetail(o));
    els.miniList.appendChild(li);
  }
}

function renderLibrary() {
  els.libraryGrid.innerHTML = "";
  if (!state.report || !state.report.installed_games.length) {
    els.libEmpty.hidden = false;
    return;
  }
  els.libEmpty.hidden = true;
  const q = (els.libSearch.value || "").toLowerCase();
  const list = state.report.installed_games.filter((g) => !q || g.name.toLowerCase().includes(q) || (g.app_id || "").includes(q));
  for (const g of list) {
    const card = document.createElement("div");
    card.className = "game-card";
    const initial = g.name.charAt(0).toUpperCase();
    let cover = `<div class="game-cover"><span class="game-cover-initial">${initial}</span></div>`;
    if (g.source === "steam" && g.app_id) {
      const url = `https://cdn.akamai.steamstatic.com/steam/apps/${g.app_id}/library_600x900_2x.jpg`;
      cover = `<div class="game-cover has-image" style="background-image:url('${url}')"><span class="game-cover-initial">${initial}</span></div>`;
    }
    card.innerHTML = `
      ${cover}
      <div class="game-info">
        <div class="game-name">${escapeHtml(g.name)}</div>
        <div class="game-meta">
          <span class="game-source-tag ${g.source}">${g.source}</span>
          <span>${humanSize(g.size_bytes)}</span>
          ${g.app_id ? `<span>app ${g.app_id}</span>` : ""}
        </div>
      </div>
    `;
    card.addEventListener("click", () => openGameDetail(g));
    if (g.source === "steam" && g.app_id) {
      card.addEventListener("contextmenu", (e) => { e.preventDefault(); invoke("open_steam_app", { appid: g.app_id }).catch(() => {}); });
    }
    els.libraryGrid.appendChild(card);
  }
}

function renderInsights() {
  if (!state.report || !state.report.orphaned_files.length) {
    els.insightEmpty.hidden = false;
    els.chartCategory.innerHTML = "<p class='muted small'>No data yet.</p>";
    els.chartPublisher.innerHTML = "<p class='muted small'>No data yet.</p>";
    els.chartConfidence.innerHTML = "<p class='muted small'>No data yet.</p>";
    els.chartTop.innerHTML = "<p class='muted small'>No data yet.</p>";
    return;
  }
  els.insightEmpty.hidden = true;
  els.insightTotal.textContent = humanSize(state.report.total_reclaimable_bytes);
  els.insightCount.textContent = state.report.orphaned_files.length;
  els.insightSub.textContent = `reclaimable across ${state.report.orphaned_files.length} items`;
  renderCategoryChart();
  renderPublisherChart();
  renderConfidenceChart();
  renderTopChart();
}

function renderCategoryChart() {
  const data = state.report.category_breakdown;
  const total = data.reduce((s, d) => s + d.bytes, 0) || 1;
  let svg = `<div class="donut-wrap">`;
  let cumulative = 0;
  const colors = ["#6ee7b7", "#38bdf8", "#fbbf24", "#f87171", "#a78bfa", "#34d399", "#fb923c"];
  const radius = 60;
  const stroke = 22;
  svg += `<svg class="donut-svg" width="140" height="140" viewBox="0 0 140 140">`;
  data.forEach((d, i) => {
    const pct = d.bytes / total;
    const len = 2 * Math.PI * radius;
    const offset = len * cumulative;
    svg += `<circle cx="70" cy="70" r="${radius}" fill="none" stroke="${colors[i % colors.length]}" stroke-width="${stroke}" stroke-dasharray="${len * pct} ${len}" stroke-dashoffset="${-offset}" transform="rotate(-90 70 70)" />`;
    cumulative += pct;
  });
  svg += `<text x="70" y="70" text-anchor="middle" dy="6" font-size="18" font-weight="700" fill="#e6ebf2">${humanSize(total)}</text>`;
  svg += `</svg><div class="donut-legend">`;
  data.forEach((d, i) => {
    const pct = ((d.bytes / total) * 100).toFixed(1);
    svg += `<div class="donut-legend-row"><span class="donut-swatch" style="background:${colors[i % colors.length]}"></span><span>${escapeHtml(d.category)}</span><span class="muted small" style="margin-left:auto">${humanSize(d.bytes)} · ${pct}%</span></div>`;
  });
  svg += `</div></div>`;
  els.chartCategory.innerHTML = svg;
}

function renderPublisherChart() {
  const data = state.report.publisher_breakdown.slice(0, 8);
  const total = data.reduce((s, d) => s + d.bytes, 0) || 1;
  els.chartPublisher.innerHTML = data.map((d) => {
    const pct = (d.bytes / total) * 100;
    return `<div class="bar-row">
      <span class="bar-name" title="${escapeHtml(d.publisher)}${d.is_game_studio ? " (game studio)" : ""}">${escapeHtml(d.publisher)}</span>
      <div class="bar-track"><div class="bar-fill" style="width:${pct.toFixed(1)}%"></div></div>
      <span class="bar-label">${humanSize(d.bytes)}</span>
    </div>`;
  }).join("") || "<p class='muted small'>No data.</p>";
}

function renderConfidenceChart() {
  const buckets = { "90-100%": 0, "75-89%": 0, "60-74%": 0, "<60%": 0 };
  let bucketBytes = { "90-100%": 0, "75-89%": 0, "60-74%": 0, "<60%": 0 };
  for (const o of state.report.orphaned_files) {
    if (o.confidence >= 90) { buckets["90-100%"]++; bucketBytes["90-100%"] += o.size_bytes; }
    else if (o.confidence >= 75) { buckets["75-89%"]++; bucketBytes["75-89%"] += o.size_bytes; }
    else if (o.confidence >= 60) { buckets["60-74%"]++; bucketBytes["60-74%"] += o.size_bytes; }
    else { buckets["<60%"]++; bucketBytes["<60%"] += o.size_bytes; }
  }
  const max = Math.max(...Object.values(buckets), 1);
  const colors = ["#6ee7b7", "#34d399", "#fbbf24", "#f87171"];
  els.chartConfidence.innerHTML = Object.entries(buckets).map(([k, v], i) => {
    const pct = (v / max) * 100;
    return `<div class="bar-row">
      <span class="bar-name">${k}</span>
      <div class="bar-track"><div class="bar-fill" style="width:${pct.toFixed(1)}%;background:${colors[i]}"></div></div>
      <span class="bar-label">${v} · ${humanSize(bucketBytes[k])}</span>
    </div>`;
  }).join("");
}

function renderTopChart() {
  const top = state.report.orphaned_files.slice(0, 8);
  const total = top.reduce((s, d) => s + d.size_bytes, 0) || 1;
  els.chartTop.innerHTML = top.map((d) => {
    const pct = (d.size_bytes / total) * 100;
    return `<div class="bar-row" style="cursor:pointer" data-id="${d.id}">
      <span class="bar-name">${escapeHtml(d.game_hint)}</span>
      <div class="bar-track"><div class="bar-fill" style="width:${pct.toFixed(1)}%"></div></div>
      <span class="bar-label">${humanSize(d.size_bytes)}</span>
    </div>`;
  }).join("");
  els.chartTop.querySelectorAll(".bar-row").forEach((row) => {
    row.addEventListener("click", () => {
      const id = row.dataset.id;
      const o = state.report.orphaned_files.find((x) => x.id === id);
      if (o) openDetail(o);
    });
  });
}

async function openDetail(o) {
  state.currentDetail = o;
  els.detailName.textContent = o.game_hint;
  els.detailPath.textContent = o.path;
  els.detailSize.textContent = humanSize(o.size_bytes);
  els.detailMeta.innerHTML = `
    <span class="orphan-tag ${o.category}">${o.category}</span>
    <span class="orphan-confidence">confidence ${o.confidence}%</span>
    ${o.source ? `<span class="game-source-tag ${o.source}">${o.source}</span>` : ""}
    <span class="muted">${o.file_count.toLocaleString()} files</span>
    <span class="muted">${escapeHtml(o.reason)}</span>
  `;
  els.detailTree.innerHTML = `<p class="muted small">Loading file tree…</p>`;
  showModal(els.detailModal);
  try {
    const tree = await invoke("dir_tree", { path: o.path, maxDepth: 3, maxEntries: 200 });
    els.detailTree.innerHTML = renderTree(tree, 0);
  } catch (e) {
    els.detailTree.innerHTML = `<p class="muted small">Could not walk directory: ${escapeHtml(String(e))}</p>`;
  }
}

function renderTree(node, depth) {
  if (depth > 4) return "";
  const prefix = "&nbsp;".repeat(depth * 4) + (node.is_dir ? "📁 " : "📄 ");
  const truncated = node.name.length > 60 ? node.name.slice(0, 60) + "…" : node.name;
  const size = node.is_dir ? "" : `<span class="size">${humanSize(node.size)}</span>`;
  let html = `<div class="detail-tree-row"><span>${prefix}${escapeHtml(truncated)}</span><span></span>${size}</div>`;
  if (node.children) {
    for (const c of node.children) html += renderTree(c, depth + 1);
  }
  return html;
}

function openGameDetail(g) {
  state.currentDetail = {
    id: g.id, game_hint: g.name, category: "library", path: g.install_path,
    size_bytes: g.size_bytes, file_count: 0, reason: `${g.source} game`,
    last_modified: null, confidence: 100, source: g.source,
  };
  els.detailName.textContent = g.name;
  els.detailPath.textContent = g.install_path;
  els.detailSize.textContent = humanSize(g.size_bytes);
  els.detailMeta.innerHTML = `
    <span class="game-source-tag ${g.source}">${g.source}</span>
    ${g.app_id ? `<span class="muted">app ${g.app_id}</span>` : ""}
    <span class="muted">Estimated save locations: ${g.estimated_save_paths.length}</span>
  `;
  els.detailTree.innerHTML = `<p class="muted small">Loading file tree…</p>`;
  showModal(els.detailModal);
  invoke("dir_tree", { path: g.install_path, maxDepth: 2, maxEntries: 200 })
    .then((tree) => { els.detailTree.innerHTML = renderTree(tree, 0); })
    .catch((e) => { els.detailTree.innerHTML = `<p class="muted small">Error: ${escapeHtml(String(e))}</p>`; });
}

async function revealPath(p) {
  try { await invoke("open_in_explorer", { path: p }); }
  catch (e) { toast(`Open failed: ${e}`, "err"); }
}

function whitelistOrphan(o) {
  if (!state.settings.whitelist.names.includes(o.game_hint)) {
    state.settings.whitelist.names.push(o.game_hint);
    saveSettingsSilent().then(() => { toast(`Whitelisted "${o.game_hint}"`, "ok"); });
  }
}

function renderWhitelistEdit() {
  const wl = state.settings.whitelist;
  els.wlNames.innerHTML = wl.names.map((n) => `<li><span>${escapeHtml(n)}</span><button data-name="${escapeHtml(n)}">×</button></li>`).join("");
  els.wlPaths.innerHTML = wl.paths.map((p) => `<li><span>${escapeHtml(p)}</span><button data-path="${escapeHtml(p)}">×</button></li>`).join("");
  els.wlNames.querySelectorAll("button").forEach((b) => b.addEventListener("click", () => { wl.names = wl.names.filter((n) => n !== b.dataset.name); renderWhitelistEdit(); }));
  els.wlPaths.querySelectorAll("button").forEach((b) => b.addEventListener("click", () => { wl.paths = wl.paths.filter((p) => p !== b.dataset.path); renderWhitelistEdit(); }));
}

function addWl(field, input) {
  const v = input.value.trim();
  if (!v) return;
  if (!state.settings.whitelist[field].includes(v)) state.settings.whitelist[field].push(v);
  input.value = "";
  renderWhitelistEdit();
}
async function exportReport(fmt) {
  if (!state.report) { toast("No report to export", "warn"); return; }
  try {
    const stamp = new Date().toISOString().replace(/[:.]/g, "-").slice(0, 19);
    const ext = fmt === "csv" ? "csv" : "json";
    const path = await invoke("export_report", { format: ext });
    let body;
    if (ext === "csv") {
      const rows = [["name", "category", "size_bytes", "confidence", "path", "reason"]];
      state.report.orphaned_files.forEach((o) => rows.push([o.game_hint, o.category, o.size_bytes, o.confidence, o.path, o.reason]));
      body = rows.map((r) => r.map((c) => `"${String(c).replace(/"/g, '""')}"`).join(",")).join("\n");
    } else {
      body = JSON.stringify(state.report, null, 2);
    }
    await invoke("write_export", { path, contents: body });
    pushHistory({ type: "scan", message: `Exported ${ext.toUpperCase()} to ${path}` });
    toast(`Exported to ${path}`, "ok", 6000);
    revealPath(path);
  } catch (e) {
    toast(`Export failed: ${e}`, "err", 6000);
  }
}

async function runEngineTests() {
  setStatus("Running engine tests…", "busy");
  try {
    const out = await invoke("run_tests");
    const passed = out.match(/test result: ok\. \d+ passed/);
    if (passed) { setStatus("Engine tests: PASSED", "ok"); toast(passed[0], "ok", 5000); }
    else { setStatus("Engine tests: FAILED", "err"); toast("Some tests failed", "err", 6000); console.log(out); }
  } catch (e) {
    setStatus(`Tests failed: ${e}`, "err");
  }
}

async function loadSettings() {
  try { state.settings = await invoke("get_settings"); }
  catch (e) {
    state.settings = { theme: "dark", backup_folder: null, auto_refresh_manifest: true, auto_scan_on_launch: false, smart_clean_categories: ["cache", "shaders", "crashes"], notifications_enabled: true, min_confidence: 50, scan_mode: "standard", whitelist: { paths: [], names: [], publishers: [] } };
  }
  applySettings();
}

function applySettings() {
  const s = state.settings || {};
  els.setTheme.value = s.theme || "dark";
  els.setBackup.value = s.backup_folder || "";
  els.setAuto.checked = !!s.auto_refresh_manifest;
  els.setAutoScan.checked = !!s.auto_scan_on_launch;
  els.setNotif.checked = !!s.notifications_enabled;
  els.setConf.value = s.min_confidence || 50;
  els.setConfVal.textContent = s.min_confidence || 50;
  els.setMode.value = s.scan_mode || "standard";
  els.backupPath.textContent = s.backup_folder || "Using Recycle Bin";
  document.documentElement.dataset.theme = s.theme || "dark";
  renderWhitelistEdit();
}

async function saveSettings() {
  const s = {
    theme: els.setTheme.value,
    backup_folder: els.setBackup.value || null,
    auto_refresh_manifest: els.setAuto.checked,
    auto_scan_on_launch: els.setAutoScan.checked,
    smart_clean_categories: ["cache", "shaders", "crashes"],
    notifications_enabled: els.setNotif.checked,
    min_confidence: parseInt(els.setConf.value, 10) || 50,
    scan_mode: els.setMode.value,
    whitelist: state.settings.whitelist,
  };
  try {
    await invoke("save_settings", { settings: s });
    state.settings = s;
    applySettings();
    hideModal(els.settingsModal);
    toast("Settings saved", "ok");
  } catch (e) {
    toast(`Save failed: ${e}`, "err");
  }
}

async function saveSettingsSilent() {
  try { await invoke("save_settings", { settings: state.settings }); } catch (e) { /* ignore */ }
}

function openSettings() { applySettings(); showModal(els.settingsModal); }

async function sendNotification(title, body) {
  try {
    if (window.__TAURI__ && window.__TAURI__.notification) {
      const { sendNotification: send } = window.__TAURI__.notification;
      if (send) await send({ title, body });
    } else {
      await invoke("plugin:notification|notify", { options: { title, body } });
    }
  } catch (e) { /* ignore */ }
}

async function refreshManifestInBackground() {
  try { await invoke("refresh_manifest"); loadManifestStatus(); } catch (e) { log("manifest refresh failed: " + e); }
}

async function loadManifestStatus() {
  try {
    const age = await invoke("manifest_age_days");
    if (age == null) { els.manifestPill.className = "manifest-pill warn"; els.manifestText.textContent = "Manifest: missing"; }
    else { els.manifestPill.className = "manifest-pill ok"; els.manifestText.textContent = `Manifest: ${age}d old`; }
  } catch (e) { els.manifestPill.className = "manifest-pill err"; els.manifestText.textContent = "Manifest: error"; }
}

(async function init() {
  setStatus("Initializing…", "busy");
  [els.confirmModal, els.settingsModal, els.detailModal, els.whitelistModal, els.shortcutsModal, els.celebrationModal, els.progress].forEach((m) => { if (m) m.hidden = true; });
  loadHistory();
  loadLastScan();
  await loadSettings();
  await loadManifestStatus();
  if (state.settings && state.settings.auto_refresh_manifest) refreshManifestInBackground();
  renderHistory();
  renderActivityFeed();
  renderOrphans();
  if (state.report) { renderReport(state.report); }
  setStatus("Ready", "ok");
  if (state.settings && state.settings.auto_scan_on_launch && !state.report) {
    setTimeout(() => runScan(state.settings.scan_mode || "standard"), 600);
  }
})();

listen("scan-progress", (event) => {
  const p = event.payload;
  if (!p) return;
  const pct = p.total > 0 ? (p.current / p.total) * 100 : 0;
  showProgress(true, p.stage.charAt(0).toUpperCase() + p.stage.slice(1), pct, p.message);
});
