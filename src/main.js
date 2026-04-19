/**
 * @file main.js
 * @description Frontend logic for Git Gud, handling UI interactions, 
 * repository state, and IPC with the Rust backend.
 */

const { invoke } = window.__TAURI__.core;
const { open, ask } = window.__TAURI__.dialog;
const { listen } = window.__TAURI__.event;

/** 
 * List of currently open repositories.
 * @type {Array<{path: string, name: string, current_branch: string, head_shorthand: string}>} 
 */
let repositories = [];

/** 
 * Index of the currently active tab.
 * @type {number} 
 */
let activeTabIndex = -1;

/** Key used for persistent storage of open repo paths. */
const STORAGE_KEY = "git-gud-repos";
/** Key used for persistent storage of theme preference. */
const THEME_KEY = "git-gud-theme";

// --- DOM Elements ---
const tabsContainer = document.getElementById("tabs-container");
const noRepoView = document.getElementById("no-repo-view");
const repoView = document.getElementById("repo-view");
const displayBranch = document.getElementById("display-branch");
const unstagedList = document.getElementById("unstaged-list");
const stagedList = document.getElementById("staged-list");
const branchesList = document.getElementById("branches-list");
const remotesList = document.getElementById("remotes-list");
const stashesList = document.getElementById("stashes-list");
const branchContainer = document.getElementById("branch-container");
const branchMenu = document.getElementById("branch-menu");
const menuRenameBranch = document.getElementById("menu-rename-branch");

const diffFilePath = document.getElementById("diff-file-path");
const diffContainer = document.getElementById("diff-container");

const renameModal = document.getElementById("rename-modal");
const oldBranchDisplay = document.getElementById("old-branch-name-display");
const newBranchInput = document.getElementById("new-branch-name-input");
const confirmRenameBtn = document.getElementById("confirm-rename-btn");
const cancelRenameBtn = document.getElementById("cancel-rename-btn");

const commitSubjectInput = document.getElementById("commit-subject");
const commitBodyInput = document.getElementById("commit-body");
const charCountDisplay = document.getElementById("char-count");
const amendCheckbox = document.getElementById("amend-checkbox");
const commitBtn = document.getElementById("commit-btn");

const themeLightBtn = document.getElementById("theme-light");
const themeDarkBtn = document.getElementById("theme-dark");
const checkLight = document.getElementById("check-light");
const checkDark = document.getElementById("check-dark");

// --- Resizers ---
const resizerNav = document.getElementById("resizer-nav");
const resizerChanges = document.getElementById("resizer-changes");
const sidebarNav = document.querySelector(".sidebar-nav");
const sidebarChanges = document.querySelector(".sidebar-changes");

/**
 * Saves the current list of repository paths to localStorage.
 */
function saveToStorage() {
  const paths = repositories.map(r => r.path);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(paths));
}

/**
 * Loads saved repositories from localStorage and attempts to open them.
 */
async function loadFromStorage() {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved) {
    try {
      const paths = JSON.parse(saved);
      for (const path of paths) {
        try {
          const repoInfo = await invoke("open_repository", { path });
          repositories.push(repoInfo);
        } catch (e) {
          console.warn(`Failed to restore repo at ${path}:`, e);
        }
      }
      
      if (repositories.length > 0) {
        renderTabs();
        setActiveTab(0);
      }
    } catch (e) {
      console.error("Failed to parse saved repositories:", e);
    }
  }
}

/**
 * Initializes the theme based on saved preference or system default.
 */
function initTheme() {
  const savedTheme = localStorage.getItem(THEME_KEY) || "light";
  setTheme(savedTheme);
}

/**
 * Sets the application theme.
 * @param {'light' | 'dark'} theme 
 */
function setTheme(theme) {
  if (theme === "dark") {
    document.body.classList.add("dark-theme");
    if (checkLight) checkLight.classList.add("hidden");
    if (checkDark) checkDark.classList.remove("hidden");
  } else {
    document.body.classList.remove("dark-theme");
    if (checkLight) checkLight.classList.remove("hidden");
    if (checkDark) checkDark.classList.add("hidden");
  }
  localStorage.setItem(THEME_KEY, theme);
}

/**
 * Opens a native folder picker to select and open a new Git repository.
 */
