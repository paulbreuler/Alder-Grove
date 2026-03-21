import { useAuth, useUser, UserButton, SignInButton } from '@clerk/react';
import { User } from 'lucide-react';

/**
 * Activity bar icon for user profile / auth.
 *
 * Signed out: user icon opens Clerk sign-in modal.
 * Signed in: Clerk UserButton with popup for sign-out/profile.
 */
export function UserAvatar(_props: { isActive: boolean }): React.JSX.Element {
  const { isSignedIn } = useAuth();
  const { user } = useUser();

  if (!isSignedIn || !user) {
    return (
      <SignInButton
        mode="modal"
        forceRedirectUrl={window.location.href}
      >
        <button
          className="flex items-center justify-center"
          aria-label="Sign in"
          // Known limitation: we cannot reliably auto-toggle the panel
          // after the Clerk modal closes because there is no close callback.
          // A setTimeout race condition was removed here. The modal simply
          // closes and the user can navigate manually.
        >
          <User size={22} strokeWidth={1.5} />
        </button>
      </SignInButton>
    );
  }

  return (
    <UserButton
      appearance={{
        elements: {
          rootBox: 'flex',
          avatarBox: 'w-[22px] h-[22px]',
          userButtonTrigger: 'p-0 focus:shadow-none',
          userButtonBox: 'flex-row-reverse',
        },
      }}
    />
  );
}
