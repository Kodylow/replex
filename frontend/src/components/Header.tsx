import { useState } from "react";
import { Menu } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { Tab, Screen } from "../types";

type HeaderProps = {
  activeTab: Tab;
  setActiveTab: (tab: Tab) => void;
  setActiveScreen: (screen: Screen) => void;
};

export default function Header({
  activeTab,
  setActiveTab,
  setActiveScreen,
}: HeaderProps) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <header className="flex justify-between items-center py-2 px-4 border-b relative">
      <h1
        className="text-lg font-bold cursor-pointer hover:underline"
        onClick={() => {
          setActiveTab(Tab.Wallet);
          setActiveScreen(Screen.Home);
        }}
      >
        repl-ex
      </h1>
      <p className="text-xs text-muted-foreground">me@repl-ex.com</p>
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
                setActiveScreen(Screen.Home);
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
    </header>
  );
}