async function handleOpenRepo() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select a Git Repository"
    });

    if (selected) {
      const repoInfo = await invoke("open_repository", { path: selected });
      
      const existingIndex = repositories.findIndex(r => r.path === repoInfo.path);
      if (existingIndex !== -1) {
        setActiveTab(existingIndex);
        return;
      }

      repositories.push(repoInfo);
      saveToStorage();
      renderTabs();
      setActiveTab(repositories.length - 1);
    }
  } catch (err) {
    console.error("Failed to open repo:", err);
    alert("Error opening repository: " + err);
  }
}

/**
 * Re-renders the tab bar based on the `repositories` array.
 */
function renderTabs() {
  if (!tabsContainer) return;
  tabsContainer.innerHTML = "";
  repositories.forEach((repo, index) => {
    const tab = document.createElement("div");
    tab.className = `tab ${index === activeTabIndex ? "active" : ""}`;
    tab.innerHTML = `
      <span>${repo.name}</span>
      <span class="tab-close" data-index="${index}">×</span>
    `;
    
    tab.addEventListener("click", (e) => {
      if (e.target.classList.contains("tab-close")) {
        closeTab(index);
      } else {
        setActiveTab(index);
      }
    });
    
    tabsContainer.appendChild(tab);
  });
}

/**
 * Switches the active view to the repository at the given index.
 * @param {number} index - The index in the `repositories` array.
 */
async function setActiveTab(index) {
  activeTabIndex = index;
  renderTabs();
  
  if (index === -1) {
    if (noRepoView) noRepoView.classList.remove("hidden");
    if (repoView) repoView.classList.add("hidden");
  } else {
    const repo = repositories[index];
    if (noRepoView) noRepoView.classList.add("hidden");
    if (repoView) repoView.classList.remove("hidden");
    
    if (displayBranch) displayBranch.textContent = repo.current_branch;

    if (commitSubjectInput) commitSubjectInput.value = "";
    if (commitBodyInput) commitBodyInput.value = "";
    updateCharCount();
    if (amendCheckbox) amendCheckbox.checked = false;

    // Reset diff view
    if (diffFilePath) diffFilePath.textContent = "Select a file to view changes";
    if (diffContainer) diffContainer.innerHTML = '<div class="diff-placeholder">No file selected</div>';

    refreshEverything();
  }
}

/** Tracks currently visible unstaged paths for the "Stage All" feature. */
let currentUnstagedPaths = [];
/** Tracks currently visible staged paths for the "Unstage All" feature. */
let currentStagedPaths = [];

/**
 * Refreshes all repository data (status, branches, remotes, stashes).
 */
async function refreshEverything() {
  if (activeTabIndex === -1) return;
  const repoPath = repositories[activeTabIndex].path;

  await Promise.all([
    refreshChanges(),
    refreshBranches(repoPath),
    refreshRemotes(repoPath),
    refreshStashes(repoPath)
  ]);
}

/**
 * Fetches and displays the current Git status (staged/unstaged files) for the active repo.
 */
async function refreshChanges() {
  if (activeTabIndex === -1) return;
  const path = repositories[activeTabIndex].path;
  
  if (unstagedList) unstagedList.innerHTML = "<li>Loading...</li>";
  if (stagedList) stagedList.innerHTML = "<li>Loading...</li>";
  
  try {
    const statuses = await invoke("get_repo_status", { path });
    if (unstagedList) unstagedList.innerHTML = "";
    if (stagedList) stagedList.innerHTML = "";
    
    const unstaged = statuses.filter(s => !s.staged);
    const staged = statuses.filter(s => s.staged);

    currentUnstagedPaths = unstaged.map(s => s.path);
    currentStagedPaths = staged.map(s => s.path);

    if (unstaged.length === 0 && unstagedList) unstagedList.innerHTML = '<li class="sidebar-item" style="color: var(--text-muted); font-style: italic;">No unstaged changes</li>';
    if (staged.length === 0) {
      if (stagedList) stagedList.innerHTML = '<li class="sidebar-item" style="color: var(--text-muted); font-style: italic;">No staged changes</li>';
      if (commitBtn) commitBtn.disabled = true;
    } else {
      if (commitBtn) commitBtn.disabled = false;
    }

    if (unstagedList) unstaged.forEach(file => unstagedList.appendChild(createFileItem(file, false)));
    if (stagedList) staged.forEach(file => stagedList.appendChild(createFileItem(file, true)));
  } catch (err) {
    console.error("Failed to fetch changes:", err);
  }
}

