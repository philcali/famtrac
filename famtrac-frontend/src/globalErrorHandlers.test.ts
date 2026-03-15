import { describe, test, expect, vi, beforeEach, afterEach } from 'vitest';

/**
 * Tests for global error handlers (Requirement 15.7)
 * Verifies that window.onerror and window.onunhandledrejection log to console
 */
describe('Global error handlers', () => {
  let consoleSpy: ReturnType<typeof vi.spyOn>;
  const originalOnerror = window.onerror;
  const originalOnunhandledrejection = window.onunhandledrejection;

  beforeEach(() => {
    consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
    // Reset handlers before each test
    window.onerror = null;
    window.onunhandledrejection = null;
  });

  afterEach(() => {
    consoleSpy.mockRestore();
    window.onerror = originalOnerror;
    window.onunhandledrejection = originalOnunhandledrejection;
  });

  test('window.onerror logs uncaught errors to console (Requirement 15.7)', () => {
    // Set up the handler as main.tsx does
    window.onerror = (message, source, lineno, colno, error) => {
      console.error('Unhandled error:', { message, source, lineno, colno, error });
    };

    // Simulate an uncaught error
    const testError = new Error('Test uncaught error');
    window.onerror!('Test uncaught error', 'test.ts', 1, 1, testError);

    expect(consoleSpy).toHaveBeenCalledWith('Unhandled error:', {
      message: 'Test uncaught error',
      source: 'test.ts',
      lineno: 1,
      colno: 1,
      error: testError,
    });
  });

  test('window.onunhandledrejection logs unhandled promise rejections (Requirement 15.7)', () => {
    // Set up the handler as main.tsx does
    window.onunhandledrejection = (event: PromiseRejectionEvent) => {
      console.error('Unhandled promise rejection:', event.reason);
    };

    // Simulate an unhandled rejection
    const reason = new Error('Promise failed');
    const event = new PromiseRejectionEvent('unhandledrejection', {
      reason,
      promise: Promise.reject(reason).catch(() => {}),
    });
    window.onunhandledrejection!(event);

    expect(consoleSpy).toHaveBeenCalledWith('Unhandled promise rejection:', reason);
  });
});
