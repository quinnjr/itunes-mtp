import { Injectable, signal, computed } from '@angular/core';

export interface AsyncState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
}

@Injectable({
  providedIn: 'root'
})
export class AsyncHandlerService {
  /**
   * Create an async state signal for handling async operations
   */
  public createAsyncState<T>(): {
    state: ReturnType<typeof signal<AsyncState<T>>>;
    setLoading: (loading: boolean) => void;
    setData: (data: T) => void;
    setError: (error: string | null) => void;
    reset: () => void;
  } {
    const state = signal<AsyncState<T>>({
      data: null,
      loading: false,
      error: null
    });

    return {
      state,
      setLoading: (loading: boolean) => {
        state.update(current => ({ ...current, loading }));
      },
      setData: (data: T) => {
        state.update(current => ({ ...current, data, loading: false, error: null }));
      },
      setError: (error: string | null) => {
        state.update(current => ({ ...current, error, loading: false }));
      },
      reset: () => {
        state.set({ data: null, loading: false, error: null });
      }
    };
  }

  /**
   * Execute an async operation with proper error handling for zoneless mode
   */
  public async executeAsync<T>(
    operation: () => Promise<T>,
    handlers: {
      setLoading: (loading: boolean) => void;
      setData: (data: T) => void;
      setError: (error: string | null) => void;
    }
  ): Promise<T | null> {
    try {
      handlers.setLoading(true);
      handlers.setError(null);

      const result = await operation();
      handlers.setData(result);

      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      handlers.setError(errorMessage);
      console.error('Async operation failed:', error);

      return null;
    }
  }

  /**
   * Create a computed signal that combines multiple async states
   */
  public combineAsyncStates<T extends Record<string, ReturnType<typeof signal<AsyncState<any>>>>>(
    states: T
  ): ReturnType<typeof computed<{
    loading: boolean;
    error: string | null;
    hasData: boolean;
    states: T;
  }>> {
    return computed(() => {
      const stateValues = Object.values(states).map(state => state());
      const loading = stateValues.some(state => state.loading);
      const errors = stateValues
        .map(state => state.error)
        .filter((error): error is string => error !== null);
      const hasData = stateValues.every(state => state.data !== null);

      return {
        loading,
        error: errors.length > 0 ? errors.join('; ') : null,
        hasData,
        states
      };
    });
  }
}
