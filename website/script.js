/* Progressive enhancement only: the page is complete without this file. */
(function () {
  "use strict";

  var LATEST = "https://github.com/pcrlnd/deew/releases/latest/download/";

  function detectPlatform() {
    var nav = navigator;
    var p =
      ((nav.userAgentData && nav.userAgentData.platform) || "") +
      " " + (nav.platform || "") + " " + (nav.userAgent || "");
    if (/android|iphone|ipad|ipod|cros/i.test(p)) return "other";
    if (/win/i.test(p)) return "windows";
    if (/linux|x11/i.test(p)) return "linux";
    return "other";
  }

  var platform = detectPlatform();

  /* Primary download button */
  var btn = document.getElementById("download-btn");
  if (btn && platform !== "other") {
    var win = platform === "windows";
    var asset = win ? "deew.exe" : "deew";
    btn.href = LATEST + asset;
    document.getElementById("download-label").textContent = win ? "Download for Windows" : "Download for Linux";
    document.getElementById("download-hint").textContent = asset;
    document.getElementById("download-icon").setAttribute("href", win ? "#i-windows" : "#i-linux");
    btn.querySelector("svg").classList.add("icon-fill");
  }

  /* Tabs */
  var tabs = Array.prototype.slice.call(document.querySelectorAll('[role="tab"]'));
  function select(tab, focus) {
    tabs.forEach(function (t) {
      var on = t === tab;
      t.setAttribute("aria-selected", on ? "true" : "false");
      t.tabIndex = on ? 0 : -1;
      var panel = document.getElementById(t.getAttribute("aria-controls"));
      panel.removeAttribute("data-js-hidden");
      panel.hidden = !on;
    });
    if (focus) tab.focus();
  }
  if (tabs.length) {
    tabs.forEach(function (tab, i) {
      tab.addEventListener("click", function () { select(tab, false); });
      tab.addEventListener("keydown", function (e) {
        var n = null;
        if (e.key === "ArrowRight") n = (i + 1) % tabs.length;
        else if (e.key === "ArrowLeft") n = (i - 1 + tabs.length) % tabs.length;
        else if (e.key === "Home") n = 0;
        else if (e.key === "End") n = tabs.length - 1;
        if (n === null) return;
        e.preventDefault();
        select(tabs[n], true);
      });
    });
    select(document.getElementById(platform === "linux" ? "tab-linux" : "tab-windows"), false);
  }

  /* Copy buttons */
  function setIcon(button, name) {
    var use = button.querySelector("use");
    use.setAttribute("href", "#i-" + name);
    button.classList.toggle("copied", name === "check");
  }
  Array.prototype.forEach.call(document.querySelectorAll(".copy-btn"), function (button) {
    var label = button.getAttribute("data-label") || "command";
    var status = button.parentNode.querySelector(".copy-status");
    var timer;
    button.addEventListener("click", function () {
      var code = button.parentNode.querySelector("code").textContent;
      if (!navigator.clipboard) return;
      navigator.clipboard.writeText(code).then(function () {
        setIcon(button, "check");
        button.setAttribute("aria-label", "Copied");
        status.textContent = "Copied to clipboard";
        clearTimeout(timer);
        timer = setTimeout(function () {
          setIcon(button, "copy");
          button.setAttribute("aria-label", "Copy " + label);
          status.textContent = "";
        }, 1800);
      }, function () { /* clipboard unavailable; ignore */ });
    });
  });

  /* Footer year */
  var year = document.getElementById("year");
  if (year) year.textContent = new Date().getFullYear();
})();
