import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'jsdom', // Use jsdom for Angular component testing
    setupFiles: ['./vitest.setup.ts'],
    include: ['src/**/*.spec.ts'],
    // Suppress console output during tests
    silent: false,
  },
});

