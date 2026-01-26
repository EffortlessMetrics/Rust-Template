from playwright.sync_api import sync_playwright, expect
import time

def run(playwright):
    browser = playwright.chromium.launch(headless=True)
    page = browser.new_page()

    print("Navigating to Dashboard...")
    # Retry logic or long timeout handled by playwright?
    # Better to just wait.
    page.goto("http://localhost:8080", timeout=60000)

    # 1. Dashboard
    print("Checking Dashboard...")
    dashboard_link = page.get_by_role("link", name="Dashboard", exact=True)
    expect(dashboard_link).to_have_attribute("aria-current", "page")

    # Check that Graph is NOT active
    graph_link = page.get_by_role("link", name="Graph", exact=True)
    expect(graph_link).not_to_have_attribute("aria-current", "page")

    # 2. Coverage
    print("Checking Coverage...")
    page.goto("http://localhost:8080/ui/coverage")
    coverage_link = page.get_by_role("link", name="AC Coverage", exact=True)
    expect(coverage_link).to_have_attribute("aria-current", "page")

    # Check Search Input Label
    print("Checking Search Input A11y...")
    search_input = page.get_by_label("Filter coverage")
    expect(search_input).to_be_visible()

    # Screenshot
    page.screenshot(path="verification/verification.png")
    print("Screenshot saved to verification/verification.png")

    browser.close()

with sync_playwright() as playwright:
    run(playwright)
