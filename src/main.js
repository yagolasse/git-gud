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
const changesList = document.getElementById("changes-list");

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
    renderChanges(repo.path);
  }
}

async function renderChanges(path) {
  changesList.innerHTML = "<li>Loading changes...</li>";
  try {
    const statuses = await invoke("get_repo_status", { path });
    changesList.innerHTML = "";
    
    if (statuses.length === 0) {
      changesList.innerHTML = "<li>No changes in this repository.</li>";
      return;
    }

    statuses.forEach(file => {
      const li = document.createElement("li");
      li.className = "change-item";
      
      const statusClass = `status-${file.status.toLowerCase()}`;
      
      li.innerHTML = `
        <span class="file-path">${file.path}</span>
        <span class="status-tag ${statusClass}">${file.status}</span>
      `;
      changesList.appendChild(li);
    });
  } catch (err) {
    changesList.innerHTML = `<li style="color: red;">Error: ${err}</li>`;
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
  loadFromStorage();
});
