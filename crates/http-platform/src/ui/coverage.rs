use axum::{extract::State, response::Html};
use maud::{Markup, html};
use spec_runtime::load_service_metadata;
use tracing::instrument;

use super::layout::layout;
use crate::PlatformState;

/// Coverage details page.
#[instrument(skip(state))]
pub async fn coverage_view<S>(State(state): State<S>) -> Html<String>
where
    S: PlatformState,
{
    let metadata =
        load_service_metadata(&state.workspace_root().join("specs/service_metadata.yaml")).ok();
    let content = coverage_content();

    Html(layout("AC Coverage", "coverage", &metadata, content).into_string())
}

/// Coverage page content markup.
fn coverage_content() -> Markup {
    html! {
        style { (coverage_styles()) }
        script { (coverage_script()) }

        .card data-uiid="coverage.summary" {
            h2 { "AC Coverage Summary" }
            .metrics {
                .metric style="border-left-color: #155724;" {
                    .metric-label { "Passing" }
                    .metric-value style="color: #155724;" id="passing-count" { "..." }
                }
                .metric style="border-left-color: #721c24;" {
                    .metric-label { "Failing" }
                    .metric-value style="color: #721c24;" id="failing-count" { "..." }
                }
                .metric style="border-left-color: #856404;" {
                    .metric-label { "Unknown" }
                    .metric-value style="color: #856404;" id="unknown-count" { "..." }
                }
                .metric {
                    .metric-label { "Total" }
                    .metric-value id="total-count" { "..." }
                }
            }
        }

        .card {
            h2 { "Acceptance Criteria Coverage" }
            .filter-controls data-uiid="coverage.filters" {
                button #filter-all.filter-btn onclick="filterData('all')" { "All" }
                button #filter-passing.filter-btn onclick="filterData('passing')" { "Passing" }
                button #filter-failing.filter-btn onclick="filterData('failing')" { "Failing" }
                button #filter-unknown.filter-btn onclick="filterData('unknown')" { "Unknown" }
                input #search-box.search-box type="text" placeholder="Search by AC ID or title..."
                    oninput="searchData()";
            }

            #table-container data-uiid="coverage.table" {
                table .coverage-table {
                    thead {
                        tr {
                            th { "AC ID" }
                            th { "Title" }
                            th { "Status" }
                            th { "Story" }
                            th { "Requirement" }
                            th { "Scenarios" }
                        }
                    }
                    tbody #coverage-tbody {
                        tr {
                            td colspan="6" style="text-align: center; padding: 2rem; color: #999;" {
                                "Loading coverage data..."
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Coverage page CSS styles.
fn coverage_styles() -> &'static str {
    r#"
    .filter-controls {
        margin-bottom: 1.5rem;
        display: flex;
        gap: 1rem;
        align-items: center;
        flex-wrap: wrap;
    }
    .filter-btn {
        padding: 0.5rem 1rem;
        border: 2px solid #667eea;
        background: white;
        color: #667eea;
        border-radius: 6px;
        cursor: pointer;
        font-weight: 500;
        transition: all 0.2s;
    }
    .filter-btn:hover {
        background: #667eea;
        color: white;
    }
    .filter-btn.active {
        background: #667eea;
        color: white;
    }
    .search-box {
        flex: 1;
        min-width: 250px;
        padding: 0.5rem 1rem;
        border: 2px solid #ddd;
        border-radius: 6px;
        font-size: 1rem;
    }
    .search-box:focus {
        outline: none;
        border-color: #667eea;
    }
    .coverage-table {
        width: 100%;
        border-collapse: collapse;
        background: white;
    }
    .coverage-table th {
        background: #f8f9fa;
        padding: 0.75rem;
        text-align: left;
        font-weight: 600;
        border-bottom: 2px solid #dee2e6;
        position: sticky;
        top: 0;
    }
    .coverage-table td {
        padding: 0.75rem;
        border-bottom: 1px solid #dee2e6;
        vertical-align: top;
    }
    .coverage-table tr:hover {
        background: #f8f9fa;
    }
    .ac-row {
        transition: opacity 0.2s;
    }
    .ac-row.hidden {
        display: none;
    }
    .scenario-list {
        margin: 0;
        padding-left: 1.5rem;
        font-size: 0.875rem;
    }
    .scenario-list li {
        margin: 0.25rem 0;
    }
    "#
}

/// Coverage page JavaScript.
fn coverage_script() -> &'static str {
    r#"
    let currentFilter = 'all';
    let allData = [];

    // Fetch coverage data on page load
    fetch('/platform/coverage')
        .then(res => res.json())
        .then(data => {
            allData = data.details;
            updateSummary(data.summary);
            renderTable(allData);
        })
        .catch(err => {
            console.error('Failed to load coverage data:', err);
            document.getElementById('table-container').innerHTML =
                '<p style="color: red;">Failed to load coverage data. Please try again.</p>';
        });

    function updateSummary(summary) {
        document.getElementById('passing-count').textContent = summary.passing;
        document.getElementById('failing-count').textContent = summary.failing;
        document.getElementById('unknown-count').textContent = summary.unknown;
        document.getElementById('total-count').textContent = summary.total;
    }

    function filterData(status) {
        currentFilter = status;

        // Update active button
        document.querySelectorAll('.filter-btn').forEach(btn => {
            btn.classList.remove('active');
        });
        document.getElementById('filter-' + status).classList.add('active');

        // Apply filter
        applyFilters();
    }

    function searchData() {
        applyFilters();
    }

    function applyFilters() {
        const searchTerm = document.getElementById('search-box').value.toLowerCase();
        const rows = document.querySelectorAll('.ac-row');

        rows.forEach(row => {
            const status = row.dataset.status;
            const text = row.textContent.toLowerCase();

            const statusMatch = currentFilter === 'all' || status === currentFilter;
            const searchMatch = searchTerm === '' || text.includes(searchTerm);

            if (statusMatch && searchMatch) {
                row.classList.remove('hidden');
            } else {
                row.classList.add('hidden');
            }
        });
    }

    function renderTable(data) {
        const tbody = document.getElementById('coverage-tbody');
        tbody.innerHTML = '';

        data.forEach(ac => {
            const row = document.createElement('tr');
            row.className = 'ac-row';
            row.dataset.status = ac.status;

            const statusBadge = ac.status === 'passing' ? '✅ pass' :
                               ac.status === 'failing' ? '❌ fail' :
                               '❓ unknown';
            const badgeClass = ac.status === 'passing' ? 'status-pass' :
                              ac.status === 'failing' ? 'status-fail' :
                              'status-unknown';

            const scenarios = ac.scenarios.length > 0
                ? '<ul class="scenario-list">' +
                  ac.scenarios.map(s => '<li>' + s + '</li>').join('') +
                  '</ul>'
                : '<em style="color: #999;">No scenarios</em>';

            row.innerHTML = `
                <td><code>${ac.id}</code></td>
                <td>${ac.title}</td>
                <td><span class="status-badge ${badgeClass}">${statusBadge}</span></td>
                <td><code>${ac.story}</code></td>
                <td><code>${ac.requirement}</code></td>
                <td>${scenarios}</td>
            `;

            tbody.appendChild(row);
        });
    }

    // Initialize with 'all' filter active
    window.addEventListener('DOMContentLoaded', () => {
        document.getElementById('filter-all').classList.add('active');
    });
    "#
}
