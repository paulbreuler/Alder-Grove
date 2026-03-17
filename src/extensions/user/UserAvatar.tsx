import { useAuth, useUser, UserButton, SignInButton } from '@clerk/react';
import { User } from 'lucide-react';
import { globalEventBus } from '@paulbreuler/shell';

/**
 * Activity bar icon for user profile / auth.
 *
 * Signed out: user icon opens Clerk sign-in modal. On modal close,
 *   emits left.toggle to return to the previous activity.
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
          onClick={() => {
            // After the modal closes (user signs in or dismisses),
            // navigate back to the previous activity
            setTimeout(() => {
              globalEventBus.emit('left.toggle', undefined);
            }, 100);
          }}
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
