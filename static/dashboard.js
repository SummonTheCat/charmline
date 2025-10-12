async function fetchJson(url, body = {}) {
    const res = await fetch(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body)
    });
    return res.json();
}

async function loadDashboard() {
    const [stats, companies, tags, solutions, sessionsByDay] = await Promise.all([
        fetchJson("/api/dashboard/stats"),
        fetchJson("/api/dashboard/top_companies", { limit: 5 }),
        fetchJson("/api/dashboard/tags"),
        fetchJson("/api/dashboard/solutions"),
        fetchJson("/api/dashboard/sessions_by_day", { days: 7 })
    ]);

    renderStats(stats);
    renderTable("companies-table", companies, "company", "session_count");
    renderTable("tags-table", tags, "tag", "count");
    renderTable("solutions-table", solutions, "solution_type", "count");
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

function renderTable(id, rows, colA, colB) {
    const table = document.getElementById(id).querySelector("tbody");
    table.innerHTML = "";

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

    new Chart(ctx, {
        type: "line",
        data: {
            labels,
            datasets: [{
                label: "Sessions",
                data: values,
                fill: true,
                borderColor: "#4da3ff",
                backgroundColor: "rgba(77,163,255,0.15)",
                tension: 0.3
            }]
        },
        options: {
            scales: {
                x: { ticks: { color: "#aaa" }, grid: { color: "#222" } },
                y: { ticks: { color: "#aaa" }, grid: { color: "#222" } }
            },
            plugins: { legend: { display: false } }
        }
    });
}

loadDashboard();