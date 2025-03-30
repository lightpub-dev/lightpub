document.addEventListener("DOMContentLoaded", () => {
    const root = document.documentElement;
    const themeToggleInput = document.getElementById("theme-toggle");
    const themeIcon = document.getElementById("theme-icon");
  
    if (!themeToggleInput || !themeIcon) return; // Prevent null errors
  
    const savedTheme = localStorage.getItem("theme");
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  
    function setTheme(theme) {
      root.setAttribute("data-theme", theme);
      localStorage.setItem("theme", theme);
      themeToggleInput.checked = (theme === "dark");
      themeIcon.className = theme === "dark" ? "fa fa-sun" : "fa fa-moon";
    }
  
    if (savedTheme) {
      setTheme(savedTheme);
    } else {
      setTheme(prefersDark ? "dark" : "light");
    }
  
    themeToggleInput.addEventListener("change", () => {
      const newTheme = themeToggleInput.checked ? "dark" : "light";
      setTheme(newTheme);
    });
  });
  