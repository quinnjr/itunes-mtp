// Vitest setup file to handle PrimeNG compatibility issues and Angular test environment
import { vi } from 'vitest';
import { TestBed } from '@angular/core/testing';
import { BrowserDynamicTestingModule, platformBrowserDynamicTesting } from '@angular/platform-browser-dynamic/testing';

// Mock PrimeNG modules before Angular initializes to prevent compatibility issues with Angular 20
vi.mock('primeng/fileupload', () => ({
  FileUpload: vi.fn().mockImplementation(() => {
    return {
      selector: 'p-fileupload',
      template: '<div>Mock FileUpload</div>',
      standalone: true
    };
  })
}));

// Mock entire primeng package to prevent initialization errors
vi.mock('primeng', () => ({
  FileUpload: vi.fn(),
  __esModule: true,
  default: {}
}));

// Initialize Angular test environment
TestBed.initTestEnvironment(
  BrowserDynamicTestingModule,
  platformBrowserDynamicTesting()
);

// Suppress console errors from PrimeNG during tests (keep console.log for debugging)
const originalError = console.error;
const originalWarn = console.warn;
console.error = vi.fn((...args) => {
  // Only suppress PrimeNG-related errors
  if (typeof args[0] === 'string' && args[0].includes('primeng')) {
    return;
  }
  originalError.apply(console, args);
});
console.warn = vi.fn((...args) => {
  if (typeof args[0] === 'string' && args[0].includes('primeng')) {
    return;
  }
  originalWarn.apply(console, args);
});