/**
 * Fetches and displays local/remote branches.
 */
async function refreshBranches(repoPath) {
  if (!branchesList) return;
  try {
    const branches = await invoke("get_branches", { repoPath });
    branchesList.innerHTML = "";
    
    // Helper to shorten full ref names for display
    function shortenUpstream(full) {
      if (full.startsWith('refs/remotes/')) return full.substring(13);
      if (full.startsWith('refs/heads/')) return full.substring(11);
      return full;
    }

    branches.forEach(branch => {
      const li = document.createElement("li");
      li.className = `sidebar-item ${branch.is_current ? "active" : ""}`;
      // Generate status icons for local branches with upstream
      let statusHtml = '';
      if (!branch.is_remote && branch.upstream) {
        const ahead = branch.ahead || 0;
        const behind = branch.behind || 0;
        if (ahead > 0 || behind > 0) {
          let icons = [];
          if (ahead > 0) icons.push(`↑${ahead}`);
          if (behind > 0) icons.push(`↓${behind}`);
          const shortUpstream = shortenUpstream(branch.upstream);
          statusHtml = `<span class="branch-status" title="${shortUpstream} (${icons.join(', ')})">${icons.join(' ')}</span>`;
        }
      }
      
      li.innerHTML = `
        <span class="item-icon">${branch.is_remote ? "☁" : ""}</span>
        <span class="branch-name">${branch.name}</span>
        ${statusHtml}
      `;
      
      // Double click to checkout
      li.addEventListener("dblclick", () => checkoutBranch(branch.name, branch.is_remote));
      
      branchesList.appendChild(li);
    });
  } catch (err) {
    console.error("Failed to fetch branches:", err);
  }
}

/**
 * Checks out a specific branch.
 * @param {string} branchName 
 * @param {boolean} isRemote 
 */
async function checkoutBranch(branchName, isRemote) {
  const confirmed = await ask(`Are you sure you want to checkout branch "${branchName}"? Any unsaved changes might be lost if they conflict.`, {
    title: 'Checkout Branch',
    kind: 'warning',
  });

  if (!confirmed) return;

  try {
    const repoPath = repositories[activeTabIndex].path;
    await invoke("checkout_branch", { repoPath, branchName, isRemote });
    refreshEverything();
  } catch (err) {
    alert("Error checking out branch: " + err);
  }
}

/**
 * Fetches and displays remotes.
 */
async function refreshRemotes(repoPath) {
  if (!remotesList) return;
  try {
    const remotes = await invoke("get_remotes", { repoPath });
    remotesList.innerHTML = "";
    
    remotes.forEach(remote => {
      const li = document.createElement("li");
      li.className = "sidebar-item";
      li.innerHTML = `
        <span class="item-icon">📡</span>
        <div style="display: flex; flex-direction: column;">
          <span>${remote.name}</span>
          <span class="item-url">${remote.url || "No URL"}</span>
        </div>
      `;
      remotesList.appendChild(li);
    });
  } catch (err) {
    console.error("Failed to fetch remotes:", err);
  }
}

/**
 * Fetches and displays stashes.
 */
async function refreshStashes(repoPath) {
  if (!stashesList) return;
  try {
    const stashes = await invoke("get_stashes", { repoPath });
    stashesList.innerHTML = "";
    
    if (stashes.length === 0) {
      stashesList.innerHTML = '<li class="sidebar-item" style="color: var(--text-muted); font-style: italic;">No stashes</li>';
      return;
    }

    stashes.forEach(stash => {
      const li = document.createElement("li");
      li.className = "sidebar-item";
      li.innerHTML = `
        <span class="item-icon">📦</span>
        <span>stash@{${stash.index}}: ${stash.message}</span>
      `;
      stashesList.appendChild(li);
    });
  } catch (err) {
    console.error("Failed to fetch stashes:", err);
  }
}

/**
 * Creates a DOM element for a single file in the change list.
 * @param {Object} file - The file status object from Rust.
 * @param {boolean} isStaged - Whether the file is currently staged.
 * @returns {HTMLElement}
 */
