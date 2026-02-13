import { defineConfig, devices } from '@playwright/test';

// Playwright configuration for Offset Explorer 3 testing
export default defineConfig({
  testDir: './e2e-tests',
  timeout: 30000,
  retries: 2,
  use: {
    // Launch options
    headless: false,
    viewport: { width: 1280, height: 720 },
  // Screenshot options
    screenshot: {
      fullPage: true,
      animations: 'disabled',
    type: 'png',
    },
  },
  },
  projects: [
    {
      name: 'chromium',
      use: devices['Desktop Chrome'],
    },
  ],
});
