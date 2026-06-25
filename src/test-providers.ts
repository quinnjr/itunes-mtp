import { provideZonelessChangeDetection } from '@angular/core';

// Global providers applied to every test via the Angular unit-test builder
// (`providersFile` in angular.json). The application runs zoneless
// (see app.config.ts) and Zone.js is not installed, so tests must use
// zoneless change detection to avoid NG0908 ("requires Zone.js").
export default [provideZonelessChangeDetection()];
