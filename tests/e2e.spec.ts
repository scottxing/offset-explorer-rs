import { test, expect } from '@playwright/test';
import { ElectronApplication } from 'playwright-electron';
import path from 'path';

// Test configuration
const BASE_URL = 'http://localhost:5173/';
const APP_NAME = 'Offset Explorer 3';

// Test suite for Offset Explorer 3
test.describe('Offset Explorer 3 - E2E Testing', () => {
  let app: ElectronApplication | null = null;

  test.beforeAll(async ({ page }) => {
    // Launch application
    console.log(`Starting ${APP_NAME} E2E test...`);

    try {
      // For Electron/Tauri apps, we use the executable path
      const appPath = path.join(__dirname, '../src-tauri/target/debug/offset-explorer-rust');

      if (process.platform === 'darwin') {
        // macOS
        app = new ElectronApplication({
          executablePath: appPath,
        name: APP_NAME
        });
      } else if (process.platform === 'win32') {
        // Windows
        app = new ElectronApplication({
          executablePath: appPath + '.exe',
          name: APP_NAME
        });
      } else {
        // Linux
        app = new ElectronApplication({
          executablePath: appPath,
          name: APP_NAME
        });
      }

      expect(app).toBeOK();

      // Get the first window
      const window = await app.firstWindow({ pages: [page] });
      expect(window).not.toBe(null();

      // Take a screenshot of initial state
      await page.screenshot({ path: 'screenshots/01-initial.png' });

    } catch (error) {
      console.error('Failed to launch application:', error);
      throw error;
    }
  });

  test.afterAll(async ({ page }) => {
    console.log('Testing completed, cleaning up...');

    // Close the application
    await app.close();
    await page.close();
  });
});