function createFileItem(file, isStaged) {
  const li = document.createElement("li");
  li.className = "change-item";
  
  const statusClass = `status-${file.status.toLowerCase()}`;
  const actionLabel = isStaged ? "Unstage" : "Stage";
  const actionFn = isStaged ? unstageFiles : stageFiles;
  
  let actionsHtml = `<button class="action-btn primary-action">${actionLabel}</button>`;
  
  if (!isStaged) {
    actionsHtml += `<button class="action-btn discard-btn">Discard</button>`;
  }
  
  li.innerHTML = `
    <div class="file-info">
      <span class="status-tag ${statusClass}">${file.status[0]}</span>
      <span class="file-path clickable-path" title="Click to view diff">${file.path}</span>
    </div>
    <div class="file-actions">
      ${actionsHtml}
    </div>
  `;
  
  li.querySelector(".clickable-path").addEventListener("click", () => showDiff(file.path, isStaged));
  li.querySelector(".primary-action").addEventListener("click", () => actionFn([file.path]));
  
  if (!isStaged) {
    li.querySelector(".discard-btn").addEventListener("click", () => discardUnstagedChanges([file.path]));
  }
  
  return li;
}

/**
 * Stages a list of files in the current repository.
 * @param {string[]} filePaths 
 */
async function stageFiles(filePaths) {
  if (filePaths.length === 0) return;
  try {
    const repoPath = repositories[activeTabIndex].path;
    await invoke("stage_files", { repoPath, filePaths });
    refreshEverything();
  } catch (err) {
    alert("Error staging files: " + err);
  }
}

/**
 * Unstages a list of files in the current repository.
 * @param {string[]} filePaths 
 */
async function unstageFiles(filePaths) {
  if (filePaths.length === 0) return;
  try {
    const repoPath = repositories[activeTabIndex].path;
    await invoke("unstage_files", { repoPath, filePaths });
    refreshEverything();
  } catch (err) {
    alert("Error unstaging files: " + err);
  }
}

/**
 * Discards unstaged changes for a list of files.
 * @param {string[]} filePaths 
 */
async function discardUnstagedChanges(filePaths) {
  if (filePaths.length === 0) return;
  
  const confirmed = await ask(`Are you sure you want to discard changes in ${filePaths.length} file(s)? This cannot be undone.`, {
    title: 'Discard Changes',
    kind: 'warning',
  });

  if (!confirmed) return;
  
  try {
    const repoPath = repositories[activeTabIndex].path;
    await invoke("discard_unstaged_changes", { repoPath, filePaths });
    refreshEverything();
  } catch (err) {
    alert("Error discarding changes: " + err);
  }
}

/**
 * Fetches and displays the diff for a specific file.
 * @param {string} file - The relative path of the file.
 * @param {boolean} staged - Whether to show the staged or unstaged diff.
 */
async function showDiff(file, staged) {
  try {
    const repoPath = repositories[activeTabIndex].path;
    const diff = await invoke("get_file_diff", { repoPath, filePath: file, staged });
    
    if (diffFilePath) diffFilePath.textContent = `${file} (${staged ? "Staged" : "Unstaged"})`;
    if (diffContainer) {
      diffContainer.innerHTML = "";
      
      if (!diff || diff.trim() === "") {
        diffContainer.innerHTML = "<div class=\"diff-placeholder\">No changes to show (might be a new untracked file or binary).</div>";
      } else {
        const lines = diff.split("\n");
        lines.forEach(line => {
          const div = document.createElement("div");
          div.className = "diff-line";
          
          if (line.startsWith("+")) {
            div.classList.add("diff-line-added");
          } else if (line.startsWith("-")) {
            div.classList.add("diff-line-removed");
          } else if (line.startsWith("@@")) {
            div.classList.add("diff-line-hunk");
          } else {
            div.classList.add("diff-line-context");
          }
          
          div.textContent = line;
          diffContainer.appendChild(div);
        });
      }
      diffContainer.scrollTop = 0;
    }
  } catch (err) {
    alert("Error fetching diff: " + err);
  }
}

/** Opens the rename branch modal. */
async function handleRenameBranch() {
  const repo = repositories[activeTabIndex];
  if (!repo || !repo.head_shorthand) return;

  if (oldBranchDisplay) oldBranchDisplay.textContent = repo.head_shorthand;
  if (newBranchInput) {
    newBranchInput.value = repo.head_shorthand;
    newBranchInput.focus();
  }
  if (renameModal) renameModal.classList.remove("hidden");
  if (branchMenu) branchMenu.classList.add("hidden");
}

