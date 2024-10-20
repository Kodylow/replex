import { useContext, useState } from "react";
import { Menu } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { Tab, Screen, APP_ACTION_TYPE } from "../types";
import {
  useAppActiveTab,
  useAppSetActiveTab,
  useAppSetCurrentScreen,
  useAppUser,
} from "../hooks/useApp";
import { AppContext } from "@/contexts/AppContext";

export default function Header() {
  const { dispatch } = useContext(AppContext);
  const activeTab = useAppActiveTab();
  const setActiveTab = useAppSetActiveTab();
  const setCurrentScreen = useAppSetCurrentScreen();
  const [isOpen, setIsOpen] = useState(false);
  const user = useAppUser();
  console.log("user", user);

  return (
    <header className="flex justify-between items-center py-2 px-4 border-b relative">
      <h1
        className="text-lg font-bold cursor-pointer hover:underline"
        onClick={() => {
          setActiveTab(Tab.Wallet);
          setCurrentScreen(Screen.Home);
          dispatch({ type: APP_ACTION_TYPE.CLEAR_TRANSACTION_STATES });
        }}
      >
        repl-ex
      </h1>
      {user && (
        <p className="text-xs text-muted-foreground">{user.name}@repl-ex.com</p>
      )}
      <div className="flex items-center">
        <Sheet open={isOpen} onOpenChange={setIsOpen}>
          <SheetTrigger asChild>
            <Button
              variant="ghost"
              size="sm"
              aria-label="Open menu"
              className="z-10"
            >
              <Menu className="h-4 w-4" />
            </Button>
          </SheetTrigger>
          <SheetContent side="right" className="z-50">
            <SheetHeader>
              <SheetTitle>Menu</SheetTitle>
            </SheetHeader>
            <nav className="flex flex-col gap-2 mt-4">
              <Button
                variant={activeTab === "wallet" ? "default" : "ghost"}
                className="justify-start"
                onClick={() => {
                  setActiveTab(Tab.Wallet);
                  setCurrentScreen(Screen.Home);
                  setIsOpen(false);
                }}
              >
                Wallet
              </Button>
              <Button
                variant={activeTab === "settings" ? "default" : "ghost"}
                className="justify-start"
                onClick={() => {
                  setActiveTab(Tab.Settings);
                  setIsOpen(false);
                }}
              >
                Settings
              </Button>
            </nav>
          </SheetContent>
        </Sheet>
      </div>
    </header>
  );
}
