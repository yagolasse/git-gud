/**
 * @file main.js
 * @description Frontend logic for Git Gud, handling UI interactions, 
 * repository state, and IPC with the Rust backend.
 */

const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;
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
const displayName = document.getElementById("display-name");
const displayPath = document.getElementById("display-path");
const displayBranch = document.getElementById("display-branch");
const unstagedList = document.getElementById("unstaged-list");
const stagedList = document.getElementById("staged-list");
const branchContainer = document.getElementById("branch-container");
const branchMenu = document.getElementById("branch-menu");
const menuRenameBranch = document.getElementById("menu-rename-branch");

const diffModal = document.getElementById("diff-modal");
const diffFilePath = document.getElementById("diff-file-path");
const diffContainer = document.getElementById("diff-container");
const closeDiffBtn = document.getElementById("close-diff-btn");

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
    checkLight.classList.add("hidden");
    checkDark.classList.remove("hidden");
  } else {
    document.body.classList.remove("dark-theme");
    checkLight.classList.remove("hidden");
    checkDark.classList.add("hidden");
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
    noRepoView.classList.remove("hidden");
    repoView.classList.add("hidden");
  } else {
    const repo = repositories[index];
    noRepoView.classList.add("hidden");
    repoView.classList.remove("hidden");
    
    displayName.textContent = repo.name;
    displayPath.textContent = repo.path;
    displayBranch.textContent = repo.current_branch;

    if (!repo.head_shorthand) {
      branchContainer.style.pointerEvents = "none";
      branchContainer.style.color = "#666";
    } else {
      branchContainer.style.pointerEvents = "auto";
      branchContainer.style.color = "inherit";
    }

    commitSubjectInput.value = "";
    commitBodyInput.value = "";
    updateCharCount();
    amendCheckbox.checked = false;

    refreshChanges();
  }
}

/** Tracks currently visible unstaged paths for the "Stage All" feature. */
let currentUnstagedPaths = [];
/** Tracks currently visible staged paths for the "Unstage All" feature. */
let currentStagedPaths = [];

/**
 * Fetches and displays the current Git status (staged/unstaged files) for the active repo.
 */
