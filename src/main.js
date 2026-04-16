const { invoke } = window.__TAURI__.core;

let greetInputEl;
let greetMsgEl;

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  greetMsgEl.textContent = await invoke("greet", { name: greetInputEl.value });
}

let repoInputEl;
let repoMsgEl;
let repoDetailsEl;
let repoNameEl;
let repoPathEl;
let repoBranchEl;

async function openRepository() {
  try {
    const info = await invoke("open_repository", { path: repoInputEl.value });
    
    // Display metadata
    repoNameEl.textContent = info.name;
    repoPathEl.textContent = info.path;
    repoBranchEl.textContent = info.current_branch;
    
    repoDetailsEl.style.display = "block";
    repoMsgEl.textContent = "Successfully opened repository!";
    repoMsgEl.style.color = "green";
  } catch (err) {
    repoDetailsEl.style.display = "none";
    repoMsgEl.textContent = "Error: " + err;
    repoMsgEl.style.color = "red";
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  repoInputEl = document.querySelector("#repo-input");
  repoMsgEl = document.querySelector("#repo-msg");
  repoDetailsEl = document.querySelector("#repo-details");
  repoNameEl = document.querySelector("#repo-name");
  repoPathEl = document.querySelector("#repo-path");
  repoBranchEl = document.querySelector("#repo-branch");

  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  document.querySelector("#repo-form").addEventListener("submit", (e) => {
    e.preventDefault();
    openRepository();
  });
});
