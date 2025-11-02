import { ComponentFixture, TestBed } from '@angular/core/testing';
import { provideAnimations } from '@angular/platform-browser/animations';
import { provideHttpClient } from '@angular/common/http';
import { NO_ERRORS_SCHEMA, provideZonelessChangeDetection } from '@angular/core';
import { vi } from 'vitest';

import { HomeComponent } from './home.component';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue([])
}));

describe('HomeComponent', () => {
  let component: HomeComponent;
  let fixture: ComponentFixture<HomeComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [HomeComponent],
      providers: [
        provideAnimations(),
        provideHttpClient(),
        provideZonelessChangeDetection()
      ],
      schemas: [NO_ERRORS_SCHEMA]
    });

    // Compile components and wait for resolution
    await TestBed.compileComponents();
    await TestBed.flushEffects();

    fixture = TestBed.createComponent(HomeComponent);
    component = fixture.componentInstance;
    // In zoneless mode, detectChanges() still works but change detection is automatic
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
