// styleManager.js
// Handles user-defined theme overrides, cookies, and preset import/export

(function () {
    const styleKeys = [
        "bg",
        "panel",
        "accent",
        "accent-hover",
        "accent-variant",
        "accent-variant-hover",
        "border",
        "text",
        "muted",
        "radius",
        "scroll-thumb"
    ];

    const defaultTheme = {
        "bg": "#0d0e14",
        "panel": "#05060c",
        "accent": "#4da3ff",
        "accent-hover": "#3390ff",
        "accent-variant": "#1a73e8",
        "accent-variant-hover": "#1669c7",
        "border": "#242424",
        "text": "#eee",
        "muted": "#999",
        "radius": "10px",
        "scroll-thumb": "#333"
    };

    const styleManager = {
        loadFromCookies() {
            const theme = {};
            styleKeys.forEach(key => {
                const value = this.getCookie(`style_${key}`);
                if (value) {
                    theme[key] = value;
                    document.documentElement.style.setProperty(`--${key}`, value);
                }
            });
        },

        getCurrentStyles() {
            const styles = {};
            styleKeys.forEach(key => {
                styles[key] =
                    getComputedStyle(document.documentElement)
                        .getPropertyValue(`--${key}`)
                        .trim() || "";
            });
            return styles;
        },

        saveUserTheme(theme) {
            for (const [key, value] of Object.entries(theme)) {
                document.documentElement.style.setProperty(`--${key}`, value);
                this.setCookie(`style_${key}`, value, 365);
            }
            alert("Theme saved!");
        },

        resetTheme() {
            for (const [key, value] of Object.entries(defaultTheme)) {
                document.documentElement.style.setProperty(`--${key}`, value);
                this.setCookie(`style_${key}`, value, 365);
            }
            alert("Theme reset to default!");
        },

        exportPreset(name, theme) {
            const blob = new Blob([JSON.stringify(theme, null, 2)], { type: "application/json" });
            const link = document.createElement("a");
            link.href = URL.createObjectURL(blob);
            link.download = `${name}.json`;
            link.click();
        },

        importPreset(file) {
            const reader = new FileReader();
            reader.onload = (e) => {
                try {
                    const theme = JSON.parse(e.target.result);
                    this.saveUserTheme(theme);
                    alert("Preset loaded!");
                } catch (err) {
                    alert("Invalid preset file.");
                }
            };
            reader.readAsText(file);
        },

        getCookie(name) {
            const match = document.cookie.match(new RegExp("(^| )" + name + "=([^;]+)"));
            return match ? decodeURIComponent(match[2]) : null;
        },

        setCookie(name, value, days) {
            const d = new Date();
            d.setTime(d.getTime() + days * 24 * 60 * 60 * 1000);
            document.cookie = `${name}=${encodeURIComponent(value)};expires=${d.toUTCString()};path=/`;
        },
    };

    window.styleManager = styleManager;

    // Apply theme immediately when script loads
    styleManager.loadFromCookies();
})();
