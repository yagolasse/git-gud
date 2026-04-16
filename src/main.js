const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;

let repositories = []; // Array of { path, name, current_branch }
let activeTabIndex = -1;

const tabsContainer = document.getElementById("tabs-container");
const noRepoView = document.getElementById("no-repo-view");
const repoView = document.getElementById("repo-view");
const displayName = document.getElementById("display-name");
const displayPath = document.getElementById("display-path");
const displayBranch = document.getElementById("display-branch");

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

function setActiveTab(index) {
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
  }
}

function closeTab(index) {
  repositories.splice(index, 1);
  
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
});
