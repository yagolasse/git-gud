const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;

let repositories = []; // Array of { path, name, current_branch }
let activeTabIndex = -1;

const STORAGE_KEY = "git-gud-repos";

const tabsContainer = document.getElementById("tabs-container");
const noRepoView = document.getElementById("no-repo-view");
const repoView = document.getElementById("repo-view");
const displayName = document.getElementById("display-name");
const displayPath = document.getElementById("display-path");
const displayBranch = document.getElementById("display-branch");
const unstagedList = document.getElementById("unstaged-list");
const stagedList = document.getElementById("staged-list");

function saveToStorage() {
  const paths = repositories.map(r => r.path);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(paths));
}

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

async function handleOpenRepo() {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select a Git Repository"
    });

    if (selected) {
      const repoInfo = await invoke("open_repository", { path: selected });
      
      // Check if already open
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

    // Fetch and render changes
    refreshChanges();
  }
}

let currentUnstagedPaths = [];
let currentStagedPaths = [];

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
    if (staged.length === 0) stagedList.innerHTML = "<li>No staged changes</li>";

    unstaged.forEach(file => unstagedList.appendChild(createFileItem(file, false)));
    staged.forEach(file => stagedList.appendChild(createFileItem(file, true)));
  } catch (err) {
    console.error("Failed to fetch changes:", err);
  }
}

function createFileItem(file, isStaged) {
  const li = document.createElement("li");
  li.className = "change-item";
  
  const statusClass = `status-${file.status.toLowerCase()}`;
  const actionLabel = isStaged ? "Unstage" : "Stage";
  const actionFn = isStaged ? unstageFiles : stageFiles;
  
  li.innerHTML = `
    <div class="file-info">
      <span class="status-tag ${statusClass}">${file.status[0]}</span>
      <span class="file-path" title="${file.path}">${file.path}</span>
    </div>
    <button class="action-btn">${actionLabel}</button>
  `;
  
  li.querySelector(".action-btn").addEventListener("click", () => actionFn([file.path]));
  
  return li;
}

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

window.addEventListener("DOMContentLoaded", () => {
  document.getElementById("menu-open").addEventListener("click", handleOpenRepo);
  document.getElementById("stage-all-btn").addEventListener("click", () => stageFiles(currentUnstagedPaths));
  document.getElementById("unstage-all-btn").addEventListener("click", () => unstageFiles(currentStagedPaths));
  loadFromStorage();
});
