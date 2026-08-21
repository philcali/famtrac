import React, { Component, type ReactNode } from 'react';
import { Button } from './Button';

interface ErrorBoundaryProps {
  children: ReactNode;
}

interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

/**
 * ErrorBoundary component
 * - Catches React errors and displays fallback UI (Requirement 15.7)
 * - Logs errors to console for debugging (Requirement 15.7)
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
    };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo): void {
    // Log error to console for debugging (Requirement 15.7)
    console.error('Application error caught by ErrorBoundary:', error, errorInfo);
  }

  handleReload = (): void => {
    window.location.reload();
  };

  render(): ReactNode {
    if (this.state.hasError) {
      return (
        <div className="mt-5 max-w-5xl mx-auto px-4">
          <div className="p-4 bg-red-50 border border-red-100 rounded-xl text-red-700 text-sm">
            <h2 className="text-base font-semibold mb-2">Something went wrong</h2>
            <p className="mb-3">The application encountered an unexpected error.</p>
            {this.state.error && (
              <details className="mt-3">
                <summary className="cursor-pointer font-medium">Error details</summary>
                <pre className="mt-2 p-2 bg-gray-50 border rounded text-xs overflow-auto">
                  {this.state.error.toString()}
                </pre>
              </details>
            )}
            <hr className="my-3 border-gray-200" />
            <div className="flex justify-end">
              <Button onClick={this.handleReload}>Reload Page</Button>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
