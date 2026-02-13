import { test, expect } from '@playwright/test';

test.describe('Offset Explorer 3 - UI Tests', () => {
  test.beforeAll(async ({ page }) => {
    console.log('Starting UI tests for Offset Explorer 3...');

    // Navigate to application
    await page.goto(BASE_URL);
    console.log('Navigated to application');

    // Wait for page to load
    await page.waitForLoadState('domcontentloaded');

    // Take initial screenshot
    await page.screenshot({ path: 'screenshots/01-initial.png' });
    console.log('Took initial screenshot');

    // Test: Check if main navigation is present
    await expect(page.locator('text=Topics')).toBeVisible();
    console.log('✓ Topics navigation is visible');

    // Test: Check if server tree is present
    await expect(page.locator('text=Servers')).toBeVisible();
    console.log('✓ Servers section is visible');

    // Test: Check if Messages navigation is present
    await expect(page.locator('text=Messages')).toBeVisible();
    console.log('✓ Messages navigation is visible');

    // Test: Check if Consumers navigation is present
    await expect(page.locator('text=Consumers')).toBeVisible();
    console.log('✓ Consumers navigation is visible');

    // Test: Check if ACLs navigation is present
    await expect(page.locator('text=ACLs')).toBeVisible();
    console.log('✓ ACLs navigation is visible');

    // Test: Check if ZooKeeper navigation is present
    await expect(page.locator('text=ZooKeeper')).toBeVisible();
    console.log('✓ ZooKeeper navigation is visible');

    // Test navigation buttons
    const refreshBtn = page.locator('button:has-text("Refresh")').first();
    await expect(refreshBtn).toBeVisible();
    console.log('✓ Refresh button is visible');

    // Test main tabs
    const mainTabs = ['Topics', 'Messages', 'Consumers', 'ACLs', 'ZooKeeper'];
    for (const tab of mainTabs) {
      await expect(page.locator(`button:has-text("${tab}")`).first()).toBeVisible();
      console.log(`✓ ${tab} tab button is visible`);
    }

    console.log('All UI tests completed successfully!');
  });

  test.afterAll(async ({ page }) => {
    console.log('Cleaning up...');

    await page.close();
  });
});
