import { useAuth } from '@clerk/react';
import { LoginScreen } from '@/components/auth/LoginScreen';

export function HomeEditor(): React.JSX.Element {
  const { isSignedIn, isLoaded } = useAuth();

  if (!isLoaded) {
    return (
      <div className="flex items-center justify-center h-full text-text-muted">
        Loading…
      </div>
    );
  }

  if (!isSignedIn) {
    return <LoginScreen />;
  }

  return (
    <div className="flex flex-col items-center justify-center h-full gap-4 text-text-secondary">
      <h1 className="text-2xl font-semibold text-text-primary">
        Alder Grove
      </h1>
      <p>Your applications grow in the Grove.</p>
    </div>
  );
}
