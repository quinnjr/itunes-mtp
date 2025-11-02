import { TestBed } from '@angular/core/testing';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { AsyncHandlerService } from './async-handler.service';

describe('AsyncHandlerService', () => {
  let service: AsyncHandlerService;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [AsyncHandlerService]
    });

    service = TestBed.inject(AsyncHandlerService);
  });

  describe('Service Initialization', () => {
    it('should be created', () => {
      expect(service).toBeTruthy();
    });
  });

  describe('createAsyncState', () => {
    it('should create async state with initial values', () => {
      const { state } = service.createAsyncState<string>();

      const initialState = state();
      expect(initialState.data).toBeNull();
      expect(initialState.loading).toBe(false);
      expect(initialState.error).toBeNull();
    });

    it('should set loading state', () => {
      const { state, setLoading } = service.createAsyncState<string>();

      setLoading(true);

      expect(state().loading).toBe(true);
      expect(state().data).toBeNull();
      expect(state().error).toBeNull();
    });

    it('should set data state', () => {
      const { state, setData } = service.createAsyncState<string>();

      setData('test data');

      expect(state().data).toBe('test data');
      expect(state().loading).toBe(false);
      expect(state().error).toBeNull();
    });

    it('should set error state', () => {
      const { state, setError } = service.createAsyncState<string>();

      setError('test error');

      expect(state().error).toBe('test error');
      expect(state().loading).toBe(false);
      expect(state().data).toBeNull();
    });

    it('should reset state', () => {
      const { state, setData, setError, reset } = service.createAsyncState<string>();

      setData('test');
      setError('error');
      reset();

      expect(state().data).toBeNull();
      expect(state().loading).toBe(false);
      expect(state().error).toBeNull();
    });
  });

  describe('executeAsync', () => {
    it('should execute successful async operation', async () => {
      const setLoading = vi.fn();
      const setData = vi.fn();
      const setError = vi.fn();

      const result = await service.executeAsync(
        async () => 'success',
        { setLoading, setData, setError }
      );

      expect(result).toBe('success');
      expect(setLoading).toHaveBeenCalledWith(true);
      expect(setData).toHaveBeenCalledWith('success');
      expect(setError).toHaveBeenCalledWith(null);
    });

    it('should handle errors in async operation', async () => {
      const setLoading = vi.fn();
      const setData = vi.fn();
      const setError = vi.fn();

      const result = await service.executeAsync(
        async () => {
          throw new Error('Test error');
        },
        { setLoading, setData, setError }
      );

      expect(result).toBeNull();
      expect(setLoading).toHaveBeenCalledWith(true);
      expect(setError).toHaveBeenCalledWith('Test error');
      expect(setData).not.toHaveBeenCalled();
    });

    it('should handle string errors', async () => {
      const setLoading = vi.fn();
      const setData = vi.fn();
      const setError = vi.fn();

      await service.executeAsync(
        async () => {
          throw 'String error';
        },
        { setLoading, setData, setError }
      );

      expect(setError).toHaveBeenCalledWith('String error');
    });

    it('should handle non-Error objects', async () => {
      const setLoading = vi.fn();
      const setData = vi.fn();
      const setError = vi.fn();

      await service.executeAsync(
        async () => {
          throw { message: 'Object error' };
        },
        { setLoading, setData, setError }
      );

      expect(setError).toHaveBeenCalled();
    });
  });

  describe('combineAsyncStates', () => {
    it('should combine multiple async states', () => {
      const state1 = service.createAsyncState<string>();
      const state2 = service.createAsyncState<number>();

      const combined = service.combineAsyncStates({
        state1: state1.state,
        state2: state2.state
      });

      const combinedValue = combined();
      expect(combinedValue.loading).toBe(false);
      expect(combinedValue.hasData).toBe(false);
      expect(combinedValue.error).toBeNull();
    });

    it('should detect loading state', () => {
      const state1 = service.createAsyncState<string>();
      const state2 = service.createAsyncState<number>();

      state1.setLoading(true);

      const combined = service.combineAsyncStates({
        state1: state1.state,
        state2: state2.state
      });

      expect(combined().loading).toBe(true);
    });

    it('should detect hasData when all states have data', () => {
      const state1 = service.createAsyncState<string>();
      const state2 = service.createAsyncState<number>();

      state1.setData('test');
      state2.setData(42);

      const combined = service.combineAsyncStates({
        state1: state1.state,
        state2: state2.state
      });

      expect(combined().hasData).toBe(true);
    });

    it('should combine errors', () => {
      const state1 = service.createAsyncState<string>();
      const state2 = service.createAsyncState<number>();

      state1.setError('Error 1');
      state2.setError('Error 2');

      const combined = service.combineAsyncStates({
        state1: state1.state,
        state2: state2.state
      });

      expect(combined().error).toContain('Error 1');
      expect(combined().error).toContain('Error 2');
    });
  });
});

