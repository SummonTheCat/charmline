async function fetchJson(url, body = {}) {
    const res = await fetch(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body)
    });
    return res.json();
}

// Utility: Sort array of objects by numeric value desc, then key asc
function sortByKeynameAndValue(arr, keyName, valueName) {
    return arr.sort((a, b) => {
        const valDiff = (b[valueName] ?? 0) - (a[valueName] ?? 0);
        if (valDiff !== 0) return valDiff;

        const aKey = (a[keyName] || "").toString().toLowerCase();
        const bKey = (b[keyName] || "").toString().toLowerCase();
        return aKey.localeCompare(bKey);
    });
}

async function loadDashboard() {
    const [stats, companies, tags, solutions, sessionsByDay] = await Promise.all([
        fetchJson("/api/dashboard/stats"),
        fetchJson("/api/dashboard/top_companies", { limit: 5 }),
        fetchJson("/api/dashboard/tags"),
        fetchJson("/api/dashboard/solutions"),
        fetchJson("/api/dashboard/sessions_by_day", { days: 7 })
    ]);

    // Apply sorting
    const sortedCompanies = sortByKeynameAndValue(companies, "company", "session_count");
    const sortedTags = sortByKeynameAndValue(tags, "tag", "count");
    const sortedSolutions = sortByKeynameAndValue(solutions, "solution_type", "count");

    renderStats(stats);
    renderTable("companies-table", sortedCompanies, "company", "session_count", 5);
    renderTable("tags-table", sortedTags, "tag", "count", 5);
    renderTable("solutions-table", sortedSolutions, "solution_type", "count", 5);
    renderSessionsChart(sessionsByDay.sessions_by_day || []);
}

function renderStats(stats) {
    const cards = document.querySelectorAll(".stat-card h3");
    if (!stats) return;
    cards[0].textContent = stats.total_sessions ?? "—";
    cards[1].textContent = stats.sessions_today ?? "—";
    cards[2].textContent = stats.sessions_this_week ?? "—";
    cards[3].textContent = stats.unique_companies ?? "—";
    cards[4].textContent = stats.unique_callers ?? "—";
    cards[5].textContent = stats.avg_duration_seconds
        ? (stats.avg_duration_seconds / 60).toFixed(1) + "m"
        : "—";
}

function renderTable(id, rows, colA, colB, maxItems = 5) {
    const table = document.getElementById(id).querySelector("tbody");
    table.innerHTML = "";

    if (rows.length > maxItems) {
        rows = rows.slice(0, maxItems);
    }

    if (!rows || !rows.length) {
        table.innerHTML = "<tr><td colspan='2'>No data</td></tr>";
        return;
    }

    rows.forEach(r => {
        table.innerHTML += `<tr><td>${r[colA]}</td><td>${r[colB]}</td></tr>`;
    });
}

function renderSessionsChart(data) {
    const ctx = document.getElementById("sessionsChart").getContext("2d");
    const labels = data.map(d => d[0]);
    const values = data.map(d => d[1]);

    // Get accent color from CSS variable
    const accentColor = getComputedStyle(document.documentElement)
        .getPropertyValue('--accent')
        .trim() || '#4da3ff';

    // Convert HEX → RGBA with adjustable alpha (to darken/mute the fill)
    function hexToRgba(hex, alpha = 0.15) {
        let c = hex.replace('#', '');
        if (c.length === 3) c = c.split('').map(x => x + x).join('');
        const r = parseInt(c.slice(0, 2), 16);
        const g = parseInt(c.slice(2, 4), 16);
        const b = parseInt(c.slice(4, 6), 16);
        return `rgba(${r}, ${g}, ${b}, ${alpha})`;
    }

    const backgroundColor = hexToRgba(accentColor, 0.15);

    new Chart(ctx, {
        type: "line",
        data: {
            labels,
            datasets: [{
                label: "Sessions",
                data: values,
                fill: true,
                borderColor: accentColor,
                backgroundColor: backgroundColor,
                tension: 0.3,
                borderWidth: 2,
                pointRadius: 3,
                pointBackgroundColor: accentColor,
                pointHoverRadius: 5
            }]
        },
        options: {
            scales: {
                x: {
                    ticks: { color: "#aaa" },
                    grid: { color: "#222" }
                },
                y: {
                    ticks: { color: "#aaa" },
                    grid: { color: "#222" }
                }
            },
            plugins: {
                legend: { display: false }
            }
        }
    });
}

loadDashboard();
