import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import { AuthProvider } from './auth/AuthProvider';
import './index.css';
import App from './App.tsx';

// Global error handlers to ensure ALL uncaught errors are logged to console (Requirement 15.7)
window.onerror = (message, source, lineno, colno, error) => {
  console.error('Unhandled error:', { message, source, lineno, colno, error });
};

window.onunhandledrejection = (event: PromiseRejectionEvent) => {
  console.error('Unhandled promise rejection:', event.reason);
};

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <BrowserRouter>
      <AuthProvider>
        <App />
      </AuthProvider>
    </BrowserRouter>
  </StrictMode>
);