/** Executes the branch rename via Rust. */
async function confirmRename() {
  const repo = repositories[activeTabIndex];
  if (!newBranchInput) return;
  const newName = newBranchInput.value.trim();
  
  if (!newName || newName === repo.head_shorthand) {
    if (renameModal) renameModal.classList.add("hidden");
    return;
  }

  try {
    await invoke("rename_branch", { 
      repoPath: repo.path, 
      oldName: repo.head_shorthand, 
      newName 
    });
    
    repo.current_branch = newName;
    repo.head_shorthand = newName;
    if (displayBranch) displayBranch.textContent = newName;
    
    if (renameModal) renameModal.classList.add("hidden");
    refreshBranches(repo.path);
  } catch (err) {
    alert("Error renaming branch: " + err);
  }
}

/**
 * Assembles and executes a Git commit.
 */
async function handleCommit() {
  if (!commitSubjectInput) return;
  const subject = commitSubjectInput.value.trim();
  if (!subject) {
    alert("Please enter a commit subject.");
    return;
  }

  const body = commitBodyInput ? commitBodyInput.value.trim() : "";
  const fullMessage = body ? `${subject}\n\n${body}` : subject;

  try {
    const repoPath = repositories[activeTabIndex].path;
    const amend = amendCheckbox ? amendCheckbox.checked : false;
    await invoke("commit_changes", { repoPath, message: fullMessage, amend });
    
    if (commitSubjectInput) commitSubjectInput.value = "";
    if (commitBodyInput) commitBodyInput.value = "";
    if (amendCheckbox) amendCheckbox.checked = false;
    updateCharCount();
    refreshEverything();
    
    if (diffFilePath) diffFilePath.textContent = "Select a file to view changes";
    if (diffContainer) diffContainer.innerHTML = '<div class="diff-placeholder">No file selected</div>';
  } catch (err) {
    alert("Error committing: " + err);
  }
}

/** Updates the subject character counter UI. */
function updateCharCount() {
  if (!commitSubjectInput || !charCountDisplay) return;
  const count = commitSubjectInput.value ? commitSubjectInput.value.length : 0;
  charCountDisplay.textContent = `${count} / 72`;
}

/**
 * Handles the "Amend" checkbox state change, fetching previous commit message if needed.
 */
async function handleAmendChange() {
  if (amendCheckbox && amendCheckbox.checked) {
    try {
      const repoPath = repositories[activeTabIndex].path;
      const fullMessage = await invoke("get_last_commit_message", { repoPath });
      
      const parts = fullMessage.split("\n\n");
      if (commitSubjectInput) commitSubjectInput.value = parts[0].trim();
      if (commitBodyInput) commitBodyInput.value = parts.length > 1 ? parts.slice(1).join("\n\n").trim() : "";
      
      updateCharCount();
    } catch (err) {
      console.error("Failed to get last commit message:", err);
      amendCheckbox.checked = false;
    }
  } else {
    if (commitSubjectInput) commitSubjectInput.value = "";
    if (commitBodyInput) commitBodyInput.value = "";
    updateCharCount();
  }
}

/**
 * Closes a repository tab.
 * @param {number} index 
 */
function closeTab(index) {
  repositories.splice(index, 1);
  saveToStorage();
  
  if (repositories.length === 0) {
    setActiveTab(-1);
  } else if (activeTabIndex === index) {
    setActiveTab(Math.max(0, index - 1));
  } else if (activeTabIndex > index) {
    setActiveTab(activeTabIndex - 1);
  } else {
    renderTabs();
  }
}

/**
 * Initializes all event listeners and background event listeners.
 */
