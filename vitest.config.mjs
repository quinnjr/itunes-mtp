import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'jsdom', // Use jsdom for Angular component testing
    setupFiles: ['./vitest.setup.ts'],
    include: ['src/**/*.spec.ts'],
    // Suppress console output during tests
    silent: false,
    // Configure test timeout for Angular compilation
    testTimeout: 10000,
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html', 'lcov'],
      exclude: [
        'node_modules/',
        'src-tauri/',
        '**/*.spec.ts',
        '**/*.config.*',
        '**/main.ts',
        '**/polyfills.ts',
      ],
    },
  },
  resolve: {
    // Enable proper resolution for Angular component resources
    alias: {
      '@angular/platform-browser/testing': '@angular/platform-browser/testing',
    },
  },
});

