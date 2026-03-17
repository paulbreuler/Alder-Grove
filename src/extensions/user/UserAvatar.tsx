import { useAuth, useUser, UserButton } from '@clerk/react';

/**
 * Activity bar icon: shows Clerk's UserButton when signed in,
 * or a generic user icon when signed out.
 */
export function UserAvatar({ isActive }: { isActive: boolean }): React.JSX.Element {
  const { isSignedIn } = useAuth();
  const { user } = useUser();

  if (!isSignedIn || !user) {
    return (
      <div
        className="w-5 h-5 rounded-full bg-bg-raised"
        style={{ opacity: isActive ? 1 : 0.7 }}
      />
    );
  }

  return (
    <UserButton
      appearance={{
        elements: {
          avatarBox: 'w-5 h-5',
          userButtonTrigger: 'p-0',
        },
      }}
    />
  );
}
