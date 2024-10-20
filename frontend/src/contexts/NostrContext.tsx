import React, {
  createContext,
  useState,
  useEffect,
  useContext,
  ReactNode,
} from "react";
import NDK, { NDKEvent, NDKSubscription, NDKFilter } from "@nostr-dev-kit/ndk";
import { useAppUser } from "@/hooks/useApp";

interface NostrContextType {
  userEvents: NDKEvent[];
}

const NostrContext = createContext<NostrContextType>({ userEvents: [] });

export const NostrProvider: React.FC<{ children: ReactNode }> = ({
  children,
}) => {
  const [userEvents, setUserEvents] = useState<NDKEvent[]>([]);
  const user = useAppUser();

  useEffect(() => {
    if (user && user.publicKey) {
      const ndk = new NDK({ explicitRelayUrls: ["wss://relay.damus.io"] });

      ndk.connect().then(() => {
        const filter: NDKFilter = {
          kinds: [1],
          authors: [user.publicKey],
        };
        const sub: NDKSubscription = ndk.subscribe(filter);

        sub.on("event", (event: NDKEvent) => {
          setUserEvents((prevEvents) => [...prevEvents, event]);
        });

        return () => {
          sub.stop();
        };
      });
    }
  }, [user]);

  return (
    <NostrContext.Provider value={{ userEvents }}>
      {children}
    </NostrContext.Provider>
  );
};

export const useNostr = () => useContext(NostrContext);