async function setupEventListeners() {
  const menuOpen = document.getElementById("menu-open");
  if (menuOpen) menuOpen.addEventListener("click", handleOpenRepo);
  
  const discardAllBtn = document.getElementById("discard-all-btn");
  if (discardAllBtn) discardAllBtn.addEventListener("click", () => discardUnstagedChanges(currentUnstagedPaths));
  
  const stageAllBtn = document.getElementById("stage-all-btn");
  if (stageAllBtn) stageAllBtn.addEventListener("click", () => stageFiles(currentUnstagedPaths));
  
  const unstageAllBtn = document.getElementById("unstage-all-btn");
  if (unstageAllBtn) unstageAllBtn.addEventListener("click", () => unstageFiles(currentStagedPaths));
  
  // Sidebar collapsible logic
  document.querySelectorAll(".section-header").forEach(header => {
    header.addEventListener("click", () => {
      const section = header.parentElement;
      section.classList.toggle("collapsed");
    });
  });

  // Panel Resizer logic for Nav sidebar
  if (resizerNav && sidebarNav) {
    let isResizing = false;
    resizerNav.addEventListener("mousedown", () => { isResizing = true; document.body.style.cursor = "col-resize"; });
    document.addEventListener("mousemove", (e) => {
      if (!isResizing) return;
      const newWidth = e.clientX;
      if (newWidth > 100 && newWidth < 400) sidebarNav.style.width = `${newWidth}px`;
    });
    document.addEventListener("mouseup", () => { isResizing = false; document.body.style.cursor = "default"; });
  }

  // Panel Resizer logic for Changes sidebar
  if (resizerChanges && sidebarChanges) {
    let isResizing = false;
    resizerChanges.addEventListener("mousedown", () => { isResizing = true; document.body.style.cursor = "col-resize"; });
    document.addEventListener("mousemove", (e) => {
      if (!isResizing) return;
      const navWidth = sidebarNav ? sidebarNav.offsetWidth : 0;
      const newWidth = e.clientX - navWidth - 4; // Subtract nav sidebar and resizer width
      if (newWidth > 150 && newWidth < 500) sidebarChanges.style.width = `${newWidth}px`;
    });
    document.addEventListener("mouseup", () => { isResizing = false; document.body.style.cursor = "default"; });
  }

  if (branchContainer) {
    branchContainer.addEventListener("contextmenu", (e) => {
      e.preventDefault();
      e.stopPropagation();
      if (branchMenu) branchMenu.classList.toggle("hidden");
    });
  }

  document.addEventListener("click", () => {
    if (branchMenu) branchMenu.classList.add("hidden");
  });

  if (branchMenu) {
    branchMenu.addEventListener("click", (e) => {
      e.stopPropagation();
    });
  }

  if (menuRenameBranch) {
    menuRenameBranch.addEventListener("click", (e) => {
      e.stopPropagation();
      handleRenameBranch();
    });
  }

  if (cancelRenameBtn) cancelRenameBtn.addEventListener("click", () => renameModal.classList.add("hidden"));
  if (confirmRenameBtn) confirmRenameBtn.addEventListener("click", confirmRename);

  if (commitSubjectInput) commitSubjectInput.addEventListener("input", updateCharCount);
  if (commitBtn) commitBtn.addEventListener("click", handleCommit);
  if (amendCheckbox) amendCheckbox.addEventListener("change", handleAmendChange);

  if (themeLightBtn) themeLightBtn.addEventListener("click", () => setTheme("light"));
  if (themeDarkBtn) themeDarkBtn.addEventListener("click", () => setTheme("dark"));

  // Listen for background repository updates emitted by the Rust file watcher
  await listen("repo-updated", (event) => {
    const updatedPath = event.payload;
    
    if (activeTabIndex !== -1 && repositories[activeTabIndex].path === updatedPath) {
      invoke("open_repository", { path: updatedPath }).then(repoInfo => {
        repositories[activeTabIndex].current_branch = repoInfo.current_branch;
        repositories[activeTabIndex].head_shorthand = repoInfo.head_shorthand;
        if (displayBranch) displayBranch.textContent = repoInfo.current_branch;
        refreshEverything();
      });
    } else {
      const repoIndex = repositories.findIndex(r => r.path === updatedPath);
      if (repoIndex !== -1) {
        invoke("open_repository", { path: updatedPath }).then(repoInfo => {
          repositories[repoIndex].current_branch = repoInfo.current_branch;
          repositories[repoIndex].head_shorthand = repoInfo.head_shorthand;
        });
      }
    }
  });
}

// Bootstrap the application
window.addEventListener("DOMContentLoaded", () => {
  initTheme();
  setupEventListeners();
  loadFromStorage();
});
