import React, { Component, type ReactNode } from 'react';
import { Container, Alert } from 'react-bootstrap';
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
        <Container className="mt-5">
          <Alert variant="danger">
            <Alert.Heading>Something went wrong</Alert.Heading>
            <p>The application encountered an unexpected error.</p>
            {this.state.error && (
              <details className="mt-3">
                <summary>Error details</summary>
                <pre className="mt-2 p-2 bg-light border rounded">
                  {this.state.error.toString()}
                </pre>
              </details>
            )}
            <hr />
            <div className="d-flex justify-content-end">
              <Button onClick={this.handleReload}>Reload Page</Button>
            </div>
          </Alert>
        </Container>
      );
    }

    return this.props.children;
  }
}
