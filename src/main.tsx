import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { ClerkProvider } from '@clerk/react';
import { BrowserRouter } from 'react-router-dom';
import { App } from './App';
import '@paulbreuler/shell/styles';
import './app.css';

// Clerk reads VITE_CLERK_PUBLISHABLE_KEY automatically at runtime.
// The SDK types require the prop explicitly for type safety.
const PUBLISHABLE_KEY = import.meta.env.VITE_CLERK_PUBLISHABLE_KEY;
if (!PUBLISHABLE_KEY) {
  throw new Error(
    'Missing VITE_CLERK_PUBLISHABLE_KEY — add it to .env.local',
  );
}

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ClerkProvider
      publishableKey={PUBLISHABLE_KEY}
      afterSignOutUrl="/"
      allowedRedirectProtocols={['grove://']}
    >
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </ClerkProvider>
  </StrictMode>,
);
