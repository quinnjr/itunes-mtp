# Zoneless Angular Configuration

This Angular application is configured to run without Zone.js, using Angular's zoneless change detection for improved performance and simplified debugging.

## Configuration

### Application Configuration

The zoneless change detection is enabled in `src/app/app.config.ts`:

```typescript
import { provideZonelessChangeDetection } from "@angular/core";

export const appConfig: ApplicationConfig = {
  providers: [
    // ... other providers
    provideZonelessChangeDetection()
  ],
};
```

### Test Configuration

All test files must include `provideZonelessChangeDetection()` in their providers:

```typescript
import { provideZonelessChangeDetection } from '@angular/core';

beforeEach(async () => {
  await TestBed.configureTestingModule({
    imports: [YourComponent],
    providers: [
      provideZonelessChangeDetection(),
      // ... other providers
    ],
  });
  // ...
});
```

The global test setup in `vitest.setup.ts` also configures the test environment for zoneless mode.

## Key Points

1. **No Zone.js Dependency**: `zone.js` is not listed in `package.json` dependencies (it may still appear in the lock file as a transitive dependency of Angular packages, but is not used).

2. **No Zone.js Polyfills**: There are no `zone.js` imports in `angular.json` or any polyfills files.

3. **Service Injection**: All services and components use `inject()` instead of constructor injection for better zoneless compatibility.

4. **Change Detection**: Components use Angular's built-in change detection signals, which work seamlessly in zoneless mode.

## Benefits

- **Improved Performance**: No Zone.js overhead means faster application execution
- **Simplified Debugging**: Stack traces are cleaner without Zone.js patching
- **Smaller Bundle Size**: Removing Zone.js reduces the application bundle size
- **Modern Angular**: Aligns with Angular's future direction toward zoneless by default

## Testing

When writing tests:
- Always include `provideZonelessChangeDetection()` in test providers
- Use `TestBed.compileComponents()` to resolve component resources
- Change detection is automatic in zoneless mode, but `fixture.detectChanges()` still works for manual triggers

## Migration Notes

If you encounter issues:
- Ensure all async operations properly trigger change detection (use signals, async pipe, or manual change detection)
- Replace any `NgZone` usage with appropriate alternatives like `afterRender` or `afterNextRender`
- Verify all components use `inject()` for dependency injection

