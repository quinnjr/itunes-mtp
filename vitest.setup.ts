// Vitest setup file to handle PrimeNG compatibility issues and Angular test environment
import { vi } from 'vitest';
import { TestBed } from '@angular/core/testing';
import { BrowserDynamicTestingModule, platformBrowserDynamicTesting } from '@angular/platform-browser-dynamic/testing';

// Initialize Angular test environment
TestBed.initTestEnvironment(
  BrowserDynamicTestingModule,
  platformBrowserDynamicTesting()
);

// Mock PrimeNG FileUpload component to avoid Angular 20 compatibility issues
vi.mock('primeng/fileupload', () => ({
  FileUpload: {
    __name: 'FileUpload',
    selector: 'p-fileupload',
    template: '<div></div>'
  }
}));

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