async function refreshChanges() {
  if (activeTabIndex === -1) return;
  const path = repositories[activeTabIndex].path;
  
  unstagedList.innerHTML = "<li>Loading...</li>";
  stagedList.innerHTML = "<li>Loading...</li>";
  
  try {
    const statuses = await invoke("get_repo_status", { path });
    unstagedList.innerHTML = "";
    stagedList.innerHTML = "";
    
    const unstaged = statuses.filter(s => !s.staged);
    const staged = statuses.filter(s => s.staged);

    currentUnstagedPaths = unstaged.map(s => s.path);
    currentStagedPaths = staged.map(s => s.path);

    if (unstaged.length === 0) unstagedList.innerHTML = "<li>No unstaged changes</li>";
    if (staged.length === 0) {
      stagedList.innerHTML = "<li>No staged changes</li>";
      commitBtn.disabled = true;
    } else {
      commitBtn.disabled = false;
    }

    unstaged.forEach(file => unstagedList.appendChild(createFileItem(file, false)));
    staged.forEach(file => stagedList.appendChild(createFileItem(file, true)));
  } catch (err) {
    console.error("Failed to fetch changes:", err);
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
    refreshChanges();
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
    refreshChanges();
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
  if (!confirm(`Are you sure you want to discard changes in ${filePaths.length} file(s)? This cannot be undone.`)) {
    return;
  }
  
  try {
    const repoPath = repositories[activeTabIndex].path;
    await invoke("discard_unstaged_changes", { repoPath, filePaths });
    refreshChanges();
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
    
    diffFilePath.textContent = `${file} (${staged ? "Staged" : "Unstaged"})`;
    diffContainer.innerHTML = "";
    
    if (!diff || diff.trim() === "") {
      diffContainer.innerHTML = "<div class=\"diff-line diff-line-context\">No changes to show (might be a new untracked file or binary).</div>";
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
    
    diffModal.classList.remove("hidden");
  } catch (err) {
    alert("Error fetching diff: " + err);
  }
}

/** Opens the rename branch modal. */
async function handleRenameBranch() {
  const repo = repositories[activeTabIndex];
  if (!repo || !repo.head_shorthand) return;

  oldBranchDisplay.textContent = repo.head_shorthand;
  newBranchInput.value = repo.head_shorthand;
  renameModal.classList.remove("hidden");
  newBranchInput.focus();
  branchMenu.classList.add("hidden");
}

/** Executes the branch rename via Rust. */
async function confirmRename() {
  const repo = repositories[activeTabIndex];
  const newName = newBranchInput.value.trim();
  
  if (!newName || newName === repo.head_shorthand) {
    renameModal.classList.add("hidden");
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
    displayBranch.textContent = newName;
    
    renameModal.classList.add("hidden");
  } catch (err) {
    alert("Error renaming branch: " + err);
  }
}

/**
 * Assembles and executes a Git commit.
 */
async function handleCommit() {
  const subject = commitSubjectInput.value.trim();
  if (!subject) {
    alert("Please enter a commit subject.");
    return;
  }

  const body = commitBodyInput.value.trim();
  const fullMessage = body ? `${subject}\n\n${body}` : subject;

  try {
    const repoPath = repositories[activeTabIndex].path;
    const amend = amendCheckbox.checked;
    await invoke("commit_changes", { repoPath, message: fullMessage, amend });
    
    commitSubjectInput.value = "";
    commitBodyInput.value = "";
    amendCheckbox.checked = false;
    updateCharCount();
    refreshChanges();
  } catch (err) {
    alert("Error committing: " + err);
  }
}

/** Updates the subject character counter UI. */
function updateCharCount() {
  const count = commitSubjectInput.length > 0 ? commitSubjectInput.value.length : 0;
  charCountDisplay.textContent = `${count} / 72`;
}

/**
 * Handles the "Amend" checkbox state change, fetching previous commit message if needed.
 */
async function handleAmendChange() {
  if (amendCheckbox.checked) {
    try {
      const repoPath = repositories[activeTabIndex].path;
      const fullMessage = await invoke("get_last_commit_message", { repoPath });
      
      const parts = fullMessage.split("\n\n");
      commitSubjectInput.value = parts[0].trim();
      commitBodyInput.value = parts.length > 1 ? parts.slice(1).join("\n\n").trim() : "";
      
      updateCharCount();
    } catch (err) {
      console.error("Failed to get last commit message:", err);
      amendCheckbox.checked = false;
    }
  } else {
    commitSubjectInput.value = "";
    commitBodyInput.value = "";
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
  document.getElementById("menu-open").addEventListener("click", handleOpenRepo);
  document.getElementById("stage-all-btn").addEventListener("click", () => stageFiles(currentUnstagedPaths));
  document.getElementById("unstage-all-btn").addEventListener("click", () => unstageFiles(currentStagedPaths));
  
  branchContainer.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    e.stopPropagation();
    branchMenu.classList.toggle("hidden");
  });

  document.addEventListener("click", () => {
    branchMenu.classList.add("hidden");
  });

  branchMenu.addEventListener("click", (e) => {
    e.stopPropagation();
  });

  menuRenameBranch.addEventListener("click", (e) => {
    e.stopPropagation();
    handleRenameBranch();
  });

  cancelRenameBtn.addEventListener("click", () => renameModal.classList.add("hidden"));
  confirmRenameBtn.addEventListener("click", confirmRename);

  closeDiffBtn.addEventListener("click", () => diffModal.classList.add("hidden"));
  
  // Close diff modal when clicking outside
  diffModal.addEventListener("click", (e) => {
    if (e.target === diffModal) {
      diffModal.classList.add("hidden");
    }
  });

  commitSubjectInput.addEventListener("input", updateCharCount);
  commitBtn.addEventListener("click", handleCommit);
  amendCheckbox.addEventListener("change", handleAmendChange);

  themeLightBtn.addEventListener("click", () => setTheme("light"));
  themeDarkBtn.addEventListener("click", () => setTheme("dark"));

  // Listen for background repository updates emitted by the Rust file watcher
  await listen("repo-updated", (event) => {
    const updatedPath = event.payload;
    
    if (activeTabIndex !== -1 && repositories[activeTabIndex].path === updatedPath) {
      invoke("open_repository", { path: updatedPath }).then(repoInfo => {
        repositories[activeTabIndex].current_branch = repoInfo.current_branch;
        repositories[activeTabIndex].head_shorthand = repoInfo.head_shorthand;
        displayBranch.textContent = repoInfo.current_branch;
        refreshChanges();
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
