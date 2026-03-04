# Famtrac Frontend

A modern single-page application (SPA) for managing families, dependents, and their activities.

## Technology Stack

- **Vite** - Fast build tool and dev server
- **React 18** - UI library with hooks
- **TypeScript** - Type safety
- **React Router v6** - Client-side routing
- **React Bootstrap** - UI components
- **Vitest** - Unit testing
- **fast-check** - Property-based testing

## Getting Started

### Prerequisites

- Node.js 18+ and npm

### Installation

1. Install dependencies:
```bash
npm install
```

2. Copy the example environment file and configure it:
```bash
cp .env.example .env.development
```

3. Update `.env.development` with your configuration:
   - Set `VITE_API_BASE_URL` to your backend API URL
   - Configure AWS Cognito settings

### Development

Start the development server:
```bash
npm run dev
```

The application will be available at `http://localhost:5173`

### Building

Build for production:
```bash
npm run build
```

Preview the production build:
```bash
npm run preview
```

### Testing

Run tests:
```bash
npm run test
```

Run tests in watch mode:
```bash
npm run test:watch
```

### Code Quality

Run linter:
```bash
npm run lint
```

Format code:
```bash
npm run format
```

## Project Structure

```
src/
├── api/              # API client and endpoint methods
├── auth/             # Authentication context and guards
├── components/       # Reusable UI components
├── config/           # Environment and configuration
├── hooks/            # Custom React hooks
├── pages/            # Route-level page components
├── styles/           # Custom CSS styles
├── test/             # Test setup and utilities
├── types/            # TypeScript type definitions
└── utils/            # Utility functions
```

## Environment Variables

Required environment variables (see `.env.example`):

- `VITE_API_BASE_URL` - Backend API base URL
- `VITE_COGNITO_DOMAIN` - AWS Cognito domain
- `VITE_COGNITO_CLIENT_ID` - Cognito app client ID
- `VITE_COGNITO_REDIRECT_URI` - OAuth redirect URI
- `VITE_COGNITO_LOGOUT_URI` - Logout redirect URI
- `VITE_COGNITO_SCOPE` - OAuth scopes

## License

Private - All rights reserved
